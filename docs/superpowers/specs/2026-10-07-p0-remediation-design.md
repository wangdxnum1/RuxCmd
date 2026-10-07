# 第 11 轮设计:P0 缺陷修复(安全、数据正确性与契约修复)

**日期**:2026-10-07
**作者**:Tim
**背景**:第 7-10 轮完成 111 个命令的移植与 workspace 基础设施后,2026-10-07 对全部命令做了源码级代码审查(报告见 [command-review-2026-10-07.md](../../command-review-2026-10-07.md))。审查发现 50 个命令存在 50 项 P0 级正确性缺陷——在最普通用法下产生错误输出、panic、伪造数据或安全风险。本轮集中修复全部 P0 项,每个修复附带回归测试。

---

## 1. 范围与目标

### 1.1 目标

1. 修复审查报告 P0 清单(50 项,50 个命令)的全部缺陷;
2. 每个修复附至少一个回归测试(在对应 crate 的 `tests/` 中,先失败后通过);
3. 消除两类系统性反模式:**伪造输出**(无法实现的查询必须显式报错)与**失败仍退出 0**;
4. 消除本轮涉及命令的全部 panic 路径(unwrap/expect/算术下溢);
5. 修复导致的语义变化同步到 `docs/compatibility.md`(双语);`docs/ps/README.md` 中未注册列的声明同步;
6. 清理提交进仓库的手工测试杂物文件;
7. 全程保持 `scripts/build.ps1 -Task Verify` 通过。

### 1.2 非目标

