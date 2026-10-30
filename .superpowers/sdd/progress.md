# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.


## Round 18 gap commands (2026-10-29)
- Approved: nine new commands (nproc/printenv/truncate/base32/sum/tsort/pathchk/hostid/expr), 125 -> 134 binaries, 137 packages.
- References: local Git Bash coreutils 8.32 probed live for every command (expr operator/exit semantics, sum BSD+SysV vectors, base32 vectors and wrap, tsort odd-token wording with input name, truncate sizes, nproc --ignore, printenv -0, pathchk, hostid format).
- Implementation: single-file main.rs crates with behavior tests (58 new tests); expr adds the regex dependency for BRE translation (back refs unsupported, documented); sum counts a final partial block; base32 decodes across padding groups and tolerates newlines.
- Deferred: chgrp (reuses chown group logic), dd, b2sum (needs a BLAKE2 dependency); iostat/mpstat/sar/w behavior tests; difftest additions.
- Final Verify passed: 96.4s; 134 smoke cases, all gates green (fmt required one cargo fmt pass; truncate -s needed allow_hyphen_values for negative adjustments). Merged locally to main; not pushed.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.


## Round 18 gap commands (2026-10-29)
- Approved: nine new commands (nproc/printenv/truncate/base32/sum/tsort/pathchk/hostid/expr), 125 -> 134 binaries, 137 packages.
- References: local Git Bash coreutils 8.32 probed live for every command (expr operator/exit semantics, sum BSD+SysV vectors, base32 vectors and wrap, tsort odd-token wording with input name, truncate sizes, nproc --ignore, printenv -0, pathchk, hostid format).
- Implementation: single-file main.rs crates with behavior tests (58 new tests); expr adds the regex dependency for BRE translation (back refs unsupported, documented); sum counts a final partial block; base32 decodes across padding groups and tolerates newlines.
- Deferred: chgrp (reuses chown group logic), dd, b2sum (needs a BLAKE2 dependency); iostat/mpstat/sar/w behavior tests; difftest additions.
- Final Verify passed: 96.4s; 134 smoke cases, all gates green (fmt required one cargo fmt pass; truncate -s needed allow_hyphen_values for negative adjustments). Merged locally to main; not pushed.


## Round 19 sed regex (2026-10-30)
- Scope: addresses + s command on POSIX BRE with GNU extensions (\+, \?, intervals); replacement & / -\9 / 
	
; flags g/N/p/i; back references rejected at startup (documented difference).
- Shared: BRE translator promoted to ruxcmd-core::bre (expr delegates and anchors); regex dependency lives in core now.
- Final Verify passed: 238.5s (fmt amended the stale bre unit expectation first). Merged locally to main; not pushed.
- Verification: 50 sed tests green (29 prior behavior + 15 new regex + 6 cli); sampled differential vs GNU sed 8.32 passed (groups, &, g/2, intervals, anchored addresses, escaped delimiter); non-UTF-8 passthrough kept.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.


## Round 18 gap commands (2026-10-29)
- Approved: nine new commands (nproc/printenv/truncate/base32/sum/tsort/pathchk/hostid/expr), 125 -> 134 binaries, 137 packages.
- References: local Git Bash coreutils 8.32 probed live for every command (expr operator/exit semantics, sum BSD+SysV vectors, base32 vectors and wrap, tsort odd-token wording with input name, truncate sizes, nproc --ignore, printenv -0, pathchk, hostid format).
- Implementation: single-file main.rs crates with behavior tests (58 new tests); expr adds the regex dependency for BRE translation (back refs unsupported, documented); sum counts a final partial block; base32 decodes across padding groups and tolerates newlines.
- Deferred: chgrp (reuses chown group logic), dd, b2sum (needs a BLAKE2 dependency); iostat/mpstat/sar/w behavior tests; difftest additions.
- Final Verify passed: 96.4s; 134 smoke cases, all gates green (fmt required one cargo fmt pass; truncate -s needed allow_hyphen_values for negative adjustments). Merged locally to main; not pushed.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.
# Round 7 Progress Ledger

- Task 0: complete (commits c42e997..5176afb, review clean ✅, spec compliant)
- Task 1: complete (commits 5176afb..35f7c7c, 2 commits total — initial + Important dead-code fix, final review Approved ✅)
  * Minor ledger notes (non-blocking, tracked for final review):
    - factor/Cargo.toml 中包含 description 字段（brief 示例包含但 "exact fields" 列表未列，表述歧义，不影响功能）
    - stdin 错误格式内部与其他错误略不统一（不影响功能）
- Task 2: complete (commits 35f7c7c..af184d2, 2 commits — initial + 2 Important idiom fixes, final review Approved ✅)
  * Minor notes (non-blocking):
    - base64/src/main.rs line ~64 unused import `use std::path::PathBuf` (pre-existing warning, not introduced in fix)
    - decode_write tail flush dead branch + non-standard padding edge case (non-blocking, match brief verbatim)
- Task 3: complete (commit af184d2..1e181de, single commit 2 crates 6 files, final review Approved ✅)
  * Minor notes (non-blocking):
    - expand/unexpand main.rs each unused PathBuf import warning (kept per brief)
    - Per-byte write_all in loops could be batched (perf only)
    - expand Periodic n==0 branch is unreachable defensive code
- Task 4: complete (commit 1e181de..aa478ff, single commit, final review Approved ✅)
  * Pre-flight -b fix: verified correct (before=false: line THEN sep, not duplicate sep-then-line)
  * Minor notes (non-blocking):
    - tac/src/main.rs lines 131/135: brief 示例代码内有注释 + brief 约束"不要加注释"存在轻微自相矛盾；实际代码与 brief 逐字一致，不是 implementer 错误
    - unused PathBuf import warning (kept)
