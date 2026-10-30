# Changelog

## Unreleased

### Round 18 命令补齐(9 个新命令,2026-10-29)

- 新增 9 个命令,命令总数 125 → 134(137 个包):nproc、printenv、truncate、base32、sum、tsort、pathchk、hostid、expr。
- 全部参照本机 GNU coreutils 8.32 实测语义实现并附行为测试:expr 支持 i64 四则/比较/|&/锚定 BRE 匹配/substr/index/length 与 GNU 退出码(BRE 反向引用除外);sum 实现 BSD 与 SysV 两种算法(实测向量 36979/542);base32 按 RFC 4648 含 GNU 换行语义;truncate 支持 +/- 与 1024/1000 进制后缀;tsort 按 GNU 措辞报环与奇数 token;printenv 支持 -0 与任一缺失退出 1;pathchk 检查组件/路径长度与便携字符集;hostid 输出机器名散列。
- expr 引入 regex 依赖(BRE 转换);其余命令零新增依赖;冒烟清单 134 项,双语兼容性表与 README 计数同步。
- 明确不做并记录:chgrp(复用 chown 组逻辑,单独一轮)、dd、b2sum(需 BLAKE2 依赖)。

### Round 17 质量收口（行为测试覆盖 + P2 清偿，2026-10-29）

- 为 27 个"仅冒烟"命令补齐行为测试：clear/cmp/df/expand/factor/free/hexdump/kill/mkdir/mkfifo/readlink/realpath/reset/rev/seq/shred/strings/time/uname/who/whoami/zcat/printf/sleep/echo/which/whereis；iostat、mpstat、sar、w 仍为仅冒烟并在兼容性表声明。
- cmp 对齐 GNU：differ 消息改走 stdout（char N、line M），退出码 0/1/2，EOF 消息带引号与字节/行上下文（末字节为换行时省略 in），-l 八进制清单，单操作数与 stdin 比较，-l/-s 互斥拒绝；修复行号误用字节位置与打开失败退出码。
- printf 全量重写：宽度/精度/标志（- 0 + 空格 #）、d/i/u/o/x/X（base-0 解析、%u 回绕）、f/e/E/g/G（指数两位带符号、%g 去尾零）、%b（\c 截断）、%q、跨参数格式复用、无转换指令多余参数警告、未知指令报错退出 1、\a\b\e\f\v\'\? 转义与 coreutils 八位八进制规则；缺参 %f 输出 0.000000（GNU 一致）。
- echo 改为手工前导选项扫描（弃用 clap）：未知选项簇按操作数输出（`echo -x`、`echo --`），-e 补齐 \a\b\c（截断且无换行）\e\f\v\0NNN\xHH 全集。
- sleep 支持多操作数求和与小写 s/m/h/d 后缀；拒绝大写后缀与负数（GNU 语义），移除非 GNU 的 -v。
- which 改用 PATHEXT 探测（精确名优先、标准扩展后接自定义项、大小写不敏感去重），输出改为 PATH 书写形式不解析符号链接。
- whereis 移除硬编码 D:\develop\rust-tools 搜索路径，输出对齐 GNU（`cmd: path...`/裸 `cmd:`），新增 -b/-m/-s 与命中退出码。
- 其他修复：expand 字节安全化（CRLF/非 UTF-8/未终止末行透传）与 GNU tab 诊断（0 尺寸、升序）；factor 0 输出 `0:`；rev 按 util-linux 字符语义反转并对非法 UTF-8 字节回退；strings 对齐 binutils（TAB 可打印、-t 偏移 7 宽空格填充、8 位模式 raw 输出、-f 按书写路径）；shred 失败传播退出码 1；seq 接受选项后的负数操作数（`seq -w -3 0`）。
- 兼容性表双语逐条更新（验证状态与差异说明）；维持 125 个命令。

## 0.3.2 — Windows 文件属性与 ps 结构收敛（2026-10-25）

- 收敛 ps 结构：将仅供 ps 使用的 ps-core 合入 ps-bin，保留 ps/pgrep/pkill 共用的 ps-sys；以静态列描述表及四组格式化函数替代 40 个列类型和动态工厂，清理未使用的过滤/采样框架及内部字段。42 个列名/别名、格式、排序和树形输出保持原行为；当前 128 个包生成 125 个命令。

- 全部 125 个命令在普通编译时嵌入 Windows 版本资源：功能说明、产品/文件版本、发布者、版权、内部名称及原始文件名；套件版本集中管理，ps/vdir 身份分别正确，补齐缺失包描述。新增仅构建时使用的 ruxcmd-build，当前 128 个包仍生成 125 个命令。
- Verify 与打包新增原生版本属性门禁，ZIP 解压后再次验证并在 manifest 记录版本字段；新增真实 RC 特殊字符/空格路径/缓存刷新回归及缺失/过期/错误身份拒绝测试。不生成自签证书或修改系统信任。

## 0.3.1 — 桌面交互测试改为手动执行（2026-10-25）

