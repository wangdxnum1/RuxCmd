# Round 9 实现计划：网络工具集

## 任务总览

| 任务 | 工具 | 内容 |
|------|------|------|
| Task 0 | 脚手架 | 创建 curl、wget、ping、netstat、nslookup 五个新 crate |
| Task 1 | nslookup | 实现 DNS 域名解析 |
| Task 2 | wget | 实现 HTTP 文件下载 |
| Task 3 | curl | 实现 HTTP/HTTPS 客户端 |
| Task 4 | ping | 实现 ICMP ping |
| Task 5 | netstat | 实现网络连接查看 |
| Task 6 | 构建验证 | 运行 build-all.bat |

---

## Task 0: 脚手架

创建五个新工具的基础结构。

---

## Task 1: nslookup

**CLI**：
```
nslookup [NAME] [SERVER]
  -type=TYPE  查询类型（A/AAAA/CNAME/MX/TXT）
  -v, --version
```

**实现**：
- 使用 `std::net::ToSocketAddrs` 解析域名
- 支持基本的 A 记录查询
- 格式化输出

---

## Task 2: wget

**CLI**：
```
wget [OPTIONS] URL
  -O FILE  输出文件名
  -P DIR   输出目录
  -q       安静模式
  -v, --version
```

**实现**：
- URL 解析（scheme, host, path）
- HTTP GET 请求
- 文件下载与保存

---

## Task 3: curl

**CLI**：
```
curl [OPTIONS] URL
  -X METHOD  HTTP 方法
  -H HEADER  请求头
  -d DATA    POST 数据
  -o FILE    输出文件
  -s         静默模式
  -v, --version
```

**实现**：
- HTTP/1.1 请求构建
- 响应解析
- 支持 GET/POST

---

## Task 4: ping

**CLI**：
```
ping [OPTIONS] HOST
  -c N      发送次数
  -t N      超时
  -s N      数据包大小
  -v, --version
```

**实现**：
- Windows ICMP API
- 发送 echo 请求
- 计算延迟

---

## Task 5: netstat

**CLI**：
```
netstat [OPTIONS]
  -a        显示所有连接
  -n        数字格式
  -p PROTO  协议
  -v, --version
```

**实现**：
- Windows TCP/UDP 连接查询
- 格式化输出

---

## Task 6: 构建验证

运行 build-all.bat，验证所有工具。