# P0 缺陷修复执行计划(Round 11)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 [代码审查报告](../../command-review-2026-10-07.md) P0 清单的全部 50 项缺陷(50 个命令),每项附回归测试;清除伪造输出与失败退出 0 两类系统性反模式;结束 `scripts/build.ps1 -Task Verify` 全绿。

**设计依据:** [2026-10-07-p0-remediation-design.md](../specs/2026-10-07-p0-remediation-design.md)(修复原则、API 选型、非目标)。

**全局约束:**

- 目标 `x86_64-pc-windows-msvc`,静态 CRT;Cargo 一律 `--locked`,不动 `Cargo.lock` 中已有版本,新增 `windows` crate feature 需说明;
- 每任务一个 commit:`fix(<cmd>): <一句话>`;一次提交只含该任务 crate 与其文档同步;
- 每任务先写**失败的回归测试**再修;测试用 assert_cmd + tempfile,断言原始字节;不联网、不杀真实进程;
- 错误信息用英文,输出到 stderr;退出码遵循设计文档 2.5 节;
- 修复后的行为变化同步 `docs/compatibility.md` 与 `docs/compatibility-zh.md`(任务清单中标注"文档"的项);
- 禁止批量 `#[allow]` 掩盖问题;不动本轮范围外的 P1/P2。

**任务顺序:** Phase A(安全)→ B(数据损坏)→ C(panic/进程)→ D(假数据)→ E(输出语义)→ F(收尾)。阶段内任务独立,单任务失败不阻塞其他任务,ledger 记录状态。

---

## Phase A — 安全

### Task 1: tar — 解包路径穿越

**证据:** 报告 P0 #32;`crates/tar/src/main.rs:71,77`(`entry.unpack(&path)` 未校验条目路径)。
**方案:** 解包前校验条目路径:拒绝绝对路径/盘符前缀/含 `..` 组件;或改用 `unpack_in(base)`。恶意条目打印错误、跳过该条目、结束退出 2。
**测试:** 测试内构造含 `../../evil.txt`、`C:\evil.txt` 条目的归档,断言目标目录外无文件产生、退出码 2。

- [ ] 失败回归测试(crates/tar/tests/)
- [ ] 实现路径校验
- [ ] cargo test -p tar --locked 全绿
- [ ] 提交 fix(tar): reject path traversal entries on extract

### Task 2: unzip — zip slip

**证据:** 报告 P0 #34;`crates/unzip/src/main.rs:58`(`join(file.name())`)。
**方案:** 用 zip 0.6.6 的 `enclosed_name()`,返回 None 的条目跳过并警告;顺带处理绝对路径条目。退出码同 Task 1 约定。
**测试:** 构造含 `../../evil.txt`、`/abs.txt` 条目的 zip(测试内用 zip crate 写出),断言逃逸被拒。

- [ ] 失败回归测试
- [ ] 改用 enclosed_name
- [ ] cargo test -p unzip --locked 全绿
- [ ] 提交 fix(unzip): prevent zip slip via enclosed_name

### Task 3: zip — PowerShell 命令注入/空格失败

**证据:** 报告 P0 #33;`crates/zip/src/main.rs:19-28`。
**方案:** 新增 `escape_ps_single_quote()`(单引号包裹、内部 `'` 加倍),所有路径参数经转义后拼接;单元测试覆盖含空格、`'`、`;`、`$()`、非 ASCII 的文件名。行为测试用临时目录真实调用一次(不联网)。
**测试:** 空格文件名归档成功;转义函数单测。

- [ ] 转义函数单测(先失败)
- [ ] 实现转义拼接
- [ ] 行为测试:含空格/引号文件名可归档
- [ ] cargo test -p zip --locked 全绿
- [ ] 文档:compatibility.md zip 行披露外壳实现与转义策略
- [ ] 提交 fix(zip): quote arguments before powershell invocation

### Task 4: sync — PATH 自递归与退出码