- 将 open 的桌面交互测试统一改为显式手动执行，避免常规测试与发布验证启动真实窗口、因临时目录删除触发资源管理器“位置不可用”弹窗。

## 0.3.0 — 命令扩展、NUL 模式与流式处理（2026-10-25）

- 修复 top 行为测试遗漏系统态 CPU、在高内核负载主机上误报的问题；断言检查各列范围与百分比总和，并以失败样本锁定回归。
- Round 16 为 15 个已有命令增加 NUL 模式：sort/uniq/cut/paste/comm/shuf/head/tail 的 -z/--zero-terminated，basename/dirname/md5sum/sha1sum/sha256sum/sha512sum/cksum 的 -z/--zero。仍为 125 个命令，无新增依赖包版本。
- ruxcmd-core 的 ByteLines 增加可选分隔符，默认 LF/CRLF 规则保留；NUL 模式原样保留记录内 CR/LF 和非法 UTF-8。head/tail 不补末记录终止符；tail follow 在收到 NUL 前即可输出部分追加内容，并显式刷新初始输出。
- 同步修复：sort 不再将不同非法字节经有损字符串混同；sort/uniq 字段按 GNU 的 space/TAB/LF 分界（CR 保留）；uniq -f 不归一化剩余空白；comm 隐藏列后压缩 TAB；paste 解码分隔符转义、重复 - 共享 stdin，串行空输入仍输出终止符；tail 多文件标题增加 LF 间隔，-v 作用于 stdin。
- 校验和 NUL 输出不转义文件名，四种摘要拒绝 -z -c（exit 1）；md5sum/sha256sum 同步 Round 15 的 stdin 清单、精确文件名/GNU 转义/tag、FAILED 输出和读取错误传播修复。
- GNU 差分增加专用 NUL/未终止/NUL 二进制/空记录形态，保持旧矩阵形态不变；摘要 tag-zero 及四种摘要清单实际比较。GNU 具体选项能力探测先要求 --help 成功，旧 cksum 8.32 不支持 --zero 时只跳过该用例；不将进程初始化失败当作不支持选项。

- Round 15 新增 14 个命令：rmdir、link、unlink、mktemp、install、sha1sum、sha512sum、cksum、users、logname、nohup、watch、traceroute、vdir；当前 128 个包生成 125 个命令。ls 增加 -h，vdir 复用 ls 库并默认启用长格式。
- 新命令补齐行为测试、125 项冒烟清单、GNU 差分矩阵、双语兼容性说明及开发文档；差分测试为修改文件的命令创建两份隔离 fixture，比较 stdout、退出码及文件内容/目录状态。SHA 检查模式实际纳入差分，MSYS 生成格式、utmp 和长列表格式差异单列豁免。
- 收尾修复：install 单操作数误报成功、缺失 -D、目标链接别名覆盖；mktemp 改用 BCryptGenRandom，并补 -t、Windows 前缀/根路径与后缀校验；SHA 清单 stdin、精确文件名/GNU 转义/BSD tag、失败输出流及读取错误传播；cksum 隐式 stdin 不再输出文件名；nohup 使用 DETACHED_PROCESS、保留重定向流并明确警告 Job 脱离限制；traceroute 不再因连续三跳静默提前终止。
- Round 15 经维护者确认将 NUL 分隔的 -z 系列交给后续 Round 16，现由上述 Round 16 实现；仍不声明完整 GNU coreutils 覆盖，也不包含 winget 上架。

- 新增 GNU 差分测试框架 scripts/difftest.ps1:对 cat/head/tail/sort/uniq/tr/cut/paste/fold/wc/nl/grep/sed/od/seq/echo/printf 等 30+ 命令的 87 个用例矩阵,生成确定性随机输入(ASCII/CRLF/非 UTF-8/空/无尾换行/长行),与 Git 自带 GNU 工具逐字节比对 stdout 与退出码;支持种子复现、单命令筛选、已知差异豁免表与缺失工具自动跳过。
- 差分测试首轮发现并修复 5 个 GNU 对齐缺陷:seq 缺结尾换行;cut 无分隔符行未按 GNU 透传;nl 未给无尾换行输入补终止换行;sed 未保留缺失尾换行(纠正 0.2.0 前的"恒补 \n"错误假设);md5sum/sha256sum 文本模式改用 GNU 两空格分隔。MSYS 二进制模式校验和输出差异登记为已知差异。
- 更新 wc/sed/nl/cut/md5sum 兼容性表条目与相关测试锁定。

- 新增 ruxcmd-core 共享 crate(字节安全读行/路径净化/CPU 采样/只读清除),which/readlink/realpath、iostat/mpstat/sar/top/vmstat、grep/cut/sort、cp/unzip 的重复实现收敛(净删约 500 行);dirname/basename/head/tail/sed/fold 因语义差异保留本地实现(已记录)。
- 流式化:curl/wget 下载与 base64 编解码改为流式(300MB 下载内存峰值 342MB→11.5MB);nl/od/hexdump/strings/iconv 逐行/分块处理(1550 次随机差分验证与旧实现输出一致)。

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