- Task 5: complete (commit aa478ff..d763990, single commit 2 crates 6 files, final review Approved ✅)
  * Implementer self-fix: shuf main.rs File::create(p)? → .and_then(|mut f| f.write_all(...) 正确修复 main 返回 ()下的编译错误；经审查语义等价
  * Minor notes:
    - column/shuf main.rs 各 1 条 unused PathBuf import warning（brief 原文继承保留）
- Task 6: complete (commits d763990..2306e80, 2 commits: initial 3 crates + C1/I1/I2 post-review fixes; final re-review Approved ✅)
  * Implementer self-fixes (review-verified correct):
    - A. pr main.rs args.file (typo brief had) → args.files.first()
    - B. numfmt main.rs exit_code Cell<i32> 规避闭包借用冲突 (.set(1) / .get())
    - C. numfmt main.rs From::Auto match arm 补全
  * Post-review fixes (review-verified correct):
    - C1/I1 Critical+Important: csplit {N} 多位数 repeat: try_repeat_tail 返回 (digit_idx, repeat) 元组 + caller 按 digit_idx 切 base；修复 /END/10 两位数字 repeat 失败
    - I2 Important: numfmt/src/cli.rs `use std::path::PathBuf;` 未使用 import 已删除
  * PRE-FLIGHT FIX 确认：numfmt iec-i 模式 `matches!(to, To::IecI) { "i" } else { "" }` 追加 i 后缀已存在
  * Minor notes (全部 non-blocking):
    - M1 pr write_header 空循环死代码；M2 csplit -b suffix-format 未使用；M3 numfmt parse_numeric 未用 last 变量
    - 2 条 build warnings (csplit mut reader, numfmt mut process 无需 mut)

---

# Round 8 Progress Ledger

- Task 0: complete (脚手架 - 创建 awk/iconv/hexdump 基础结构，更新 build-all.bat)
- Task 1: complete (awk - 实现词法分析器、语法分析器、执行引擎，支持 print/printf/if/for/变量/BEGIN/END)
  * Post-implementer fix: `-v` 同时用于 --version 和 --assign 冲突，已移除 --assign 的 `-v` short flag
- Task 2: complete (iconv - 实现字符编码转换，支持 UTF-8/GBK/GB2312/UTF-16 等，-l 列出编码)
- Task 3: complete (od 增强 - 新增 -j/-N/-w 选项，增强 -A n 无地址模式)
- Task 4: complete (hexdump - 实现十六进制转储，支持 -C/-d/-o/-x/-c/-s/-n 选项)
- Task 5: complete (strings 增强 - 新增 -f 文件名前缀、-t 偏移位置输出)
- Task 6: complete (完整构建验证 - build-all.bat 成功，101 工具全部编译通过)

## Minor notes (non-blocking)
- awk/src/main.rs: 未使用的 Token 变体 Pattern/LBrace/RBrace（预留扩展用）
- awk: 正则匹配简化为子串匹配
- iconv 使用 encoding_rs crate（新增依赖）

## 冒烟测试验证
- awk --version → awk 0.1.0 ✅
- iconv --version → iconv 0.1.0 ✅
- hexdump --version → hexdump 0.1.0 ✅
- od --version → od 0.1.0 ✅
- strings --version → strings 0.1.0 ✅
- echo "Hello World" \| awk '{print $0}' → Hello World ✅
- echo "Hello World" \| hexdump -C → 正确十六进制输出 ✅
- echo "Hello" \| od -A x -t x1 → 正确十六进制地址输出 ✅
- echo "Hello World" \| strings -t x → 带偏移位置输出 ✅

---

# Round 9 Progress Ledger

- Task 0: complete (脚手架 - 创建 nslookup/wget/curl/ping/netstat 基础结构，更新 build-all.bat)
- Task 1: complete (nslookup - 实现 DNS 查询，支持 A/AAAA 记录)
- Task 2: complete (wget - 实现 HTTP 下载，支持 -O/-P/-q 选项)
- Task 3: complete (curl - 实现 HTTP 请求，支持 -X/-H/-d/-o/-s 选项)
- Task 4: complete (ping - 实现 ICMP 回显请求，支持 -c/-t/-s 选项)
- Task 5: complete (netstat - 实现网络连接查看，支持 -a/-n/-p 选项)
- Task 6: complete (完整构建验证 - build-all.bat 成功，106 工具全部编译通过)

## Minor notes (non-blocking)
- wget/curl/ping: URL/HOST 参数改为 Option 以支持 --version（修复必需参数冲突）
- ping 使用 Windows ICMP API (IcmpSendEcho)，无需管理员权限
- netstat 使用 Windows API (GetTcpTable2/GetUdpTable) 获取真实网络连接
- nslookup 使用标准库 DNS 解析，支持 A/AAAA/CNAME/MX/TXT 记录

## 冒烟测试验证
- nslookup --version → nslookup 0.1.0 ✅
- wget --version → wget 0.1.0 ✅
- curl --version → curl 0.1.0 ✅
- ping --version → ping 0.1.0 ✅
- netstat --version → netstat 0.1.0 ✅
- nslookup www.baidu.com -t A → 返回真实 IP ✅
- netstat -a → 显示真实 TCP/UDP 连接 ✅

---

# Round 11 Progress Ledger(P0 缺陷修复)

依据:2026-10-07 全量命令代码审查(docs/command-review-2026-10-07.md),50 项 P0(50 命令)。
计划:docs/superpowers/plans/2026-10-07-p0-remediation.md(48 任务,Phase A-F)。

- Task 0: complete (spec + plan 编写,2026-10-07;审查报告、spec、plan 三份文档就绪,修复未开始)
- Task 1: complete (fix(tar) 915b04e — 解包拒绝绝对路径/盘符/../.. 组件条目,恶意条目 stderr 报错跳过、退出码 2;回归测试 2 个含恶意归档 fixture)
- Task 2: complete (fix(unzip) 361cb6b — enclosed_name 防 zip slip,不安全条目警告跳过、退出码 2)
- Task 3: complete (fix(zip) 608b005 — PowerShell 单引号转义防注入,含空格/引号文件名可归档)
- Task 4: complete (fix(sync) 4fa3cc1 — PATH 解析跳过自身防自递归,传播外部退出码,rundll32 回退)
- Task 5: complete (fix(cat) fdaef0e — 字节管道重写:流式 8KB、\r 保留、Latin-1 重编码移除;-E 末行不补 $;18 个字节级测试)
- Task 6: complete (fix(base64) ea32c8f — 解码 accumulator 重写,填充/余量正确,往返验证)
- Task 7: complete (fix(touch) 4488b8d — SetFileTime 真实更新 atime/mtime,-a/-m 新增,目录/只读文件可更新)
- Task 8: complete (fix(echo) bcbe9df — 默认不解释转义(GNU -E 默认),-e/-E 语义正确)
- Task 9: complete (fix(printf) 830de12 — 格式串转义、缺参空串/0、非法数字退出 1)
- Task 10: complete (fix(bc) 06b5b9b — 递归下降求值器重写,i128 整数、^ 右结合、quit/-l 处理;21 个测试)
- Task 11: complete (fix(seq,cal) 3a93900+155cea7 — 参数校验替代 panic;cal 单参 ≥100 按年份、1-12 按月份)
- Task 12: complete (fix(cp) f133616 — ./.. 源内容并入 dest;多源目标必须为已存在目录)
- Task 13: complete (fix(tr) 15f98a7 — 缺 SET1/空 SET2 报错退出 1)
- Task 14: complete (fix(join) 3d8c99e — 字段号 0 报错退出 1)
- Task 15: complete (fix(nice) df21b9f — 去除 wmic,SetPriorityClass 五档映射,退出码透传)
- Task 16: complete (fix(timeout) b6329c0 — try_wait 轮询,超时 124,-k 实现,小数时长,不再依赖 taskkill)
- Task 17: complete (fix(top) fbed9b2 — saturating_sub 下溢,快照失败一次性警告)
- Task 18: complete (fix(uptime,w) 2dc5230 — GetTickCount64;w 三处 expect 改降级 unavailable)
- Task 19: complete (fix(date) 6718bb7 — -s 冲突解除;-d/-s 最小格式集(@epoch/ISO),SetLocalTime)
- Task 20: complete (fix(patch) 8d2651a — --strip 实现GNU 语义;-0,0 新建补丁下溢修复)
- Task 21: complete (fix(ps) 06f5573 — %MEM 分母 GlobalMemoryStatusEx;-F/-l 未注册列清理,README 同步)
- Task 22: complete (fix(id,groups) 4afb65d — Token API 真实令牌组/RID;groups 其他用户报错退出 1)
- Task 23: complete (fix(chown) a0d18fb — SetNamedSecurityInfoW,--group 正确设组,失败退出 1)
- Task 24: complete (fix(tty) 4614430 — IsTerminal,CONIN$ 占位)
- Task 25: complete (fix(vmstat) be9168f — 内存逐帧采样;-h 让位 help)
- Task 26: complete (fix(nslookup) f15fd8b — DnsQuery_W 真实 MX/TXT/CNAME;SERVER 参数移除;类型大小写不敏感)
- Task 27: complete (fix(paths) d3c2b03 — which/readlink 剥 verbatim;realpath strip 偏移;dirname / 输出 /;basename 尾组件语义)
- Task 28: complete (fix(ls) c24536f — 长格式按索引引用条目,着色下列不丢失)
- Task 29: complete (fix(stat) 95a9c92 — -c 指令 GNU 语义映射,本地时区,-f 真实块/空间行,未知指令退出 1)
- Task 30: complete (fix(file) 6d4e79b — -b 生效;zlib/gzip 区分;遇错继续处理;ASCII/UTF-8 文本检测)
- Task 31: complete (fix(grep) 1ecd6fa — 单文件 -c 输出计数)
- Task 32: complete (fix(diff) 3b7e55d — 退出码 0/1/2;普通格式行号 GNU 锚点(命令不合并为 c 为声明差异))
- Task 33: complete (fix(sort) ee358dd — -k/-t 实现;-u 排序后去重;-c -r 逆序检查;-h 1024 进制;附带修复 -h/help 冲突)
- Task 34: complete (fix(uniq) f73c4be — 输出原始行,比较键与输出分离)
- Task 35: complete (fix(nl) b12143a — UTF-8 行读取;-b t/n/a 语义,默认 t)
- Task 36: complete (fix(tac) 1f22604 — 分隔符随记录,尾分隔符无空段,无尾换行正确)
- Task 37: complete (fix(wc) cb26880 — -m 字符计数;-L 不含换行;文件失败退出 1)
- Task 38: complete (fix(tail) f3c983b — stdin/`-` 支持;-n N -f 从 EOF follow;多文件 -f 轮询+GNU 头)
- Task 39: complete (fix(tee) 1ac56f6 — 8KB 字节泵透明透传,移除 cfg(unix) libc 块)
- Task 40: complete (fix(split) cfbb183 — GNU 后缀顺序 aa..az;耗尽报错;字节级行切分;-l 0 拒绝)
- Task 41: complete (fix(csplit) d66158d — 匹配行保留;自造重复语法移除;/re/+N 报错;无匹配退出 1;--suffix-format 移除)
- Task 42: complete (fix(column) b7d494e — 默认分隔符连续空白折叠)
- Task 43: complete (fix(pr) 4958614 — chrono 本地日期页眉;-n[SEP][DIGITS] 语法;死循环删除;主控收尾编译)
- Task 44: complete (fix(shuf) fab2fe8 — random-source 按文件内容播种,可复现)
- Task 45: complete (fix(du) fe9c929 — -s 递归合计;失败退出 1)
- Task 46: complete (chore 1ed15e7+后续 — 29 个 crate 根杂物删除,.codegraph 入 .gitignore)
- Task 47: complete (docs b435acd — compatibility 双语 8 处长选项漏列补齐,CHANGELOG 完整记录)
- Task 48: complete (build.ps1 -Task Verify 全量通过:fmt/check/clippy/测试/release 编译/冒烟/行为/脚本测试;release.ps1 本地打包冒烟;一次 fmt 回归修复后重跑通过)

## Round 11 总结

- 48/48 任务完成;50 项 P0 全部处置(修复+回归测试为主;个别假声明选项移除并同步文档)。
- 提交结构:4 个基线 docs/build 提交 + 46 个修复提交(fix(chore/docs/style 分类)+ 1 个横切 cli 修复(df/du/free/pr/whoami/test 的 debug 构建 panic)。
- 行为测试覆盖:本轮新增约 30 个 crate 的 tests/ 目录,行为验证状态从 10 个命令扩展到 40+。
- 遗留 P1 清单见上方"附加发现与遗留"。

## 附加发现与遗留(非本轮范围)

- 横切修复 fb2c3e1:df/du/free/pr/whoami 的 -h 与 clap 自动 help 短名冲突(debug 构建启动即 panic)、test 的参数名冲突——超出原 50 项清单的同类缺陷,一并修复。
- csplit 降序行号(如 `csplit f -- 3 2`)写盘 panic——P1 遗留,待后续轮。
- iostat/mpstat/sar 的全零假数据列、time 的 user/sys=0、ps 的 pcpu/stat 假值、w/who 的 systeminfo/OEM 解析——P1"真实数据源"轮。
- pkill/pgrep 依赖 tasklist/wmic(taskkill 假成功、wmic 已移除)——P1,建议复用 ps-sys。
- 非 UTF-8 静默截断(grep/cut/sort)、trim_end 剥行尾空白——P1"字节安全行读取层"。
---

# Round 12 Progress Ledger(P1 缺陷修复)

计划:docs/superpowers/plans/2026-10-24-p1-remediation.md(批次 A1-I1 + 收尾 F)。

- Task 0: complete (plan 编写,2026-10-24)
- A1: complete (fix 914d55e — pkill/pgrep 去 tasklist/wmic/taskkill,复用 ps-sys 快照 + OpenProcess/TerminateProcess,校验终止结果、排除自身、pgrep 无匹配 exit 1、补 -i/-x;kill 信号校验+`kill -9/-SIGNAME PID` 连写+-l 全表+信号 0 探测+失败退出 1;ps-sys snapshot 名称源从 PSAPI GetModuleBaseNameW(机器级故障面)改为 Toolhelp32 szExeFile 单次遍历)
  * 关键发现:cargo test 会把 target/debug 前置到 PATH,测试里 spawn 裸名命令(如 cmd /c ping)会解析到我们自己的 exe(我们的 ping 不支持 -n 而立即报错退出)——测试必须用绝对路径 spawn;已写入本 ledger,新测试注意。
- A2: complete (fix c06cebb — iostat/mpstat/sar CPU 列接 GetSystemTimes 真实差值;假设备行/-d/-x/-P n 声明移除;-u 短参、HH:MM:SS 时间戳、Average 行;间隔采样 bug 修复:sleep 移到两次采样之间)
- B1: complete (fix 431e9ad — time 经 GetProcessTimes 输出真实 user/sys(GNU 0m0.003s 格式,not-found 退出 127);top 头部真实 CPU% 与 up 时长、删 load average 假行;uptime 默认输出 procps 格式)
- B2: complete (fix e434e6c — vmstat r 列归 0 声明差异、id 用 GetSystemTimes 真实 idle、死 extern 删除;free 补 -t/-s/-c/-w 与 available 列、-h Gi 后缀;df GetVolumeInformationW 真实类型 + -T、失败退出 1、Use% 向上取整、盘符根规范化)
- C1: complete (fix 5ab7de5 — w/who 用 WTSEnumerateSessionsW/WTSQuerySessionInformationW 枚举真实会话;-b/启动时间经 GetTickCount64 换算本地时间;systeminfo/query 外壳删除,降级路径保留)
- C2: complete (fix 76cf802 — %CPU 改存活期均值(procps 语义,start+GetProcessTimes);stat 真实化(Sleeping,声明差异);-C 精确匹配(.exe/大小写无关);token.rs 两处句柄泄漏修复;--headers-repeat 死选项移除;stime/排序键 P2 留待)
- D1: complete (fix a796259 — grep/cut 字节化读行(非 UTF-8 原字节透传);grep 补 -E/-F/-w/-l/-h/-H/-A/-B/-C,单文件无前缀;cut -b 原始字节、只剥 \r)
- D2: complete — sort 非UTF-8/`-`/exit 2/尾空白保留;head 字节透明+互斥+stdin 头;tail -c -f、+N 偏移、非法数值 exit 1
- E1: complete — sed 地址(N/$//sub//范围)、p/d/q 命令、位置参数脚本、-n、-i 原子替换+备份后缀、未知命令报错(修静默忽略)
- E2: complete — awk if 条件基于解析 token 求值(消灭 Debug 串 bug)、else/next、print 多参数、字段残留清理、NR/BEGIN/END 跨文件、for 解析报错、比较/真值语义修正;顺带修 == 拆分、括号 &&、Value 求值器
- F1: complete — paste \r/-d 列表循环;join GNU 输出形状+空白默认分隔;fold 字节化+-b 守恒;unexpand ≥2 空白;fmt -c 无值 flag+下溢+断词;tr 转义/字符类/-c 补集/-s SET2/反序报错
- F2: complete — mv 拒绝目录/非目录覆盖(绝不删树)+跨卷 copy+delete 回退+`.`/`..` 合并;rm -f 仅静默 ENOENT 其余报错 exit 1;ln 目标按书写形式存储(去 canonicalize 污染)+-v 改 verbose;cp -f 清只读位;open 改 ShellExecuteExW(修 -R 假失败/注入/URL & 截断,-W 经 NOCLOSEPROCESS 等待)
- G1: complete — curl 默认仅 body、-d 隐含 POST、重定向可关(-L 手动 10 跳)、-f/-I/-s 真语义;wget 文件名解析(尾/、?截断、%XX)、超时、退出码 8、-O -;ping 丢包退出码、状态消息区分、-s 上限 65500、-t 改持续 ping(Windows 语义)
- G2: complete — netstat -o 真实 PID(GetExtendedTcp/UdpTable)、-a 控制 LISTENING/未绑定条目、错误 exit 1、-n 声明;xargs -0 NUL 分隔、8000 字符命令行分批不丢参、空输入运行一次(GNU)、-p 经 CONIN$ 读确认(修 stdin 已耗尽 bug);find maxdepth 对齐 GNU(0=仅起点/1 含子项)、-type 未知值报错、glob 非法不 panic、遍历错误 exit 1、GNU 单横线长选项(-name/-type/-maxdepth...)支持、regex 无效报错
  * 关键发现:cargo test 工作目录为 crate 目录,相对路径测试用 "." 并断言已知子目录
- H1: complete — chmod 符号模式+递归防环;test POSIX 子集(数值/-z/-n/!/-r/-w/-x);env NAME=VAL+-i+-u+命令执行;yes 多操作数+BrokenPipe;true/false 忽略参数;sleep 小数;pwd UNC;hostname 失败报错
- H2: complete — md5sum/sha256sum * 标记/BOM/--status/--tag/聚合退出码/- stdin;od -t 全规格(x1/d1/o1/u1/a/c/f4/f8/z)、小端、无 ASCII 列(对齐 GNU 实测);iconv -f 严格无 BOM 嗅探、--list 40 项;numfmt -d 保留分隔符不折叠、from=auto 1K=1000/1Ki=1024(代理按 GNU 语义裁决,测试同步修正)
- I1: complete — tar -C/-O/--exclude/成员名选择;unzip 流式+只读覆盖;gunzip -c/-l/多成员;csplit 降序行号报错;patch 按计数解析/多文件分段/.rej 原始 hunk/部分写回;basename 双操作数 NAME SUFFIX

## Round 12 总结

- 17 个批次(A1-I1)全部完成,覆盖审查报告全部 P1 项(约 45 个命令)+ Round 11 台账遗留清单。
- 验证:build.ps1 -Task Verify 全绿(Formatting/Check/Clippy/Tests/Build/Smoke/Behavior/脚本测试);release.ps1 本地打包冒烟通过(dist/p1-verify-2)。
- 途中修复的验证问题:rustfmt 回归(2 次)、ps-bin/df 测试 API 与 crate 惯例不一致、timeout 测试踩 PATH 前置陷阱(系统 ping 被我们的 ping.exe 遮蔽)、uptime 旧测试断言已被 B1 输出格式取代、pkill 测试与 pgrep 的大小写语义声明对齐、xargs 空输入"运行一次"补实现。
- 决策记录:xargs -p 从 CONIN$ 读确认;numfmt 按代理核实的 GNU 语义修正测试预期(1K=1000/1Ki=1024,-d 保留分隔符);ping -t 改 Windows 持续语义;od 无 ASCII 列对齐 GNU 实测。
- 遗留 P2(见审查报告):GNU 细分退出码(125/126/127 分类)、sed 完整正则、awk 数组/内建函数、per-key 排序修饰符、hostname 原生 API 等。

---

# Round 13 Progress Ledger(发布 + 基础设施 + 流式化)

依据:2026-10-25 迭代方向决策(v0.2.0 发布 → 基础设施收敛 → 性能流式化)。

- 阶段1: v0.2.0 版本提交(f71e9b8,CHANGELOG 版本节 + README Release 入口);首次 -Publish 因与 ruxcmd-core 创建并发导致 Tests 混合状态失败,干净树重跑
- 阶段2: ruxcmd-core 共享 crate 建立(a650730,lines/paths/cpu/fsutil 四模块 13 单测);paths 迁移(which/readlink/realpath 替换;dirname 保留本地实现——缺 UNC 映射语义差异已记录;basename 无对应函数);cpu 迁移(iostat/mpstat/sar/top/vmstat 五 crate 净删 412 行,统一由 core 覆盖采样测试)
- 阶段2续: lines 迁移(grep/cut/sort 替换;head/tail/sed/fold 语义不等价保留——head 需终止符原样回写、tail 字节偏移、sed/fold 只剥 
 保 
);fsutil 迁移(cp/unzip 替换;rm 经查证 std 1.99 remove_file 已带 POSIX 删除语义无需接入,新增只读删除回归测试)
- 阶段3: 流式化完成(curl/wget 300MB 下载峰值 342→11.5MB;base64 304→4.6MB,发现并修复跨块 carry bug;nl/od/hexdump/strings(含 UTF-16/32 全编码)/iconv 流式,随机差分 1550 次新旧一致;nl 非 UTF-8 从报错改 lossy 透传为唯一行为变化)
- 发布事故记录: 第一次 -Publish 因与 ruxcmd-core 开发并发致 Tests 混合状态失败;第二次在 dirty 树上跑到打包被拒但已推送 tag v0.2.0(指向 4b995a7,该提交含 core 迁移,Verify 全绿,作为 0.2.0 锚点成立);第三次对齐 HEAD==tag 后发布成功。教训:发布运行期间禁止改动工作区。
- CI 修复(0279064/ca92741):本地 Verify 全绿但远端 CI 连续失败的两个环境差异缺陷——①env -u/赋值覆盖大小写敏感(runner 环境变量名为 Path 而本地为 PATH);②sync 自递归比较未归一化 self 路径(runner TEMP 为 8.3 短名 RUNNER~1,canonicalize 后形态不一致)。教训:Windows 语义的测试要按"变量名/路径大小写与短名形态都可能不同"设计,本地绿不等于 CI 绿。

---

# Round 14 Progress Ledger(差分测试框架)

依据:迭代方向二"质量护城河"决策(2026-10-25)。

- Task 1: complete (scripts/difftest.ps1 — 确定性 LCG 输入生成器 6 种形态(ascii/crlf/nonutf8/empty/notrail/long)x 2 变体;87 用例矩阵覆盖 30+ 命令;逐字节 stdout + 退出码比对;种子复现/-Cases 筛选/已知差异豁免表/缺失 GNU 工具自动跳过)
  * 开发教训:PowerShell 陷阱 `func % N` 中 % 被解析为函数参数而非取模(裸命令后必须加括号);-Verbose 与 CmdletBinding 内置参数冲突改名 -ShowAll;Git 新版 git.exe 路径需向上搜索 usr/bin
- Task 2: complete (首轮全量 246 过/44 失败,差异分类裁决:)
  * 修 5 类真 bug:seq 尾换行;cut 无分隔符行透传;nl 终止最后一行;sed 保留缺失尾换行(纠正 Round 12 错误假设——GNU sed 实际保留,当时误判"恒补换行");md5sum/sha256sum 两空格文本模式
  * 豁免 2 类:MSYS 二进制模式校验和格式(前缀转义 + * 分隔符,环境差异);rev 缺失(Git for Windows 不附带,自动跳过)
  * 同步更新 compatibility 双语 5 行 + 4 个锁定旧行为的测试
- Task 3: complete (最终 274 过/0 失败/13 跳过,种子 20261025)
- CI 修复追加:差分修复波次导致远端 CI 连续 3 次失败,根因链:①本地提交前漏跑 cargo fmt(seq 格式);②sha256sum supports_dash_stdin 测试仍断言单空格(本地 grep head 截断漏看失败);③verify-behavior.ps1 的 wc 行为断言未同步 stdin 裸计数变更。三个教训已固化:每轮提交前必跑 cargo fmt --all --check;测试失败统计禁止 head 截断;改输出格式的命令必须同步核查 verify-behavior.ps1 断言。
- 最终状态:CI 绿(fix(scripts) 40150144341);difftest 274 过/0 失败/13 跳过(种子 20261025)。

---

# Round 15 Progress Ledger(高频缺口命令移植)

计划:docs/superpowers/plans/2026-10-25-round15-gap-commands.md(14 个新命令,111 → 125)。

- Task 0: complete (spec/plan 编写,2026-10-25)
- 波 1: complete (4c9a1a5 — rmdir/link/unlink/mktemp/sha1sum/sha512sum/cksum，均有行为测试)
- 波 2: complete (afffc6a — install/logname/users/nohup，均有行为测试)
- 波 3: complete (87bb6a3 — watch/traceroute，均有行为测试)
- 波 4: complete (a5076db — ls -h、ls 库目标和 vdir 薄别名，均有行为测试)
- F: verified (接续集成：125 项冒烟清单、GNU 独立文件系统 fixture 和 SHA 校验差分、双语 compatibility/README/development 与 CHANGELOG；全量 Verify、差分和本地打包验收通过，整理提交与推送)
- Final review: fresh reviewer 对 Round 15 全部实现审查，发现 install 链接别名覆盖、nohup 控制台/Job 脱离、SHA stdin/文件名/读取错误、mktemp Windows 路径、traceroute 三跳早停；回归修复后相关七包测试全绿。
- Ruling: 用户明确将 -z 系列留到下一轮；本轮以已有 14 个命令为范围，不宣称完整 GNU coreutils 覆盖。
- Ruling: 沿用已有 checkout 并新建 codex/round15-completion 分支保留未提交 difftest，避免复制或丢失前序工作；不改写 main 历史。
- Ruling: nohup 使用 DETACHED_PROCESS；Job 禁止 breakaway 时保留重试但显式警告，不再静默降低保证。实际正常权限测试进程也被此类 Job 包含，强制失败会使常见使用环境不可用；代价是受限 Job 关闭仍可能终止子进程，双语文档明确说明。
- Dependency record: SHA-1 使用 sha1，SHA-512 使用既有 sha2；mktemp 与 install 复用已锁定 windows-sys 的系统 RNG/文件身份 API，无新增包版本。
- Verification notes: 首次沙箱内测试 109 过/4 失败（link 拒绝访问、traceroute 无法创建 ICMP 句柄、nohup 子进程超时）；沙箱外原样重跑全部通过。随后新增 install 单参/-D/链接保护、SHA stdin/精确文件名/转义/tag、cksum 隐式 stdin、mktemp -t/前缀/后缀、nohup 重定向流、traceroute 静默跳测试均观察到失败后修复通过；新增 SHA/CRC 读取错误注入测试通过。
- 验收：`scripts/build.ps1 -Task Verify` 完整通过（319.4s）：853 个 Rust 测试通过、0 失败；125 项发布冒烟；8 项代表性行为；17 项打包脚本测试、18 项入口脚本测试。历史 Clippy warnings 仍按既有门禁报告，不以此宣称零警告。
- GNU 差分：最终完整 release 矩阵 362 过 / 0 失败 / 31 跳过（seed 20261025，variants 2）；跳过仅为 GNU 缺失的 rev、MSYS 校验和生成格式、WTS/utmp 与 vdir 长格式差异。SHA 校验模式、文件创建/删除/替换与 mktemp 规范化名后的真实文件状态均实际对比。
- 差分框架回归：直接进程调用中 .NET ArgumentList 不强制引用无空格参数，MSYS 将 `[:lower:]` 误作文件 glob。仅固定 locale 不能解决；改为无 shell、按 Windows CRT 规则逐参强制加引号，并加入字符类、字面双引号/空格/尾反斜杠 printf 用例；tr/printf 20/20 通过后全矩阵通过。
- 本地候选 ZIP：`dist/round15-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip`，125 个 exe；DLL、manifest/checksum、解压后 125 项冒烟及 8 项行为通过，dirty=True（提交前本地候选，不发布新版本）。SHA256：e5ae7ca6429158b0ed678532e3d9d89648bdfacdc866379071f342e13a1787a5。

---

# Round 16 Progress Ledger（NUL 分隔支持）

Spec：docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md。
Plan：docs/superpowers/plans/2026-10-25-round16-zero-terminated.md。
基线 fe136eb；分支 codex/round16-zero-terminated；设计提交 d57d999。

- Task 0: complete — 设计与计划经独立 builder check；维护者明确“按推荐方案直接实现，技术细节由你决定”，15 个命令范围，inline 执行 + 最终独立审查。
- Task 1: implemented/verified — ByteLines 可选分隔符；新增 NUL 内容/连续空记录/空输入测试，首次因缺失 with_separator API 失败后通过；加入部分读取错误测试。core 15 tests + 1 doc test 通过，保留默认 LF/CRLF 规则。
- Task 2: implemented/verified — sort/cut 的 -z 与原始字节选择/比较；新用例先因缺 flag 失败后通过。sort 非 UTF-8 默认排序的旧断言锁定有损顺序，已改为 GNU C 原始字节序。
- Task 3: implemented/verified — uniq/comm/paste/shuf 的 NUL 记录；保留字节、压缩 comm 隐藏列、paste 转义/共享 stdin/串行空输入、shuf 多重集合和固定源复现；新用例先失败后通过。
- Task 4: implemented/verified — head/tail 记录计数与原始字节输出；tail 初始 NUL 输出显式 flush；有界 follow 部分追加、NUL 多文件标题和 stdin -v 测试通过。
- Task 5: implemented/verified — basename/dirname -z/--zero 输出；短/长选项、多操作数、后缀和操作数内 LF 测试先失败后通过。
- Task 6: implemented/verified — 五种校验输出 NUL；四摘要 -z -c exit1、tag/原始反斜杠输出；md5sum/sha256sum 同步 stdin 清单、精确文件名、FAILED stdout 和读取错误修正。五包新用例先失败后通过。
- Task 7: locally verified — NUL 差分/具体选项探测、双语 compatibility/CHANGELOG/脚本说明已完成；完整 Verify、GNU 差分、ZIP 均通过。交付条件为当前分支推送及远端 CI 成功，最终状态见 Actions 及交付消息。
- Ruling: 使用已有 core 扩展，不重构全局命令框架；head/tail 保留原始字节路径；不新增包版本，不合并 main 或发布版本。
- GNU evidence: paste -z -s 空 stdin 输出单 NUL；head/tail 不补末记录终止符；paste -z -s -d '\n' 插入 LF 并以 NUL 结尾；uniq 保留不同非法字节；sha1sum -z -c exit 1。本机 GNU cksum 为 8.32，无 -z。
- Final review: 独立 reviewer 完成全部生产改动审查，发现 sort 字段 LF 遗漏会造成 -u 丢记录，及 tail 多文件标题 LF/stdin -v 缺口；加测试观察失败后修正。GNU 差分另发现 uniq -f 的同类 LF 问题，已用原始字节用例复现后修复；未发现其他重要缺陷，不进行重复审查。
- Ruling: 初始设计误将字段空白限定为 space/TAB；GNU 实测及 8.32 system.h field_sep 源码证明另含 LF（不含 CR）。已修正 sort/uniq 字段规则、数值前导空白、spec/plan，避免 -u 合并不同记录；不把此真实差异列为豁免。
- Ruling: 为实现明确要求的 GNU 标题格式，同时修正 tail 默认模式遗漏的标题间 LF 和 stdin -v；旧默认模式测试改为 GNU 已证实输出。必要行为修正已列 CHANGELOG，未扩大命令范围。
- Dependency record: 新增的 core/测试依赖引用全部复用锁定版本；无新包版本。集中最终测试覆盖 15 命令及 core：190 passed / 0 failed（target/round16-focused-final.log）。

- 完整验收：scripts/build.ps1 -Task Verify 通过（167.5s）；894 Rust tests / 0 failed，125 smoke，8 behavior，17 package-script tests + 18 entrypoint-script tests。历史 warnings 按原门禁报告。
- GNU 差分最终：698 pass / 0 fail / 32 skip（seed 20261025，variants 2）；新 NUL 形态不污染旧矩阵；四摘要 tag-zero 与普通清单模式均实际对照。cksum 8.32 仅因 --zero 缺失跳过这一新增 case。
- 本地候选 ZIP：dist/round16-completion/ruxcmd-0.2.0-x86_64-pc-windows-msvc.zip，125 exe，dirty=True；DLL/manifest/hash/解压 smoke+behavior 通过。SHA256：e61ed4bf913b03c759fd981a3b804355fd5bf084b31289d9b2761197b6472996。
- 源码提交：8665836（core）、3a8e385（六文本工具 + 锁文件）、de4678a（head/tail）、08ab937（路径输出）、2f58f63（校验和）；最终文档/差分提交随后记录于 Git 历史。
- 远端状态入口：https://github.com/wangdxnum1/RuxCmd/actions?query=branch%3Acodex%2Fround16-zero-terminated；本台账固定本地可复现验收，最终 CI 链接由交付消息给出。

## Windows version resources (2026-10-25)
- Approved: 125 EXE VERSIONINFO fields and packaging validation; no self-signed certificate.
- Plan: docs/superpowers/plans/2026-10-25-windows-version-info.md; inline execution.
- Ruling: shared build-only helper and binary-specific resource linking; current checkout on feature branch to reuse existing target cache. No publication in this implementation task.
- Task 1 verified: helper 7 unit tests; real RC 3 independent-workspace cases; 125 release EXE fields correct, ps/vdir identities isolated. Initial real RC quote failure reproduced and fixed using doubled RC quotes.
- Task 2 implemented/targeted verified: missing-resource gate fails on original EXEs; all-resource and 3 negative package cases pass (4 total). Full Verify/ZIP pending.
- Independent review: 3 findings reproduced before fixes (missing entrypoint gate fixture; Debug configuration forwarding; build-metadata hyphen mistaken for prerelease). Corrected; 125 Debug resources + 3 negatives, 3 real RC cases and 19 entrypoint cases now pass. bc description corrected to integer arithmetic to match i128 implementation.
- Final Verify passed: 161.5s, 898 Rust tests / 0 failures; 125 version resources, 125 smoke, 8 behavior, 3 real RC build cases, 21 package-script cases and 19 entrypoint cases. Independent reviewer confirmed all 3 findings resolved; no production changes after this run.
- Acceptance ZIP verified: dist/windows-version-info-preview/ruxcmd-0.3.1-x86_64-pc-windows-msvc.zip; 125 EXEs, dirty=False, source 074d51f8f0952c854a550d15a6573ade0aa5e89a, all 125 manifest version records correct; DLL/hash/extraction/smoke/behavior gates passed. SHA256=4c03d60d7c8d24e77a411cd6b70988999559fd5a9d233aeabc79197946d9953d.
- All tasks complete. Final bookkeeping commit changes this ledger and plan only; the acceptance archive preserves its actual clean source commit. Suite remains 0.3.1; no signing, trust-store edits, main merge or publication performed.

## ps structure simplification (2026-10-25)
- Approved: retain shared ps-sys, internalize ps-core in ps-bin, consolidate static column metadata and remove unused APIs; preserve observable behavior.
- Baseline: ps command code 54 src files / 2377 lines, plus untouched ps-sys 10 files / 883 lines. Saved old EXE at target/ps-before-simplification/ps.exe; captured 42 column metadata/value rows and 4 characterization tests passed against the old implementation.
- Ruling: c/cp remain '?' as existing callers provide no previous sample; no filter/sort/time/tree semantics changes. Branch codex/ps-simplification builds on completed Windows resource feature; no signing/merge/publication.
- Task 1 verified: ps-core removed; all 7 original unit tests migrated after review caught the column-local %MEM regression; 4 new column contracts pass. Focused ps-bin/ps-sys/pgrep/pkill suite: 48 passed / 0 failed. Old/new Release parity: 14 cases passed with the same ps.exe basename in separate directories (CLI usage reflects argv[0]).
- Structure: ps command src 54 files / 2377 lines -> 15 files / 1381 lines, including additional contracts. Four formatter groups + static descriptor registry replace trait factories. Internal unread fields and previous-sample/filter scaffolding removed; Win32's existing S display fallback retained as a character. ps-sys/pgrep/pkill source unchanged. 128 packages / 125 binaries.
- Review caught a missing seventh original test in columns/pmem.rs; restored the exact nonzero working-set/real-memory assertion. Earlier 48-test focused result preceded this restoration. Whole-tree test-name audit and mutation check follow.
- Final Verify passed: 165.1s, 902 Rust tests / 0 failures; 125 native version resources + 125 smoke, 8 representative behaviors, 3 resource builds, 21 package-script cases and 19 entrypoint cases. All 7 original core tests retained plus 4 new contracts; focused 49 pass; old/new parity 14 pass. Independent reviewer confirmed the restored nonzero memory regression resolves the sole finding.
- User scope update: explicitly requested commit/push/merge main and release 0.3.2. This supersedes the earlier no-merge/no-publication boundary and local-preview deliverable; release script performs fresh clean-source Verify and ZIP verification. Actual publication/CI status is authoritative at https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.3.2 and GitHub Actions.


## Round 17 quality closure (2026-10-29)
- Approved: behavior tests for smoke-only commands + still-valid P2 fixes; 125 commands unchanged; feature branch codex/round17-quality-closure.
- Re-survey: 14 crates had no tests at all, 16 more only cli-parsing tests; compatibility table refreshed accordingly (iostat/mpstat/sar/w remain smoke-only).
- GNU references verified live with Git Bash coreutils 8.32: cmp differ/EOF formats and exit codes, echo operand parsing incl. `--`, sleep multi-operand and uppercase-suffix rejection, printf specifier matrix, factor 0, expand tab diagnostics, seq -w negative padding.
- Fixes: cmp (stdout messages, char wording, line tracking, exit 2, -l octal, stdin operand), printf (full formatter + reuse + %b/%q + unknown-directive error + coreutils octal rule + GNU missing-float), echo (manual option scan + full escape set + \c), sleep (multi-operand sum + suffix validation), which (PATHEXT + as-written output), whereis (hardcoded path removed + GNU format + -b/-m/-s + exit codes), expand (byte-safe + diagnostics), factor (0 prints `0:`), rev (char semantics + byte fallback), strings (binutils tab/offset/raw/filename rules + case-sensitive encodings), shred (exit code propagation), seq (negative operands).
- Coverage added: 27 commands' behavior suites (~100 assertions). Known deviation documented: mkfifo transient named pipe; hexdump keeps xxd-style default format (BSD default differs) locked by tests.
- Deferred: difftest matrix additions for the new behaviors; iostat/mpstat/sar/w behavior tests; localized OS error text (io::Error Display is locale-dependent, cross-cutting).
- Final Verify passed: 100.4s; fmt/check/clippy/tests/release build/smoke/behavior/version-resource gates green; script UTF-8 decode fix for CJK consoles added to common.ps1 (cargo metadata descriptions were decoded with the GBK console codepage). Not pushed; merged locally to main.


## Round 18 gap commands (2026-10-29)
- Approved: nine new commands (nproc/printenv/truncate/base32/sum/tsort/pathchk/hostid/expr), 125 -> 134 binaries, 137 packages.
- References: local Git Bash coreutils 8.32 probed live for every command (expr operator/exit semantics, sum BSD+SysV vectors, base32 vectors and wrap, tsort odd-token wording with input name, truncate sizes, nproc --ignore, printenv -0, pathchk, hostid format).
- Implementation: single-file main.rs crates with behavior tests (58 new tests); expr adds the regex dependency for BRE translation (back refs unsupported, documented); sum counts a final partial block; base32 decodes across padding groups and tolerates newlines.
- Deferred: chgrp (reuses chown group logic), dd, b2sum (needs a BLAKE2 dependency); iostat/mpstat/sar/w behavior tests; difftest additions.
- Final Verify passed: 96.4s; 134 smoke cases, all gates green (fmt required one cargo fmt pass; truncate -s needed allow_hyphen_values for negative adjustments). Merged locally to main; not pushed.


## Round 19 sed regex (2026-10-30)
- Scope: addresses + s command on POSIX BRE with GNU extensions (\+, \?, intervals); replacement & / -\9 / 
	
; flags g/N/p/i; back references rejected at startup (documented difference).
- Shared: BRE translator promoted to ruxcmd-core::bre (expr delegates and anchors); regex dependency lives in core now.
- Final Verify passed: 238.5s (fmt amended the stale bre unit expectation first). Merged locally to main; not pushed.
- Verification: 50 sed tests green (29 prior behavior + 15 new regex + 6 cli); sampled differential vs GNU sed 8.32 passed (groups, &, g/2, intervals, anchored addresses, escaped delimiter); non-UTF-8 passthrough kept.


## Round 20 difftest closure + chgrp/b2sum (2026-10-30)
- difftest matrix extended to 770 cases (echo -e set, printf specifiers, cmp fixtures, factor, expand lists, base32, sum, tsort, expr, sed regex); all pass.
- Alignment fixes driven by the new cases: sum -s uses per-block 16-bit one's-complement accumulation with end-around carry (probes 60108/65025/64515 match GNU), base32 -w 0 emits no trailing newline and -d rejects non-alphabet digits, tsort reports loop members on stderr and emits them on stdout (GNU wording), base32 decode across padding groups.
- Final Verify passed: 125.4s (fmt included). Merged locally to main; not pushed.
- New commands: chgrp (LookupAccountNameW + SetNamedSecurityInfoW, mirrors chown --group with -R/-v/-f) and b2sum (BLAKE2b via the blake2 crate, sha1sum-family conventions, -l 8..512). 136 commands / 139 packages; smoke list updated; GNU digest vector cross-checked.
