# pwd — Linux 风格的 Windows 工作目录打印器 (Rust)

一个原生的 Windows 版 GNU coreutils `pwd` 命令。标志、输出格式和错误信息遵循 Linux 原版风格，让你在 Windows 上也能保持 Linux 的操作习惯。

> 使用正斜杠分隔符输出当前工作目录，自动去除 Windows `\\?\` 扩展长度前缀，并在逻辑模式下支持 `$PWD` 环境变量。

## 快速开始

```powershell
# 从源码构建
cargo build --release
.\target\release\pwd.exe
```

```bash
# 默认（逻辑模式）
pwd

# 物理模式 — 解析所有符号链接 / junction
pwd -P

# 逻辑模式 — 显式指定
pwd -L
```

## 命令标志

| 短选项 | 长选项          | 描述                                                      |
|--------|-----------------|-----------------------------------------------------------|
| `-L`   | `--logical`     | 如果 `$PWD` 匹配当前目录则打印其值（默认行为）            |
| `-P`   | `--physical`    | 解析所有符号链接，打印规范化的物理路径                    |
|        | `--help`        | 显示帮助信息并退出                                        |
|        | `--version`     | 显示版本信息并退出                                        |

当同时指定 `-L` 和 `-P` 时，**最后一个生效**（与 GNU 行为一致）。

## 退出码

| 退出码 | 含义 |
|--------|------|
| `0`    | 成功 |
| `1`    | 失败（如当前目录不可访问） |

## 错误格式

```
pwd: error retrieving current directory: <原因>
```

与 GNU coreutils 的错误前缀一致，确保脚本兼容性。

## 逻辑模式与物理模式

**逻辑模式（`-L`，默认）：**

1. 读取 `PWD` 环境变量。
2. 如果 `PWD` 已设置且解析后与 `current_dir()` 指向同一目录，
   则打印 `PWD` 的值（保留用户可见的符号链接 / junction）。
3. 否则回退到 `std::env::current_dir()`。

**物理模式（`-P`）：**

1. 使用 `std::fs::canonicalize(".")` 解析所有符号链接、NTFS
   junction 和重解析点。
2. 去除 `\\?\` 扩展长度前缀。

## Windows 特殊处理

- 路径分隔符规范化：`\` → `/`
- 从规范化路径中去除 `\\?\` 扩展长度前缀
- 当工作目录已被删除时优雅报错

## 项目架构

```
pwd/
├── src/
│   ├── cli.rs       # clap derive CLI 参数定义
│   └── main.rs      # 入口点 + 路径解析逻辑
├── Cargo.toml
└── .gitignore
```

## 许可证

MIT — 详见 [LICENSE](LICENSE)。
