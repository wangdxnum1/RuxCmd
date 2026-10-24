# Changelog

## 0.2.0 — 缺陷修复与行为验证轮（2026-10-25）

- 完成第 12 轮 P1 缺陷修复(批次 A1-I1,约 45 个命令):进程工具原生化(pkill/pgrep/kill 弃用 tasklist/wmic/taskkill,复用 ps-sys;w/who 改 WTSEnumerateSessions);真实数据源(iostat/mpstat/sar/time/top/vmstat 的 GetSystemTimes、ps 的存活期 %CPU);文本工具字节安全化(grep/cut/sort/head/tail/sed/fold 的非 UTF-8 透传);sed 地址与 p/d/q 命令;awk 条件求值重写;网络工具语义修正(curl body-only/-d POST/-L/-f、wget 退出码 8、ping -t 持续语义);netstat -o PID、xargs -0 与分批、find GNU maxdepth 与单横线选项;chmod 符号模式、test POSIX 子集、env 命令执行;md5sum/sha256sum --tag/--status、od 全规格、iconv 严格 -f、numfmt 分隔符;tar -C/-O/--exclude、patch 多文件按计数解析等。
- 完成第 11 轮 P0 缺陷修复(48 任务,覆盖审查报告全部 50 项 P0):每个修复附行为回归测试,涉及命令的验证状态由"仅冒烟"升级为"冒烟 + 行为测试"。
  - 安全:tar/unzip 解包拒绝路径穿越条目(退出码 2);zip 参数经 PowerShell 单引号转义防注入;sync 防自递归并传播退出码。
  - 数据正确性:cat 字节透明化;base64 解码重写;touch 经 SetFileTime 真实更新时间戳;echo 默认不解释转义;printf 实现格式串转义;bc 求值器重写。
  - 进程控制:nice 改用优先级类并透传退出码;timeout 轮询子进程、超时返回 124 并实现 -k。
  - 伪造数据清除:nslookup 经 DnsQuery_W 真实查询 MX/TXT/CNAME;id/groups 取自进程令牌;ps %MEM 用真实总内存;chown --group 正确设置组;tty 改用 IsTerminal;vmstat 逐帧采样。
  - 输出与路径语义:which/readlink/realpath/dirname/basename 输出净化;ls 着色下长格式列不丢失;stat -c 对齐 GNU;sort 实现 -k/-t;diff 退出码与行号;uniq/tac/nl/wc/tee/tail/split/csplit/column/pr/shuf/du/grep/file 语义修复。
  - 附加:df/du/free/pr/whoami/sort 的 -h 与 help 冲突(六命令 debug 构建启动即 panic)修复;test 的 clap 参数名冲突修复。
- 完成全部 111 个命令的源码级代码审查(报告 docs/command-review-2026-10-07.md):50 个命令存在 P0 级缺陷,报告给出逐命令的功能/移植完整性/技术评估与 P0-P2 分级。
- 制定第 11 轮 P0 缺陷修复设计与计划(specs/2026-10-07-p0-remediation-design.md、plans/2026-10-07-p0-remediation.md):安全(zip slip/命令注入/自递归)、数据损坏、panic、伪造数据清除四大类,48 个任务。
- 清理提交进仓库的手工测试杂物(29 个 crate 根目录的 test_*/checksum/fixture 文件与 ls 的 .codegraph/reasonix.toml),.codegraph 加入 .gitignore。
- 新增全仓库 MIT 许可证（LICENSE）、贡献者行为准则（CODE_OF_CONDUCT.md）与安全策略（SECURITY.md）。
- 根 Cargo.toml 增加 `[workspace.package]`，113 个 crate 统一声明 license/repository 元数据。
- 贡献规范、开发说明、命令兼容性表及脚本使用说明提供中英双语版本，中文版使用 `-zh.md` 后缀；两份 README 更新徽章、文档链接与许可证章节。
- 清理迁移遗留的根目录构建产物（游离 `ls/` 目录、`build-output.log`、`rust.html`）。
- 用 PowerShell 构建入口替代 build-all.bat，支持任务、单包、Debug/Release、离线、并行数及只读预览。
- 增加本地与 GitHub 发布入口，复用清单、校验和及解压验证，拒绝冲突 tag 和覆盖已有 Release。
- CI 与 Release workflow 复用统一入口，补充入口失败路径测试及中英文使用说明。

## 0.1.0 — 基础设施基线

- 将 111 个 Windows 命令归入根 Cargo workspace，固定工具链并统一依赖锁定。
- 增加代表性行为测试及每个命令的发布冒烟策略。
- 增加带源码信息、PE 依赖记录、SHA-256 及解压验证的 ZIP 打包流程。
- 增加 Windows CI、手动发行 artifact workflow 及贡献文档。
- 修复 cp/mv 源文件参数声明导致的 debug 启动断言，以及 grep 普通模式未匹配时的退出码。
- 将 csplit 的溢出边界检查改为 checked_add，消除 Clippy 阻断。

此版本建立发布基线；不代表所有命令已经完整兼容 GNU/POSIX 行为。
