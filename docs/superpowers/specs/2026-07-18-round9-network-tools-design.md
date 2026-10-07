# 第 9 轮 Linux 工具移植设计：网络工具集

**日期**：2026-07-18
**作者**：Tim
**背景**：前 8 轮已移植 91 个 Linux 命令行工具到 Windows (Rust, 静态 CRT MT 模式)。本轮移植 5 个网络类工具。

---

## 1. 范围与目标

### 1.1 本轮移植的 5 个工具

| # | 工具 | 功能简述 | 实现难度 | 依赖策略 |
|---|------|----------|----------|----------|
| 1 | `curl` | HTTP/HTTPS 客户端，支持 GET/POST 请求 | ⭐⭐⭐ | 标准库 hyper（或手写 HTTP） |
| 2 | `wget` | 文件下载工具 | ⭐⭐ | 标准库 + 简单 HTTP |
| 3 | `ping` | ICMP 网络连通性测试 | ⭐⭐⭐ | 标准库（Windows raw socket） |
| 4 | `netstat` | 查看网络连接状态 | ⭐⭐ | Windows API（sysinfo 或 winapi） |
| 5 | `nslookup` | DNS 域名解析 | ⭐⭐ | 标准库 std::net |

### 1.2 不在范围内

- 不引入新 crate 依赖（除 `clap`）；
- curl/wget 不实现完整协议支持（仅 HTTP/HTTPS GET/POST）；
- ping 不支持复杂 ICMP 选项；
- netstat 不支持所有平台的全部功能。

---

## 2. 总体架构与代码组织

### 2.1 目录结构

```
d:\Work\rust\
├── curl/
├── wget/
├── ping/
├── netstat/
└── nslookup/
```

### 2.2 Cargo.toml 统一模板

```toml
[package]
name = "<TOOL_NAME>"
version = "0.1.0"
edition = "2024"
description = "<SHORT_DESCRIPTION>"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

**curl/wget 额外依赖**（HTTPS 支持）：
```toml
reqwest = { version = "0.12", features = ["blocking", "rustls-tls"] }
```

**ping/netstat 额外依赖**（Windows API）：
```toml
windows-sys = { version = "0.61", features = ["Win32_Foundation", "Win32_NetworkManagement_IpHelper"] }
```

### 2.3 关键约定

- `disable_version_flag = true` + 自定义 `-v/--version`；
- 错误输出格式：`"<tool>: <message>"`；
- `-` 表示 stdin/stdout。

---

## 3. 各工具详细设计

### 3.1 curl

**CLI**：
```
curl [OPTIONS] URL
  -X, --request=METHOD     HTTP 方法（GET/POST/PUT/DELETE，默认 GET）
  -H, --header=HEADER      请求头
  -d, --data=DATA          POST 数据
  -o, --output=FILE        输出文件
  -s, --silent             静默模式
  -v, --version            版本
```

**实现要点**：
- 使用 `reqwest` 库（Rust 生态最流行的 HTTP 客户端）；
- 原生支持 HTTP 和 HTTPS，自动处理 TLS 证书验证；
- 支持 GET/POST/PUT/DELETE 方法；
- 自定义请求头和请求体；
- 输出到 stdout 或指定文件。

---

### 3.2 wget

**CLI**：
```
wget [OPTIONS] URL
  -O, --output-document=FILE  输出文件名
  -P, --directory-prefix=DIR  输出目录
  -q, --quiet                 安静模式
  -v, --version               版本
```

**实现要点**：
- 使用 `reqwest` 库支持 HTTP/HTTPS；
- 解析 URL，提取文件名；
- 发送 HTTP GET 请求；
- 写入文件（默认同名或指定路径）；
- 支持自定义输出目录。

---

### 3.3 ping

**CLI**：
```
ping [OPTIONS] HOST
  -c, --count=N        发送次数
  -t, --timeout=N      超时时间（秒）
  -s, --size=N         数据包大小
  -v, --version        版本
```

**实现要点**：
- Windows 使用 `ICMP.dll` 或 raw socket；
- 发送 ICMP echo 请求；
- 接收响应并计算往返时间；
- 统计成功率和延迟。

---

### 3.4 netstat

**CLI**：
```
netstat [OPTIONS]
  -a, --all            显示所有连接
  -n, --numeric        数字格式（不解析主机名）
  -p, --protocol=PROTO 协议（tcp/udp）
  -v, --version        版本
```

**输出格式**：
```
Proto  Local Address          Foreign Address        State
TCP    127.0.0.1:8080         0.0.0.0:0              LISTENING
TCP    192.168.1.100:443      10.0.0.1:12345         ESTABLISHED
```

**实现要点**：
- Windows 使用 `GetTcpTable2` API；
- 解析 TCP/UDP 连接表；
- 格式化输出。

---

### 3.5 nslookup

**CLI**：
```
nslookup [OPTIONS] [NAME] [SERVER]
  -type=TYPE           查询类型（A/AAAA/CNAME/MX/TXT）
  -v, --version        版本
```

**实现要点**：
- 使用标准库 `std::net::ToSocketAddrs` 或 DNS 查询；
- 解析域名到 IP 地址；
- 支持常见记录类型。

---

## 4. 错误处理规范

| 场景 | stderr 格式 | 退出码 |
|------|-------------|--------|
| 网络连接失败 | `"<tool>: connection failed: <error>"` | 1 |
| URL 解析失败 | `"<tool>: invalid URL: '<url>'"` | 1 |
| DNS 解析失败 | `"<tool>: DNS lookup failed: <error>"` | 1 |
| 文件写入失败 | `"<tool>: write error: <error>"` | 1 |

---

## 5. 构建集成

PROJECTS 变量末尾追加：
```
curl wget ping netstat nslookup
```

---

## 6. 验收检查清单

| 项 | 说明 |
|----|------|
| 编译通过 | 5 个工具全部成功 |
| 基本功能冒烟 | 每个工具至少 3 条验证 |
| build-all.bat 集成 | 新工具已加入 |
| 产物复制 | 输出目录出现新 exe |
| Git 状态 | 只显示预期文件 |

---

## 7. 风险与回退

| 风险 | 缓解策略 |
|------|----------|
| HTTPS 支持复杂 | 使用 `reqwest` 库原生支持，自动处理 TLS |
| ping 需要管理员权限 | 在帮助信息中注明 |
| netstat API 差异 | 使用 Windows 原生 API (`windows-sys`) |