- P1/P2 缺陷不在本轮(除非与 P0 是同一处代码,如 no-op 选项的实现/移除决策);
- 不引入共享的"字节安全行读取层"等基础设施(基础设施轮再做;本轮允许 2-3 个 crate 内重复小工具函数,如 `\\?\` 前缀剥离);
- 不追求完整 GNU/POSIX 兼容;**已声明的有意差异不变**(如 mkfifo 管道语义、chmod readonly 映射、sar 无历史文件);
- 性能数据源的大规模重写(如 mpstat/iostat/sar 接入 PDH)属 P1,不做——但它们的 P0 邻居(vmstat 不重采样、top 下溢、uptime 错误 API、ps 硬编码内存)必须修;
- 不新增重量级依赖;允许给 `windows` crate 增加 feature(如 `Win32_NetworkManagement_Dns`),不允许拉入 GMP/hickory 级别的依赖;
- 不做 ARM64/x86、最低 Windows 版本验证。

### 1.3 与第 10 轮(系统监控)的关系

第 10 轮设计的 vmstat/iostat/mpstat/sar/top 已实现,本轮不重做,只修复审查发现的 P0(vmstat 重采样、top 下溢)。iostat/mpstat/sar 的全零假数据列为 P1,留给后续"真实数据源"轮。

---

## 2. 修复原则(设计决策)

本轮所有任务遵循以下决策;个别命令的特殊决策在计划任务中标注。

### 2.1 安全(P0 中的最高优先级)

| 命令 | 决策 |
|---|---|
| tar / unzip | 解包条目路径必须落在目标目录内:拒绝绝对路径、盘符前缀、含 `..` 组件的条目(tar 用组件校验,unzip 用 `enclosed_name()`)。恶意归档用测试 fixture 锁定行为。 |
| zip | 禁止把未转义的用户输入拼进 PowerShell 命令串。所有路径参数经单引号转义(内部 `'` 加倍)后拼接;新增转义函数的单元测试。原生 zip 实现是后续改进,本轮不做。 |
| sync | 禁止按裸名 `sync` spawn(会递归拉起自身):先在 PATH 中解析候选路径,若与 `current_exe()` 相同则直接走 `rundll32` 回退;传播外部命令退出码。 |

### 2.2 伪造输出清除

无法真实实现的查询,宁可选**显式报错退出非零**,不输出貌似合理的假数据:

- `nslookup` MX/TXT/CNAME:优先用 `windows` crate 的 `DnsQuery_W`(dnsapi.dll)实现真实查询;若实现中发现工作量失控,降级为"该记录类型在 Windows 上不支持"错误退出 1,并从 `--help`/compatibility.md 移除假支持声明。`SERVER` 位置参数同样处理(实现或移除,不留静默忽略)。
- `id` / `groups`:改用 `OpenProcessToken` + `GetTokenInformation(TokenGroups)` + `LookupAccountSidW`;数字 ID 用 SID 的子授权(RID)表示。`groups [USER]` 仅对当前用户实现,查询其他用户显式报错"Windows 上不支持查询其他用户的令牌"。
- `ps` `%MEM`:分母改 `GlobalMemoryStatusEx` 的真实总物理内存。

### 2.3 声明与实现对齐

对"接受但不工作"的 no-op 选项,二选一,禁止维持现状:

- 修复成本小(≤ 半天)→ 实现:如 `sort -k/-t`(最小字段语法)、`netstat -a/-n`、`patch -p`、`timeout -k`、`date -d/-s`(最小格式集)、`iostat -d/-x` 移除(P1 数据源都不在,声明收窄);
- 成本大 → 从 `cli.rs` 移除声明并同步 `compatibility.md`,如无必要不采用。

### 2.4 数据正确性

- `cat` 重写为字节管道:默认原样透传,任何选项都不丢弃 `\r`,不再做 byte→char 重编码;`-v/-E/-T` 的变换在字节层完成(非 ASCII 原样输出)。
- `base64` 解码按标准算法重写 accumulator(处理 `=` 填充与非 4 倍长度),与自身编码往返必须一致。
- `touch` 按 `SetFileTime`(FILE_WRITE_ATTRIBUTES 打开)重写,更新已存在文件的 mtime/atime;`-a/-m` 一并实现;`-d/-t` 解析留 P2。
- `bc` 重写表达式求值:词法分析(运算符/括号/一元负号)+ 优先级爬升求值;数值用 i128,除法向零截断(GNU 默认 scale=0 语义);`scale/ibase/obase`、函数、`-l` 留 P1。
- `echo` 默认关闭转义(GNU 默认 -E),仅 `-e` 开启;`\c` 等转义补全是 P1。

### 2.5 panic 与退出码

- 全部 unwrap/expect/切片/下溢改为错误处理:非数字参数报错退出 1(seq/cal);句柄/写入失败报错(shred 不在本轮但 timeout 同理);`yes` 的 BrokenPipe 静默退出 0。
- 退出码对齐 GNU 惯例:diff 0/1/2、cmp 2、file 1(处理完所有文件)、wc/df/du/sort/rm/sync/kill/pkill/pgrep/chown/ping 失败非零;pgrep 无匹配退出 1。clap 用法错误维持 2(与 GNU 的差异可接受,不作为本轮目标)。
- `mv` 对非空目录的整树删除(GNU 拒绝)是 P1,但因与 `cp` 的 `.`/`..` 修复同文件,本轮允许顺手修(P0 邻近原则)。

### 2.6 Windows API 选型(涉及命令)

| 命令 | API |
|---|---|
| touch | `CreateFileW(FILE_WRITE_ATTRIBUTES)` + `SetFileTime` |
| uptime / w | `GetTickCount64`;w 同时移除三处 expect,外部命令失败降级输出 unavailable |
| top / ps | `GlobalMemoryStatusEx` |
| id / groups | `OpenProcessToken` + `GetTokenInformation` + `LookupAccountSidW` + `ConvertSidToStringSidW` |
| chown | `SetNamedSecurityInfoW` + `GROUP_SECURITY_INFORMATION` |
| nice | `CreateProcessW` 或子进程句柄 `SetPriorityClass`(BELOW_NORMAL 等),等待并传递退出码 |
| timeout | `std::process::Child::try_wait` 轮询区分"谁先结束",退出码 124 语义正确;进程树终止用 `taskkill /T` 或 Job Object(本轮 try_wait + child.kill 即可,Job Object 留基础设施轮) |
| tty | `io::stdin().is_terminal()`(std 已有) |
| nslookup | `DnsQuery_W`(新增 windows feature)或显式不支持 |

### 2.7 路径净化

`which`、`readlink -f` 输出统一剥 `\\?\` 前缀并转 `/`(以 `realpath` 现有 `format_path` 为模板,本轮允许在 crate 内复制该小函数);`realpath --strip-components` 跳过 Prefix/RootDir 组件;`dirname /` 输出 `/`;`basename ./`、`a/..` 输出 `.`/`..`。

---

## 3. 分批结构

| 阶段 | 主题 | 命令 |
|---|---|---|
| A | 安全 | tar、unzip、zip、sync |
| B | 数据损坏与核心功能失效 | cat、base64、touch、echo、printf、bc |
| C | panic 与进程控制 | seq、cal、cp、tr、join、nice、timeout、top、uptime、w、date、patch |
| D | 伪造数据清除 | nslookup、id、groups、ps、chown、tty、vmstat |
| E | 输出与路径语义 | ls、stat、file、which、readlink、realpath、dirname、basename、grep、diff、sort、uniq、nl、tac、wc、tail、tee、split、csplit、column、pr、shuf、du |
| F | 收尾 | 杂物清理、文档同步、全量验证 |

阶段内任务互相独立;阶段间按 A→E 顺序执行(安全最先)。每任务一个 commit,message 约定 `fix(<cmd>): <一句话>`。

## 4. 测试策略

- 每个修复在 `crates/<cmd>/tests/` 新增回归测试,先写失败断言再修;使用 assert_cmd + tempfile(workspace 已有);
- 恶意归档(tar/unzip)fixture 在测试内构造(含 `../../evil.txt`、绝对路径条目),不提交二进制 fixture;
- 不联网、不杀真实进程(timeout 用 `cmd /c exit 0` 类快速子进程;kill/pkill 不在 P0 范围);
- 文本类断言使用原始字节(`stdout()` 字节比较)锁定 CRLF/二进制行为;
- 阶段 F 运行完整 `scripts/build.ps1 -Task Verify` 与 `scripts/release.ps1` 本地打包冒烟。

## 5. 风险

| 风险 | 缓解 |
|---|---|
| `echo`/`cat` 行为改动破坏既有 smoke/behavior 断言 | 修复前先跑受影响命令的现有测试,差异逐条确认后更新断言 |
| `windows` crate 新 feature(Dns)在 CI 拉取失败 | rsproxy 镜像已有缓存;失败则降级为"显式不支持"方案 |
| `bc`/`timeout`/`nice` 重写引入新缺陷 | 每个重写类任务至少 5 个行为用例(含边界),审查后再合并 |
| ls 显示名/元数据分离改动面较大 | 仅动 print_long 的条目查找逻辑,不动渲染管线其余部分 |
| 50 项修复跨 13 轮惯例外的多名命令 | 严格一任务一 commit,ledger 逐任务记录,失败任务不阻塞后续独立任务 |

## 6. 验收标准

1. 审查报告 P0 清单 50 项全部处于以下状态之一:已修复(附回归测试)/ 已实现或已移除声明(no-op 选项)/ 在 compatibility.md 显式声明为有意差异且不再产生错误数据;
2. 报告"横切共性问题"中的伪造输出类(第 4 条)在本轮范围内清零;
3. `scripts/build.ps1 -Task Verify` 通过;受影响命令的 `cargo test -p <pkg> --locked` 全绿;
4. `docs/compatibility.md` 双语与代码一致(报告"文档同步项"清单核销);
5. 仓库内无提交状态的手工测试杂物文件;
6. `.superpowers/sdd/progress.md` 逐任务记录完成状态。