**证据:** 报告 P0 邻近项(P1 #自递归升格处理,设计 2.1);`crates/sync/src/main.rs:10,13-20`。
**方案:** 先按 PATH 搜索 `sync.exe` 候选,`fs::canonicalize` 后与 `current_exe()` 比较,命中自身则直接走 `rundll32` 回退;传播外部命令退出码,找不到外部命令时回退成功退出 0 并打印提示到 stderr。
**测试:** 测试内伪造 PATH 指向含同名 exe 的目录不可行(自身即 sync.exe),以单测覆盖"候选解析函数拒绝 current_exe";行为测试断言退出码传播逻辑(用 cmd /c exit 7 模拟外部命令)。

- [ ] 候选解析单测(先失败)
- [ ] 实现自递归防护 + 退出码传播
- [ ] cargo test -p sync --locked 全绿
- [ ] 文档:compatibility.md sync 行更新
- [ ] 提交 fix(sync): avoid self recursion and propagate exit code

---

## Phase B — 数据损坏与核心功能失效

### Task 5: cat — 字节管道重写

**证据:** 报告 P0 #1;`crates/cat/src/main.rs:130-133,99,168`。
**方案:** 移除 `\r` 丢弃与 byte→char 重编码;输入按 8KB chunk 字节流处理,`-v/-E/-T` 变换在字节层完成(原样字节直接 write_all);整文件 read_to_end 改流式(P1 项顺手完成,同函数)。先运行现有测试,因行为修正而失败的断言逐条更新并在 commit 说明。
**测试:** 非 ASCII UTF-8 文件、二进制(含 \r\n、0x80+)经无选项 cat 原样往返;`-A`/`-E` 对 CRLF 输出符合 GNU(行尾 `$`、^M 显示)。

- [ ] 更新/新增字节级回归测试(先失败)
- [ ] 重写输出管线(字节层)
- [ ] cargo test -p cat --locked 全绿;cat 现有行为测试全部通过或已更新
- [ ] 提交 fix(cat): stream bytes without CR stripping or latin1 re-encoding

### Task 6: base64 — 解码算法修复

**证据:** 报告 P0 #30;`crates/base64/src/main.rs:135-154,178-193`(`QQ==` → 0x00)。
**方案:** 重写解码 accumulator:`=` 触发按已积累位数右对齐输出;非法字符仍报错退出 1(但先 flush 已解码部分);保留 -w 换行语义。
**测试:** `QQ==`→"A"、`SGVsbG8=`→"Hello"、`QQ`→"A"(无填充容错与 GNU 一致)、编码↔解码往返(随机字节)、非法字符退出码 1。

- [ ] 失败回归测试
- [ ] 重写解码
- [ ] cargo test -p base64 --locked 全绿
- [ ] 提交 fix(base64): correct decoder padding and remainder handling

### Task 7: touch — SetFileTime 重写

**证据:** 报告 P0 #14;`crates/touch/src/main.rs:27-43`。
**方案:** `CreateFileW` 以 `FILE_WRITE_ATTRIBUTES` 打开 + `SetFileTime` 同时更新 mtime/atime 为当前时间;`-a` 只更新 atime、`-m` 只更新 mtime、`-c` 不创建;目录与只读文件可更新时间。`-d/-t/-r` 不实现(P2),`--help` 注明。
**测试:** 已存在文件 mtime 变化(metadata.modified? 比较)、`-c` 对不存在文件不创建、目录 touch 成功、不存在文件创建。

- [ ] 失败回归测试(mtime 不变 → 修复后变化)
- [ ] 按 SetFileTime 重写
- [ ] cargo test -p touch --locked 全绿
- [ ] 文档:compatibility.md touch 行更新(-a/-m 新增,-d/-t 声明未实现)
- [ ] 提交 fix(touch): update timestamps via SetFileTime

### Task 8: echo — 默认关闭转义

**证据:** 报告 P0 #2;`crates/echo/src/main.rs:10`。
**方案:** 默认 -E(不解释转义),仅 `-e` 开启;`-E` 显式关闭;`echo -x` 类未知参数因 clap 限制维持 exit 2(P2 记录)。现有测试若依赖旧行为逐条更新。
**测试:** `echo C:\new` 原样输出;`echo -e "a\nb"` 两行;`echo -E "a\nb"` 原样。

- [ ] 失败回归测试
- [ ] 修条件逻辑
- [ ] cargo test -p echo --locked 全绿
- [ ] 提交 fix(echo): disable escape interpretation by default

### Task 9: printf — 格式串转义与参数语义

**证据:** 报告 P0 #3;`crates/printf/src/main.rs:86-88,23,31-33`。
**方案:** 格式串先做反斜杠转义(\n \r \t \\ \" \0NNN \xHH,与 echo -e 同一套表);参数缺失时 %s→空串、%d/%x/%f→0;非法数字参数打印错误退出 1。宽度/精度/循环复用不做(P1)。
**测试:** `printf "a\nb"` 两行;`printf "%s-%d" x`(缺参)→ `x-0`;`printf %d abc` 退出 1。

- [ ] 失败回归测试
- [ ] 实现转义与缺参语义
- [ ] cargo test -p printf --locked 全绿
- [ ] 提交 fix(printf): interpret format escapes and handle missing args

### Task 10: bc — 表达式求值器重写

**证据:** 报告 P0 #31;`crates/bc/src/main.rs:62,81-134`(`1+2` → 0)。
**方案:** 词法分析(数字/标识符/`+ - * / % ^ ( )`/赋值/一元负号/换行)+ 优先级爬升求值;数值 i128,`/` 与 `%` 向零截断(GNU 默认 scale=0);变量赋值静默(不打印值);除以零报错退出 1;`quit` 两种模式都识别;移除伪装 banner 的 `bc 1.07.1` 字样(改为 `bc 0.1.0 (RuxCmd)`)。scale/ibase/obase/函数/-l 不实现,遇到时打印 "not supported in this build" 到 stderr 并忽略该行(不静默)。
**测试:** `1+2`→3、`2+3*4`→14、`(1+2)*3`→9、`10/3`→3、`-5+3`→-2、`a=5; a*2`→10、`1/0` 报错、乘方 `2^10`→1024。

- [ ] 失败回归测试
- [ ] 重写 lexer/求值器
- [ ] cargo test -p bc --locked 全绿
- [ ] 提交 fix(bc): rewrite expression evaluator with correct tokenization

---

## Phase C — panic 与进程控制

### Task 11: seq + cal — 参数 panic

**证据:** 报告 P0 #4(seq,main.rs:20/24/30)、#44(cal,main.rs:25 unwrap 链)。
**方案:** seq:非数字参数报错 `seq: invalid floating point argument: '<x>'` 退出 1。cal:month 校验 1-12(越界报错退出 1);单参数 >12 按年份解释、1-12 按月份;年份 1-9999 之外报错;unwrap 链全部替换。
**测试:** `seq abc` exit 1 且无 panic 字样;`cal 2024` 整年不 panic;`cal 13`/`cal 0` exit 1;`cal 7 2024` 正常。

- [ ] 失败回归测试(两个 crate)
- [ ] 分别修复
- [ ] cargo test -p seq -p cal --locked 全绿
- [ ] 提交 fix(seq,cal): validate arguments instead of panicking

### Task 12: cp — `.`/`..` panic 与多源覆盖

**证据:** 报告 P0 #13;`crates/cp/src/main.rs:104,47-51`。
**方案:** 源路径无 Normal 尾组件(`.`/`..` 结尾)时:dest 为已存在目录 → 复制源目录内容到 dest 下(与 `cp -r src/. dest` 语义一致,写入 compatibility.md);dest 不存在 → 报错退出 1。多源复制时 dest 必须是已存在目录,否则报错 `target '<x>' is not a directory` 退出 1。
**测试:** `cp -r ./. dest`(dest 已存在)内容并入;`cp a b c`(c 为文件)exit 1;`cp -r .. dest` 不 panic。

- [ ] 失败回归测试
- [ ] 实现组件处理与多源校验
- [ ] cargo test -p cp --locked 全绿
- [ ] 文档:compatibility.md cp 行声明 `/.` 语义与多源规则
- [ ] 提交 fix(cp): handle dot sources and validate multi-source target

### Task 13: tr — 空集合 panic

**证据:** 报告 P0 #19;`crates/tr/src/main.rs:112,71`。
**方案:** 缺 SET1 → 打印 usage 错误退出 1(不 expect);SET2 为空串且需翻译时报错退出 1;`-d` 模式允许 SET2 缺省。`-c`/`-s` 语义(P1)不动。
**测试:** `tr` 无参 exit 1;`tr 'a' ''` exit 1;`tr -d 'a'` 正常。

- [ ] 失败回归测试
- [ ] 修参数校验与下溢
- [ ] cargo test -p tr --locked 全绿
- [ ] 提交 fix(tr): reject missing or empty sets instead of panicking

### Task 14: join — 字段号 0 下溢

**证据:** 报告 P0 #52(报告 #41 邻近,`-1 0` 下溢);`crates/join/src/main.rs:11-12`。
**方案:** 字段号解析为 0 时报错 `join: invalid field number '0'` 退出 1(GNU 从 1 计数)。输出形状(file2 键重复)是 P1 不动。
**测试:** `join -1 0 -2 1 a b` exit 1。

- [ ] 失败回归测试
- [ ] 修下溢
- [ ] cargo test -p join --locked 全绿
- [ ] 提交 fix(join): reject field number 0

### Task 15: nice — SetPriorityClass 重写

**证据:** 报告 P0 #36;`crates/nice/src/main.rs:16-49`。
**方案:** 去掉 wmic 路径:直接 `Command::new(program).args(...)` spawn,拿到子进程句柄后 `SetPriorityClass`(adjustment ≤-15→IDLE_PRIORITY_CLASS,≤-10→BELOW_NORMAL,0→NORMAL,≥10→HIGH,≥15→REALTIME;负调整向下、正调整向上,映射方向与 nice 一致:更"nice"=更低优先级);`child.wait()` 透传退出码;`wait()` 错误报错退出 1 而非 unwrap。
**测试:** `nice -n 5 cmd /c exit 3` 退出码 3(证明等待+透传);`nice cmd /c exit 0` 正常;参数不再丢失(`nice -n 5 cmd /c echo hi` 输出 hi)。

- [ ] 失败回归测试(证明参数丢失/退出码不透传)
- [ ] 重写
- [ ] cargo test -p nice --locked 全绿
- [ ] 文档:compatibility.md nice 行更新(优先级映射表)
- [ ] 提交 fix(nice): spawn with real priority class and propagate exit code

### Task 16: timeout — try_wait 轮询重写

**证据:** 报告 P0 #37;`crates/timeout/src/main.rs:32-46,54-63`。
**方案:** 主线程循环 `try_wait()`(50ms 间隔):子进程先结束 → 透传退出码;超时先到 → `child.kill()` + `exit(124)`;`-k` 实现:kill 后再等 kill_after 秒,仍存活再 kill 并按 124;时长解析支持 s/m/h 后缀与小数(如 1.5s),解析失败报错退出 1(不再归 0);移除 taskkill 依赖,`child.kill()` 终止直接子进程(进程树留 Job Object 轮)。
**测试:** `timeout 5s cmd /c exit 7` → 7 且立即返回;`timeout 1s ping -n 30 127.0.0.1`(长进程)→ 124 且约 1 秒返回;`timeout 1x cmd` exit 1;`timeout 1.5s cmd /c exit 0` 正常。

- [ ] 失败回归测试(证明提前完成仍阻塞/124 失效)
- [ ] 重写
- [ ] cargo test -p timeout --locked 全绿
- [ ] 文档:compatibility.md timeout 行更新
- [ ] 提交 fix(timeout): poll try_wait and return 124 on expiry

### Task 17: top — 下溢

**证据:** 报告 P0 #38;`crates/top/src/main.rs:78`。
**方案:** `processes.saturating_sub(1)`;快照失败时打印 stderr 警告。CPU 假数据(P1)不动。
**测试:** 快照路径难以注入,以单测覆盖计数函数(提取为可测函数,0 进程时结果为 0 不回绕)。

- [ ] 提取+失败单测
- [ ] saturating_sub
- [ ] cargo test -p top --locked 全绿
- [ ] 提交 fix(top): avoid underflow when snapshot is empty

### Task 18: uptime + w — GetTickCount64 与优雅降级

**证据:** 报告 P0 #39(uptime,main.rs:32-38)、#40(w,main.rs:10,28,46)。
**方案:** uptime Windows 分支改 `GetTickCount64()` 毫秒;-p 格式不变。w:三处 expect 改为外部命令失败时打印 `w: <source> unavailable` 到 stderr 并继续(空值输出),不再 panic。
**测试:** uptime -p 输出包含天/小时量级(开机时长 < 1000 天断言);w 在 PATH 缺失 query.exe 的环境下(测试内伪造空 PATH)不 panic 退出 0。

- [ ] 失败回归测试
- [ ] 分别修复
- [ ] cargo test -p uptime -p w --locked 全绿
- [ ] 提交 fix(uptime,w): use GetTickCount64 and degrade instead of panicking

### Task 19: date — -s 冲突与 -d/-s 最小实现

**证据:** 报告 P0 #43;`crates/date/src/cli.rs:12,16,9,13`。
**方案:** 去掉 `--seconds` 的 `-s` 短名(保留长名);实现 `-d/--date` 最小解析:`@<epoch秒>`、ISO 8601(`YYYY-MM-DD[THH:MM[:SS]]`)、空串=now,其他格式报错退出 1;实现 `-s/--set`:同 -d 解析后 `SetLocalTime`(失败报错退出 1);`date_string` 位置参数与 -d 同义。compatibility.md 同步。
**测试:** `date -d @0 -u` → 1970-01-01;`date -d 2026-10-07` 输出该日期;`date -d garbage` exit 1;`date -s` 与 `--seconds` 不再冲突(`date --help` 在 debug 构建不 panic — 由测试进程本身不 panic 隐式覆盖)。

- [ ] 失败回归测试(含 debug 构建 help 冒烟)
- [ ] 实现
- [ ] cargo test -p date --locked 全绿
- [ ] 文档:compatibility.md date 行更新
- [ ] 提交 fix(date): resolve -s clash and implement minimal -d/-s

### Task 20: patch — -p 实现与下溢

**证据:** 报告 P0 #50;`crates/patch/src/cli.rs:16-19,main.rs:208-218,223`。
**方案:** 实现 `--strip=N`:从 `+++`/`---` 路径剥离 N 个前导组件(含盘符与根);N 缺省 1;`-p` 短名映射同义。`hunk.old_start - 1` 改 `saturating_sub(1)`(新建文件补丁 `@@ -0,0` 起始行为 0)。hunk 解析计数/多文件/.rej 内容(P1)不动。
**测试:** git 风格补丁(`b/file.txt`)+ `-p1` 可应用;`-p0` 保留前缀找不到文件报错;新建文件补丁(`@@ -0,0 +1,2 @@`)可应用不 panic。

- [ ] 失败回归测试
- [ ] 实现 strip + 下溢修复
- [ ] cargo test -p patch --locked 全绿
- [ ] 提交 fix(patch): implement --strip and fix new-file hunk underflow

---

## Phase D — 伪造数据清除

### Task 21: ps — %MEM 真实总内存

**证据:** 报告 P0 #35;`crates/ps-core/src/columns/pmem.rs:20`。
**方案:** 启动时 `GlobalMemoryStatusEx` 取 `ullTotalPhys` 缓存,pmem 列用它作分母;取不到时报错退出 1。顺带移除 `docs/ps/README*.md` 与注册表中不一致的 sz/psr/wchan/cputime 列声明(README 侧改为与 columns/mod.rs 一致,P1 列实现留后续)。
**测试:** ps -o pid,pmem 输出的 pmem 值 ≤ 100;ps -F 无 "unknown column" 警告(stderr 为空)。

- [ ] 失败回归测试
- [ ] 接入 GlobalMemoryStatusEx
- [ ] 同步 docs/ps/README.md 与 README-zh.md 列清单
- [ ] cargo test -p ps-bin --locked 全绿
- [ ] 提交 fix(ps): compute %MEM from real total physical memory

### Task 22: id + groups — Token API 重写

**证据:** 报告 P0 #41/#42;`crates/id/src/main.rs:30-47,79-116`、`crates/groups/src/main.rs:15,61-75`。
**方案:** id:`OpenProcessToken` + `GetTokenInformation(TokenUser/TokenPrimaryGroup/TokenGroups)` + `LookupAccountSidW`;`uid/gid` 输出 SID 子授权(RID)数字,`-n` 输出账户名,`groups` 列输出 token 真实组;`-u/-g/-G` 按令牌数据。groups:当前用户走同一 token 路径;带 `[USER]` 参数时若非当前用户,报错 `groups: cannot query other users on windows` 退出 1(设计 2.2);删除 env 拼凑回退。删除 id 的两处死分支。
**测试:** `id` 输出含数字 uid/gid 与 groups 且格式正确(正则断言);`id -u` 纯数字;`groups` 输出包含当前用户所属组(至少 Authenticated Users 真实存在项);`groups someone-else` exit 1。

- [ ] 失败回归测试(证明输出为编造/USER 被忽略)
- [ ] 重写两 crate
- [ ] 文档:compatibility.md id/groups 行更新(RID 语义声明)
- [ ] cargo test -p id -p groups --locked 全绿
- [ ] 提交 fix(id,groups): derive identity from process token

### Task 23: chown — --group 语义

**证据:** 报告 P0 #48;`crates/chown/src/main.rs:22,129,159`。
**方案:** `--group` 改走 `SetNamedSecurityInfoW(GROUP_SECURITY_INFORMATION)`(SetNamedSecurityInfoW 同时替代老的 Get/SetFileSecurityW 链,owner 分支一并迁移);`--user` 与 `--group` 可同时给;任一文件失败退出 1(顺带修 P1 退出码,同函数)。
**测试:** 临时文件 `chown --group Administrators f` 后 GetSecurityDescriptorGroup 校验(owner 不变);失败路径退出码 1。

- [ ] 失败回归测试
- [ ] 迁移到 SetNamedSecurityInfoW
- [ ] cargo test -p chown --locked 全绿
- [ ] 提交 fix(chown): set group via GROUP_SECURITY_INFORMATION

### Task 24: tty — is_terminal 重写

**证据:** 报告 P0 #49;`crates/tty/src/main.rs:37-47`。
**方案:** 判断依据改 `io::stdin().is_terminal()`;是 tty 打印 `CONIN$`(占位,compatibility.md 声明)退出 0,否则 `not a tty` 退出 1;`-a` 检查三个流,任一是 tty 退出 0,全部否则 1;`-s` 静默。
**测试:** 非 tty 管道环境 `tty` exit 1;`tty -a` 在测试环境(stdout 是管道)exit 1;`tty -s` 无输出。

- [ ] 失败回归测试
- [ ] 重写
- [ ] cargo test -p tty --locked 全绿
- [ ] 文档:compatibility.md tty 行更新
- [ ] 提交 fix(tty): detect console via IsTerminal

### Task 25: vmstat — 采样循环重采样

**证据:** 报告 P0 #46;`crates/vmstat/src/main.rs:150`。
**方案:** `get_memory_info()` 移入循环体内逐帧采样;swpd/free 列逐帧变化;`r` 列(P1 语义)本轮改为 0 并在表头注释?——不:保持现值但加 stderr 一次性说明属于 P1;本轮只修重采样。id 列(P2)不动。
**测试:** `vmstat 1 2` 输出两行数据行(解析列数=17),通过注入不同内存快照难以实现——以输出结构断言 + 手动验收记录到 ledger。

- [ ] 结构回归测试(两帧输出、列数一致)
- [ ] 移动采样调用
- [ ] cargo test -p vmstat --locked 全绿
- [ ] 提交 fix(vmstat): sample memory every frame

### Task 26: nslookup — 真实 DNS 或显式不支持

**证据:** 报告 P0 #47;`crates/nslookup/src/main.rs:74-97`;SERVER 参数 `cli.rs:20-21` 静默忽略。
**方案:** 方案一(优先):给 `windows` crate 加 `Win32_NetworkManagement_Dns` feature,`DnsQuery_W` 实现 MX/TXT/CNAME 真实查询(DNS_TYPE=15/16/5),失败报 DNS 错误退出 1。方案一失控时降级方案二:MX/TXT/CNAME 打印 `nslookup: record type '<t>' is not supported` 退出 1,`--help` 与 compatibility.md 收窄声明。SERVER 参数两方案都实现不了则从 cli.rs 移除并同步文档(禁静默忽略)。A/AAAA 保持 getaddrinfo。类型名大小写不敏感化。
**测试:** `nslookup example.com -t MX`(联网测试——**替代**:方案一用 mock 不可行,采用"显式错误路径"测试:`-t NS` 等未支持类型 exit 1 且输出不含伪造记录;MX/TXT 真实查询在 ledger 记录手动验收)。

- [ ] 失败回归测试(未支持类型不输出假数据)
- [ ] 实现 DnsQuery_W(或降级方案)
- [ ] SERVER 参数实现或移除
- [ ] cargo test -p nslookup --locked 全绿
- [ ] 文档:compatibility.md nslookup 行更新
- [ ] 提交 fix(nslookup): stop fabricating MX/TXT/CNAME answers

---

## Phase E — 输出与路径语义

### Task 27: which + readlink + realpath + dirname + basename — 路径净化

**证据:** 报告 P0 #8/#9/#10/#11/#12。
**方案:** 各 crate 引入相同的 `strip_verbatim()`(剥 `\\?\`、`\\?\UNC\` → `//`、反斜杠转 `/`;从 realpath 的 format_path 复制,允许重复,基础设施轮收敛):which 输出与 readlink -f 输出过该函数;realpath --strip-components 跳过 Prefix/RootDir 后再剥 N;dirname `/` 输出 `/`、根分支过 format_path;basename 尾 `./`/`..`/`a/..` 输出 `.`/`..`、`//` → `/`。
**测试:** 各 crate 一个净化断言(readlink -f 对临时 symlink;dirname / ;basename ./ 等)。

- [ ] 失败回归测试(5 crate)
- [ ] 实现
- [ ] cargo test -p which -p readlink -p realpath -p dirname -p basename --locked 全绿
- [ ] 提交 fix(paths): normalize verbatim paths across path tools

### Task 28: ls — 长格式条目反查

**证据:** 报告 P0 #5;`crates/ls/src/display.rs:322-344,379-385`。
**方案:** print_long 改为按索引直接引用条目(排序后的 Vec 下标),删除按显示名 position 反查与 extract_name_from_longfield;色码/引号/-F 只影响渲染不影响查找。
**测试:** 在启用 color 的环境断言 `ls -li` 输出含 inode 列(非 `?` 缺列);用 `ls -li --color=always` 回归。

- [ ] 失败回归测试
- [ ] 改索引引用
- [ ] cargo test -p ls --locked 全绿
- [ ] 提交 fix(ls): look up entries by index in long format

### Task 29: stat — 指令语义与本地时间

**证据:** 报告 P0 #6;`crates/stat/src/main.rs:124-135,157-171`。
**方案:** `-c` 指令对齐 GNU 语义:`%t` 设备号(Windows 输出 0 并文档声明)、`%b` 512B 块数(ceil(size/512))、`%m` 挂载点(卷根路径);`%y/%Y/%x/%X/%z/%Z` 按 Windows 可得字段映射;时间格式化改 chrono Local;未识别指令报错退出 1(GNU 行为)。
**测试:** `stat -c %b` 对已知大小文件;`stat -c %y` 与文件 mtime 一致且为本地时区;未知指令 `%q` exit 1。

- [ ] 失败回归测试
- [ ] 实现映射
- [ ] 文档:compatibility.md stat 行更新
- [ ] cargo test -p stat --locked 全绿
- [ ] 提交 fix(stat): align -c directives with GNU semantics

### Task 30: file — -b、zlib、遇错继续

**证据:** 报告 P0 #7;`crates/file/src/cli.rs:16,main.rs:50-69,431-439`。
**方案:** `-b/--brief` 生效(省略文件名前缀);zlib 魔数 `78 01/9C/DA` 改标 `zlib compressed data`;单个文件打不开打印错误继续处理其余,结束时如有失败退出 1;WebP 规则修正为偏移 8(P2 顺手,同函数)。
**测试:** `file -b x.txt` 无前缀;gzip 文件(`1F 8B`)标 gzip、zlib 头标 zlib;不存在文件 + 正常文件混合时全部处理且 exit 1;纯 ASCII 文本不再报 "data"(最小文本检测:可打印+换行 → "ASCII text",设计允许,顺手修 P1 最小集)。

- [ ] 失败回归测试
- [ ] 实现
- [ ] cargo test -p file --locked 全绿
- [ ] 提交 fix(file): honor -b, fix zlib magic, keep going on errors

### Task 31: grep — 单文件 -c

**证据:** 报告 P0 #15;`crates/grep/src/main.rs:59`。
**方案:** 计数打印不再要求 multiple_files;单文件 -c 只输出数字。多余前缀(P1)不动。
**测试:** `grep -c pattern single-file` 输出数字。

- [ ] 失败回归测试
- [ ] 修条件
- [ ] cargo test -p grep --locked 全绿
- [ ] 提交 fix(grep): print count for single-file input

### Task 32: diff — 退出码与行号

**证据:** 报告 P0 #16;`crates/diff/src/main.rs:58-62,140-163`。
**方案:** 主函数维护 found_diff 标志,有差异退出 1、读错误退出 2;普通格式打印重写:沿 LCS 结果遍历,公共行推进 i/j,d/a/c 按当前 i/j 生成行号(与 GNU hunk 语义一致);统一格式 hunk 头/上下文截断(P1)不动。
**测试:** 相同文件 exit 0;`a,b,x` vs `a,b,y` 输出 `3d2` 与 `2a3`(GNU 对照);差异文件 exit 1;不存在文件 exit 2。

- [ ] 失败回归测试
- [ ] 重写打印循环
- [ ] cargo test -p diff --locked 全绿
- [ ] 提交 fix(diff): correct exit codes and line numbers

### Task 33: sort — -k/-t/-u/-c 修复

**证据:** 报告 P0 #17;`crates/sort/src/cli.rs:33-38,main.rs:35-43,67-75`。
**方案:** 实现 `-t <char>` 分隔与 `-k <n>[,<m>]` 最小字段语法(字段取 n 到 m,缺 m 到行尾;`-n` 时按字段前导数字);`-u` 改排序后相邻去重;`-c -r` 分支修正(reverse 检查逆序);`-h` 进制改 1024(P2 顺手)。
**测试:** `sort -t, -k2 -n` 数值按第 2 字段;`sort -u` 去重;`sort -c -r` 逆序有序通过;`sort -k2` 字典序。

- [ ] 失败回归测试
- [ ] 实现
- [ ] 文档:compatibility.md sort 行更新
- [ ] cargo test -p sort --locked 全绿
- [ ] 提交 fix(sort): implement -k/-t and fix -u/-c ordering

### Task 34: uniq — 输出原始行

**证据:** 报告 P0 #18;`crates/uniq/src/main.rs:44-47`。
**方案:** prev 存原始行,print_line 输出原始行;比较仍用 make_key;`-c` 计数前缀格式保持 `%7ld `。
**测试:** `uniq -i` 输出保持原大小写;`uniq -f 1` 输出完整行。

- [ ] 失败回归测试
- [ ] 修存储
- [ ] cargo test -p uniq --locked 全绿
- [ ] 提交 fix(uniq): print original lines instead of keys

### Task 35: nl — UTF-8 行处理

**证据:** 报告 P0 #20;`crates/nl/src/main.rs:116`。
**方案:** 输入按 UTF-8 行读取(BufRead::lines),编号逻辑不变;`-b n` 语义修正:支持 `t`(非空,GNU 默认)、`n`(不编号)、`a`(全部),默认 t; compatibility.md 同步。`-n/-v/-i`(P1)不做。
**测试:** 非 ASCII 行编号输出不乱码;`nl` 默认空行不编号;`nl -b a` 空行编号。

- [ ] 失败回归测试
- [ ] 重写读取与编号
- [ ] 文档:compatibility.md nl 行更新
- [ ] cargo test -p nl --locked 全绿
- [ ] 提交 fix(nl): read utf-8 lines and correct -b styles

### Task 36: tac — 记录语义

**证据:** 报告 P0 #21;`crates/tac/src/main.rs:105-116`。
**方案:** 重写切分:按 `\n`(或 -s 分隔符)切记录,**分隔符跟随记录**(默认后置,-b 前置);尾分隔符不产生空记录;无尾换行输入最后一段不带分隔符输出。
**测试:** `a\nb\n` → `b\na\n`;`a\nb` → `ba\n`?——GNU tac 无尾换行 `a\nb` 输出 `b a`(b 无换行,紧跟 a\n):断言按 GNU 实际输出 `b\na` 写(执行时以 GNU 行为为准校准,测试内注释说明)。

- [ ] 失败回归测试(与 GNU 对照)
- [ ] 重写切分
- [ ] cargo test -p tac --locked 全绿
- [ ] 提交 fix(tac): carry separator with records

### Task 37: wc — -m 与 -L

**证据:** 报告 P0 #22;`crates/wc/src/main.rs:117-118,119,123`。
**方案:** `-m` 按 chars 计数(读入后 chars(),或按 UTF-8 解码计数);`-L` 行长不含 `\n`;读错误不再静默 break(报错退出 1,顺带修 P1 退出码)。
**测试:** UTF-8 中文文件 `-m` 等于字符数、`-c` 等于字节数;`-L` 不含换行;打不开文件 exit 1。

- [ ] 失败回归测试
- [ ] 修计数
- [ ] cargo test -p wc --locked 全绿
- [ ] 提交 fix(wc): count chars for -m and exclude newline from -L

### Task 38: tail — stdin、-n 0 -f、多文件 -f

**证据:** 报告 P0 #23;`crates/tail/src/main.rs:12-15,74-78,30-38`。
**方案:** 支持无文件/stdin(`-` 符号);`-n 0 -f` 先 seek EOF 再 follow;多文件 -f 改为统一轮询循环:每轮对每个文件读取新增字节并输出(带 `==> name <==` 分隔按 GNU 轮转输出)。
**测试:** `echo x | tail` 输出 x;`tail -n 0 -f`(临时文件先有内容)不输出旧内容(测试用短超时子进程 + 写入触发,或拆分为可测函数断言初始偏移);多文件非 -f 已有;多文件 -f 以可测函数断言"每个文件都注册轮询"。

- [ ] 失败回归测试
- [ ] 实现
- [ ] cargo test -p tail --locked 全绿
- [ ] 提交 fix(tail): support stdin and multi-file follow

### Task 39: tee — 字节透传

**证据:** 报告 P0 #24;`crates/tee/src/main.rs:48-67`。
**方案:** 读 8KB chunk,逐 chunk write_all 到 stdout 与全部文件;读错误报错退出 1;移除 cfg(unix) libc 块与 no-op -i(保留参数,help 注明 Windows 上无操作——已声明差异);删除 crate 根 test_tee.txt。
**测试:** 二进制数据(含 0x00/0x80/裸 \r)经 tee 到文件与 stdout 字节一致;CRLF 文件不被改写。

- [ ] 失败回归测试
- [ ] 重写
- [ ] cargo test -p tee --locked 全绿
- [ ] 提交 fix(tee): pass bytes through unchanged

### Task 40: split — 后缀序列与字节模式

**证据:** 报告 P0 #25;`crates/split/src/main.rs:96-109,166-180`。
**方案:** 后缀生成改为 GNU 顺序(第二位先进:aa,ab..az,ba..;-d 数字不变);后缀超宽报 `output file suffixes exhausted` 退出 1;行模式改字节级按行边界切分(识别 `\n`,保留 `\r`),非 UTF-8 安全;`-l 0` 报错退出 1。
**测试:** 27 个文件时前 27 个后缀为 aa..az,ba;二进制内容按行切分字节守恒(总字节=输入字节);`-l 0` exit 1。

- [ ] 失败回归测试
- [ ] 实现
- [ ] cargo test -p split --locked 全绿
- [ ] 提交 fix(split): gnu suffix order and byte-exact line splitting

### Task 41: csplit — 丢行与假偏移

**证据:** 报告 P0 #26;`crates/csplit/src/main.rs:40-56,150-162`。
**方案:** 重复路径保留匹配行在当前输出文件;移除自造 `模式N` 重复语法解析(该语法非 GNU,兼容性表未声明),`/re/+N|-N` 显式报 `invalid pattern` 退出 1(不再静默);`--suffix-format` 从 cli.rs 移除(假声明)。
**测试:** `/pat/{...}`(重复)输出文件包含匹配行;`/pat/+3` exit 1 带错误;`--suffix-format` 报未知参数。

- [ ] 失败回归测试
- [ ] 实现
- [ ] 文档:compatibility.md csplit 行更新
- [ ] cargo test -p csplit --locked 全绿
- [ ] 提交 fix(csplit): keep matched lines and reject bogus repeat syntax

### Task 42: column — 空白折叠

**证据:** 报告 P0 #27;`crates/column/src/main.rs:121-139`。
**方案:** 默认(未给 -s)按连续空白 run 切分(单列);显式 -s 按单字符切分不折叠;其余不动。
**测试:** `a  b`(双空格)两列;`column -t -s,` 对 `a,,b` 三列(空列)。

- [ ] 失败回归测试
- [ ] 实现
- [ ] cargo test -p column --locked 全绿
- [ ] 提交 fix(column): collapse whitespace runs in default mode

### Task 43: pr — 日期与 -n 解析

**证据:** 报告 P0 #28;`crates/pr/src/main.rs:127-136,138-163,cli.rs:33`。
**方案:** 页眉日期用 chrono 本地时间(`%Y-%m-%d %H:%M`);`-n` 改 `num_args(0..=1)`:无值为 TAB+宽 5;带值按 GNU `-n[SEP][DIGITS]` 解析(首字符非数字为 SEP,其余为宽度);删除死代码空循环。
**测试:** 页眉含当年日期;裸 `pr file` 不报错;`pr -n:5 file` 冒号分隔宽 5。

- [ ] 失败回归测试
- [ ] 实现
- [ ] cargo test -p pr --locked 全绿
- [ ] 提交 fix(pr): real header dates and gnu -n syntax

### Task 44: shuf — --random-source

**证据:** 报告 P0 #29;`crates/shuf/src/main.rs:16-17,116-123`。
**方案:** `--random-source=<path>` 从该文件读取最多 64KB 字节做种子(FNV over content);文件打不开报错退出 1;不再哈希参数字符串本身。同一路径多次调用结果一致(GNU 可复现语义的近似,compatibility.md 声明)。
**测试:** 两次同 source 调用输出一致;不同 source 不同;源文件不存在 exit 1。

- [ ] 失败回归测试
- [ ] 实现
- [ ] 文档:compatibility.md shuf 行更新
- [ ] cargo test -p shuf --locked 全绿
- [ ] 提交 fix(shuf): seed from random-source file contents

### Task 45: du — -s 递归

**证据:** 报告 P0 #45;`crates/du/src/main.rs:19,44`。
**方案:** `-s` 时仍递归累加,仅不打印子目录行(打印参数行合计);默认模式维持只输出参数行(GNU 逐子目录列为 P2);错误退出码 1 顺带修(同函数)。
**测试:** 嵌套目录 `du -s` 数值等于各文件字节之和且只输出一行;`du`(无 -s)输出一行;不可读路径 exit 1。

- [ ] 失败回归测试
- [ ] 修递归条件
- [ ] cargo test -p du --locked 全绿
- [ ] 提交 fix(du): recurse for summarize

---

## Phase F — 收尾

### Task 46: 仓库杂物清理

**内容:** 删除提交进仓库的手工测试杂物:crates/{tee/test_tee.txt, split/test_split.txt, chmod/test_chmod.txt, chmod/test_dir/, readlink/test_target.txt, rev/test.txt, sed/test_sed..bak, join/test1-4.txt, patch/test.patch, patch/test.txt, patch/test.txt.rej, md5sum/test.txt, md5sum/md5sum.txt, tar/test.tar, tar/test.txt, tar/extract_test/, cat/test_cat.txt, find/test_cat.txt}(以 git ls-files 实际结果为准)以及 crates/ls 下 .env、reasonix.toml、.codegraph/。

- [ ] git ls-files 核对清单
- [ ] git rm 删除
- [ ] 提交 chore: remove hand-test litter from crate roots

### Task 47: 文档同步核销

**内容:** 汇总本轮所有"文档"标注项,核对 `docs/compatibility.md` 与 `-zh` 两条线与代码一致;核销审查报告"文档同步项"清单中与本轮相关的条目(grep/cmp 漏列等属 P1 文档项,若顺手一并补);CHANGELOG Unreleased 增加本轮条目。

- [ ] 逐条核对 compatibility 双语
- [ ] CHANGELOG 更新
- [ ] 提交 docs: sync compatibility tables for p0 fixes

### Task 48: 全量验证

- [ ] `pwsh -File scripts/build.ps1 -Task Verify` 通过
- [ ] `pwsh -File scripts/release.ps1 -OutputDir dist/p0-verify` 本地打包冒烟通过
- [ ] 对照审查报告 P0 清单 50 项逐条标注状态(修复/移除声明/声明差异),回填 ledger
- [ ] 提交 chore: p0 remediation verification pass

---

## Self-Review

**1. 覆盖核对:** 审查报告 P0 清单 50 项 → Task 映射:1-12(T27/T28/T29/T30/T11/T12/T5/T31/T32/T33/T34/T13/T35/T36/T37/T38/T39/T40/T41/T42/T43/T44/T6/T10/T1/T3/T2/T21/T15/T16/T17/T18/T18/T22/T22/T19/T11/T45/T25/T26/T23/T24/T20)——50 项全部有归属,无遗漏。
**2. 依赖策略:** 无新 crate;`windows` 可能新增 feature(Win32_NetworkManagement_Dns,Task 26);stdlib 优先。
**3. 非目标防越界:** 每个 Task 标注的 P1/P2 相邻项均以"顺带、同函数、低成本"为限,超出的记录到 ledger 不做。

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-10-07-p0-remediation.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, record to `.superpowers/sdd/progress.md`

**2. Inline Execution** — execute tasks in this session with checkpoints

**Which approach?**
