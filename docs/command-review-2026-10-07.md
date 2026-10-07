# RuxCmd 全量命令代码审查报告(2026-10-07)

## 概述

对全部 **111 个命令(113 个 package)** 逐一进行了源码级只读审查,每个命令按四个维度评估:功能、移植完整性(对照 GNU coreutils / procps / binutils 的常用用法)、技术准确性(panic、边界条件、错误传播、退出码)、Windows 合理性(Win32 API 使用、路径/编码处理、Unix 概念映射)。所有问题均附 `file:line` 证据,并逐条与 `docs/compatibility.md` 的声明核对——**已声明为有意差异的行为不计为缺陷**。

审查方式:13 个并行审查批次,每批次通读该组命令的 `src/`、`tests/` 与 `Cargo.toml`,并对照兼容性表逐行核实声明。其中两处疑似 P0(base64 解码、bc 求值)已按源码逻辑逐行模拟验证。

## 总体结论

**项目目前处于"冒烟级"质量状态,与 compatibility.md 中绝大多数命令标注的 "Smoke only; behavior unverified" 相符。** 全部命令都能编译、启动并响应 `--help`,但:

- **约 49 个命令(44%)存在至少一个 P0 级正确性缺陷**——在最普通用法下产生错误输出、崩溃、假数据或安全风险;
- **48 个命令存在 P1 级问题**(重要 GNU 功能缺失、错误契约失效、脆弱实现),无 P0;
- **仅 14 个命令基本健康**(仅 P2 或无需调整):whereis、mkdir、mkfifo、comm、expand、rev、hexdump、strings、gunzip、zcat、whoami、uname、clear、reset;
- 有行为测试的命令仍只有 10 个(cat、echo、wc、grep、cp、mv、rm、true、false、ps),测试覆盖与缺陷分布一致——测试过的命令问题明显更少。

## 统计总览

| 等级 | 含义 | 命令数 |
| --- | --- | --- |
| P0 | 正确性缺陷:错误输出/崩溃/假数据/安全风险,应尽快修 | 49 |
| P1 | 重要功能缺口或契约失效,无 P0 | 48 |
| P2 | 次要改进项 | 14(仅 P2) |

## P0 缺陷清单(49 个命令)

| # | 命令 | P0 缺陷 | 证据 |
| --- | --- | --- | --- |
| 1 | cat | `\r` 无条件丢弃 + 非 ASCII 经 Latin-1 重编码乱码,无选项时也破坏数据 | main.rs:130-133, 99, 168 |
| 2 | echo | 默认解释转义(应默认 -E),`echo C:\new` 输出错误 | main.rs:10 |
| 3 | printf | 格式串反斜杠转义未实现,`printf "a\nb"` 字面输出 | main.rs:86-88 |
| 4 | seq | 非数字参数 unwrap panic | main.rs:20/24/30 |
| 5 | ls | `-li/-ls` 用含色码显示名反查条目失败,inode/块数列静默丢失 | display.rs:322-344 |
| 6 | stat | `%t/%b/%m` 与 GNU 语义冲突;时间按 UTC 而非本地 | main.rs:124-135, 157-171 |
| 7 | file | `-b` 从未使用;zlib 魔数误标 GZIP;单个文件打不开即中止全部 | cli.rs:16, main.rs:50-69, 431-439 |
| 8 | which | 输出 `\\?\` verbatim 路径 | main.rs:45-46 |
| 9 | readlink | `-f` 输出 `\\?\` verbatim 路径 | main.rs:12-16 |
| 10 | realpath | `--strip-components` 偏移 1(Prefix/RootDir 计入) | main.rs:31-47 |
| 11 | dirname | `dirname /` 输出 "." | main.rs:41-44 |
| 12 | basename | 尾部 `./`/`..` 时输出原路径而非 `.`/`..` | main.rs:26-31 |
| 13 | cp | `cp -r . dest` 等场景 `file_name().unwrap()` panic | main.rs:104 |
| 14 | touch | 已存在文件不更新时间戳,核心功能 no-op | main.rs:27-43 |
| 15 | grep | 单文件 `-c` 无任何输出 | main.rs:59 |
| 16 | diff | 退出码恒 0;普通格式行号全错 | main.rs:58-62, 140-163 |
| 17 | sort | `-k/-t` 声明未实现;`-u` 排序前去重;`-c -r` 死分支 | cli.rs:33-38, main.rs:35-43, 67-75 |
| 18 | uniq | 输出比较键而非原行,`-i/-f` 下输出错误 | main.rs:44-47 |
| 19 | tr | 无 SET1 时 expect panic;空 SET2 下溢 panic | main.rs:112, 71 |
| 20 | nl | byte-as-char,非 ASCII 输入全部乱码 | main.rs:116 |
| 21 | tac | 尾分隔符多出空段;无尾换行输入语义错误 | main.rs:114-116, 105-116 |
| 22 | wc | `-m` 逐字节计数(等价 -c);`-L` 偏大 1 | main.rs:117-118, 119, 123 |
| 23 | tail | 不支持 stdin;`-n 0 -f` 输出全文件 | main.rs:12-15, 74-78 |
| 24 | tee | 逐行处理破坏 CRLF 与二进制数据 | main.rs:48-61 |
| 25 | split | 字母后缀序列错位(x0a,应为 xaa);行模式非字节透明 | main.rs:96-109, 166-180 |
| 26 | csplit | 重复路径丢匹配行;`/re/+N` 静默失效 | main.rs:44-56, 40-43 |
| 27 | column | 默认分隔符下连续空白不折叠,列错位 | main.rs:121-139 |
| 28 | pr | 页眉日期手搓估算错误;裸 `-n` 直接报错 | main.rs:127-136, cli.rs:33 |
| 29 | shuf | `--random-source` 把参数字符串哈希成种子,结果确定性 | main.rs:16-17, 116-123 |
| 30 | base64 | 解码对含 `=` 填充或非 4 倍长度输入输出错误字节,与自身编码往返失败 | main.rs:135-154, 178-193 |
| 31 | bc | 任何含运算符的表达式恒输出 0 | main.rs:62, 81-134 |
| 32 | tar | 解包无路径清洗,`../../x` 与绝对路径条目逃逸目标目录(zip slip 同类) | main.rs:71, 77 |
| 33 | zip | 文件名未转义拼入 PowerShell 命令:注入 + 含空格文件名必败 | main.rs:19-28 |
| 34 | unzip | zip slip:join 原始条目名,未用 `enclosed_name` | main.rs:58 |
| 35 | ps | `%MEM` 硬编码总内存 8GB | ps-core/src/columns/pmem.rs:20 |
| 36 | nice | 主路径整体失效:参数丢弃、priority 值非法且方向反、不等待、退出码恒 0 | main.rs:16-39 |
| 37 | timeout | 子进程提前完成后仍阻塞满额时长;超时退出码 124 永不触发 | main.rs:32-41, 43-46 |
| 38 | top | 快照失败时 `processes - 1` u64 下溢 | main.rs:78 |
| 39 | uptime | Windows 分支返回 epoch 至今秒数而非开机时长 | main.rs:32-38 |
| 40 | w | 三处 expect,无 query.exe 的系统必 panic | main.rs:10, 28, 46 |
| 41 | id | `-G`/groups 输出编造数据而非令牌真实组 | main.rs:30-47 |
| 42 | groups | USER 参数被忽略;组数据编造 | main.rs:15, 61-75 |
| 43 | date | `-s` 被 --set/--seconds 双声明(debug 构建 panic);`--date/--set` 静默 no-op | cli.rs:12, 16 |
| 44 | cal | `cal 2024` 单参数按月份解析后 unwrap panic | main.rs:25 |
| 45 | du | `-s` 不递归,目录输出 ≈0 | main.rs:19, 44 |
| 46 | vmstat | 内存只在循环外采样一次,多帧输出全同 | main.rs:150 |
| 47 | nslookup | MX/TXT/CNAME 输出伪造数据 | main.rs:74-97 |
| 48 | chown | `--group` 实际修改的是 owner | main.rs:22, 129, 159 |
| 49 | tty | 用 CON 环境变量判断,真实终端恒 "not a tty" | main.rs:37-47 |
| 50 | patch | `-p/--strip` 接受参数但零引用;新建文件补丁 usize 下溢 | cli.rs:16-19, main.rs:223 |

## 横切共性问题(系统性)

1. **`BufRead::lines()` 行处理模式波及十余个文本命令**:CRLF 被改写为 LF、末行无换行符时补 `\n`、非 UTF-8 输入中断或静默截断(head/tail/tee/split/csplit/column/pr/shuf/grep/cut/sort/fold 等)。建议抽出一个字节安全的行读取层统一修复。
2. **byte-as-char / Latin-1 重编码乱码**(cat、nl、cut -b、uniq 比较键):字节被 `as char` 塞进 String 再按 UTF-8 输出,任何非 ASCII 数据损坏。
3. **退出码契约大面积失效**:失败仍 exit 0 的命令包括 wc、rm -f、diff、file、df、du、ping、kill、pkill、pgrep、chown、sort、xargs、shred、hostname、timeout 等。这破坏脚本/CI 对退出码的一切依赖。
4. **输出假数据比报错更糟**:nslookup 的 MX/TXT/CNAME、id/groups 的组列表、mpstat/iostat/sar/vmstat 的全零列、time 的 user/sys、ps 的 pcpu/stat、w 的头部、shuf 的随机源——用户拿到的是貌似合理的错误数据。
5. **选项声明与实现脱节(no-op 选项)**:sort -k/-t、date -d/-s、sar -d、netstat -a/-n、iostat -d/-x、patch -p、shred --force、timeout -k、ps --headers-repeat、cp --preserve、env 命令执行。compatibility.md 亦有约 10 处与代码不同步(grep/cmp/diff/join/od/hexdump/numfmt/uname/basename/ls/chown)。
6. **shell out 外部程序脆弱**:hostname/pkill/pgrep/nice/sync/zip/w/who/groups/uname 依赖 PowerShell/wmic/tasklist/taskkill/systeminfo/net user。其中 wmic 在 Windows 11 24H2+ 已移除、PowerShell 5.1 生成的 zip 条目名用 `\` 导致跨平台不可移植、sync 存在 PATH 自递归风险。ps-sys 已具备原生枚举/终止能力但未被 pkill/pgrep/nice 复用。
7. **panic 面较大**:seq、cal、cp/mv 的 `.`/`..`、tr 空参、join -1 0、od -t ''、hexdump 边界、sed/awk 的 file_name().unwrap()、shred 写失败、yes 的 BrokenPipe(管道关闭 exit 101)、w 的 expect 链。
8. **Windows API 使用存在正面样板**:ps-sys 的 FFI 层、free(GlobalMemoryStatusEx)、df(GetDiskFreeSpaceExW)、find(symlink_metadata 防环)、ping(IcmpSendEcho 免管理员)、iconv(encoding_rs,GBK 系列可用)、chown(安全 API 链路方向正确)——这些可作为后续整改的参考实现。
9. **crate 根目录散落手工测试杂物**(已提交进仓库):tee/test_tee.txt、split/test_split.txt、chmod/test_chmod.txt 与 test_dir/、readlink/test_target.txt、rev/test.txt、sed/test_sed..bak、join/test1-4.txt、patch/test.*、md5sum/test*、tar/test.tar 与 extract_test/、ls 下 .env/reasonix.toml/.codegraph、find 的 test_cat.txt 等,应统一清理。

## 逐命令详细结果

### 第 1 组:基础输出与数值(cat / echo / printf / yes / true / false / seq / sleep / pwd / hostname)

#### cat
- 功能:拼接文件/stdin 输出到 stdout,支持 -n/-b/-s/-E/-T/-v/-A 行号与不可见字符显示。
- 移植完整性:已覆盖 -n -b -s -E -T -v -A -u 与 `-` 读 stdin、出错继续处理下一文件、exit 1;无重大缺失,`-u` 仅为占位。compatibility.md 声明与代码一致,但漏列了代码里实际存在的 `--unbuffered`。
- 技术评估:① `\r` 字节被无条件丢弃(main.rs:130-133),CRLF 文本与二进制数据(如 `cat a.png > b.png`)被破坏,GNU cat 原样透传;② 所有输出字节经 `(byte as char)` 按 Latin-1 映射再以 UTF-8 重编码(main.rs:99、168),任何非 ASCII 内容(UTF-8 中文、二进制)输出即乱码;③ 整个文件 `read_to_end` 进内存(main.rs:115-116);④ 无结尾换行的最后一行在 -E 下也补 `$`(main.rs:186-188);⑤ 测试仅覆盖 ASCII(crates/cat/tests/cli.rs:7),掩盖了上述问题。
- Windows 合理性:按字节处理思路合理,但把字节当 char 塞进 String 在 UTF-8 中文场景必错;正确做法是原样 io::copy 字节。
- 结论:P0 修复 `\r` 丢弃与字节→char 重编码;P1 改流式输出;P2 补 --unbuffered 文档、修无换行末行加 `$`、补非 ASCII/二进制测试。

#### echo
- 功能:输出参数拼接的文本,支持 -n(无换行)、-e/-E(转义)。
- 移植完整性:已覆盖 -n -e -E;缺失 `\a \b \c \e \f \v \0NNN \xHH`(main.rs:30-52 只实现 \n \r \t \\ \"),`\c` 与 `\0NNN` 脚本常用。
- 技术评估:默认行为与 GNU 相悖——未给 -e/-E 且有参数时仍解释转义(main.rs:10),`echo C:\new` 输出 `C:`+换行+`ew`,Windows 路径反斜杠场景直接出错;未知选项 `-x` 被 clap 拒绝 exit 2,GNU echo 将其按普通文本输出。
- Windows 合理性:UTF-8 经 Rust std 写控制台合理;无文件 I/O。
- 结论:P0 默认应关闭转义(默认 -E,main.rs:10);P1 补齐转义序列(main.rs:23-60);P2 `echo -x` 应按操作数输出。

#### printf
- 功能:按 %s/%d/%x/%f/%c/%% 对参数做最简格式化输出。
- 移植完整性:仅 6 个最基本指示符;缺失格式串反斜杠转义、宽度/精度/标志(%5s、%.2f、%05d)、%i %u %o %X %e %g %b、格式循环复用、参数缺失时输出空串/0。
- 技术评估:① 格式串完全不处理反斜杠转义(main.rs:86-88),`printf "a\nb"` 输出字面 `a\nb`,最常用用法失效;② 参数不足时输出字面 `%s`/`%d`(main.rs:23/32/44/56/70);③ 非法数字不报错仍 exit 0(main.rs:31-33);④ `%f` 用 Rust Display,`1.5` 而非 GNU 的 `1.500000`(main.rs:54);⑤ `--version` 会被 clap 当 flag,无法作为被打印的操作数。
- Windows 合理性:纯字符串处理,无平台问题。
- 结论:P0 实现格式串转义(main.rs:86-88);P1 补宽度/精度/标志与缺失指示符、对齐 GNU 参数语义;P2 修 %f 精度、trailing_var_arg、补测试。

#### yes
- 功能:无限重复输出 "y" 或指定字符串。
- 移植完整性:缺失多操作数拼接(`yes a b` 应输出 "a b",cli.rs:9 只收单个)。
- 技术评估:stdout 管道关闭时 `println!` panic(main.rs:11),exit 101 并打印 panic 信息;GNU yes 对 EPIPE/SIGPIPE 静默退出。全仓库未见任何 BrokenPipe 处理。
- Windows 合理性:无限 println! 每行 flush,性能弱于 GNU 缓冲写,可接受。
- 结论:P1 捕获 BrokenPipe 静默退出(main.rs:11);P1 支持多操作数(cli.rs:9);P2 加缓冲输出。

#### true / false
- 功能:恒以退出码 0/1 结束,--version/--help 可用。
- 移植完整性:GNU 规定忽略一切操作数与未知选项;此处只声明 --version。
- 技术评估:`true foo`、`true --bar` 被 clap 以 usage error 拒绝,退出码 2(POSIX 要求 0);false 同理应为 1 而非 2。版本输出硬编码 "true 1.0.0"(main.rs:11)。tests/cli.rs 仅测无参退出码。
- Windows 合理性:无平台问题。
- 结论:P1 让未知参数被忽略并保持 exit 0/1;P2 版本串改 env!("CARGO_PKG_VERSION")。

#### seq
- 功能:打印等差数列,支持 -s 分隔符、-w 等宽补零、-f 格式(仅 %f/%g/%e 三种字面量)。
- 移植完整性:缺失通用 printf 浮点格式(`%.2f` 等均被拒,main.rs:87-90)、GNU 的 `-t/--terminator`。
- 技术评估:① 非数字参数 `.unwrap()` panic(main.rs:20/24/30),`seq abc` exit 101;② 输出缺结尾换行(main.rs:14);③ f64 累加误差:`seq 0 0.1 1` 丢终点或输出 0.30000000000000004(main.rs:46-49);④ `seq inf` 无限生成 Vec 吃尽内存;⑤ 全序列物化为 Vec(main.rs:41-61),`seq 1 100000000` 内存数十 GB;⑥ -w 对负数把零补在 `-` 前(main.rs:111-119)。
- Windows 合理性:纯计算无平台问题。
- 结论:P0 参数解析改报错退出、消除 panic;P1 补结尾换行、修浮点累加、拒绝 inf、--format 支持任意 printf 浮点格式;P2 流式输出、修 -w 负数、补测试。

#### sleep
- 功能:休眠指定时长,支持 s/m/h 后缀。
- 移植完整性:缺失小数时长(`sleep 0.5` 报错,cli.rs:14)、`d` 后缀、多参数求和。
- 技术评估:`18446744073709551615h` 溢出(main.rs,cli.rs:22),release 回绕;错误退出码为 clap 的 2,GNU 为 1。
- Windows 合理性:thread::sleep 是恰当映射。
- 结论:P1 支持 f64 时长(cli.rs:14、22);P2 d 后缀、多参数、溢出检查、退出码。

#### pwd
- 功能:打印当前目录;-P 物理路径,-L/$PWD 逻辑路径,默认 -L(与 GNU 一致)。
- 移植完整性:无重大缺失。
- 技术评估:`fs::canonicalize` 返回 `\\?\` 或 `\\?\UNC\` 前缀,format_path 只剥 `\\?\`(main.rs:53),映射盘/UNC 下 `pwd -P` 输出错误路径。
- Windows 合理性:本组中最像样的移植:verbatim 剥离、`\` 转 `/`、$PWD 校验回退;仅 UNC 变体遗漏。
- 结论:P1 修复 `\\?\UNC\` 前缀剥离(main.rs:53)。

#### hostname
- 功能:打印主机名;-i/--ip-address 打印 IPv4,-f/--fqdn 打印 FQDN。
- 移植完整性:缺失 -s/--short、-d/--domain、-a/--alias、-F/--file。
- 技术评估:① 失败路径静默(不打印错误、退出码 0,main.rs:10-22);② 未检查子进程退出状态(main.rs:43-51/69-78);③ `-v` 与 net-tools hostname 的 verbose 语义冲突(cli.rs:18)。
- Windows 合理性:默认取 COMPUTERNAME 环境变量偏脆弱(main.rs:25-34);-i/-f 借助 PowerShell 子进程(main.rs:40-42、66-69),冷启动数百 ms 至秒级,应改 GetAdaptersAddresses/GetComputerNameExW——本组唯一 shell out 实现。
- 结论:P1 失败时打印错误并 exit 1;P2 改用原生 Win32 API、补 -d/-s、重审 -v 冲突。

### 第 2 组:文件信息与路径(ls / stat / file / which / whereis / readlink / realpath / dirname / basename)

#### ls
- 功能:列出目录/文件内容,支持长格式(-l)、分栏(-C/-x/-m/-1)、六种排序、隐藏过滤、-R 递归、-F/-p 指示符、-Q 引用、--color、--time-style、--block-size 等。
- 移植完整性:覆盖面全组最好;缺失 `-h` 短选项(仅 long,cli.rs:114)、`-o`、`-L/--dereference`、`-f`、`--si`;宽度短选项用了 `-W`(GNU 是 `-w`,cli.rs:138)。
- 技术评估:① P0:print_long 用"显示名"(含 ANSI 色码/引号/-F 指示符)反查原始条目(display.rs:322-344),交互终端默认 color=auto 时匹配必然失败,`ls -li/-ls` 的 inode/块数列被静默丢弃;② 单操作数 `ls -l` 不输出 total 行(main.rs:17);③ 多操作数时对普通文件也打印 "path:" 头(main.rs:23-28);④ display_name/pad_right 用字节长度而非显示宽度(display.rs:416, 521-531),中文文件名分栏错位;⑤ `--color` 未设 require_equals(cli.rs:134),`ls --color foo` 把 foo 当 WHEN;⑥ `-k` 被错误用于把 size 除以 1024(display.rs:435-437);⑦ `-a` 不输出 `.` 和 `..`(entry.rs:131-155);错误退出码 2 符合惯例(main.rs:41)。
- Windows 合理性:owner 显示当前 USERNAME、group 显示 USERDOMAIN(perm.rs:102-111)是应注明的近似;隐藏属性把权限末字符替换为 `h`(perm.rs:78-81,产生 `-rw-r--r-h`);目录 nlink=目录项数+1(entry.rs:94-103);inode 恒 `?`(entry.rs:64-71);GetConsoleScreenBufferInfo 取终端宽合理(display.rs:534-593)。
- 结论:P0 修 -li/-ls 列丢失(display.rs:322);P2:total/文件头/对齐/--color 贪婪/-k/-a/.与../-h 短选项/owner 简化;crate 目录混入 .env、reasonix.toml、.codegraph 等杂物应清理。

#### stat
- 功能:显示文件大小/类型/三种时间戳,-c 支持约 14 个格式指令子集,-t 简表,-f 占位输出。
- 移植完整性:缺失 -L/--dereference、--printf 及常用指令 %A/%a/%F/%y/%Y/%x/%X/%z/%Z/%W(未识别指令原样输出而非报错,main.rs:144)。
- 技术评估:① P0:`-c` 指令与 GNU 语义冲突——%t 输出类型字符(GNU:设备主号 16 进制)、%b 输出字节数(GNU:512B 块数)、%m 输出 mtime(GNU:挂载点)(main.rs:124-135);② P0:format_time 输出 UTC 而非本地时区(main.rs:157-171);③ P1:`-f` 输出全为 "unknown"(main.rs:71-86),纯占位;④ Access: 行打印 Rust Debug 格式(main.rs:63);terse 硬编码 rwxrwxrwx(main.rs:102);Device/Inode/Links/Uid/Gid 全硬编码 0/1(main.rs:60-65);缺操作数报 missing operand 退出 1 正确(main.rs:9-12)。
- Windows 合理性:硬编码是可接受简化,可用 file_index/卷信息改进。
- 结论:P0 修 %t/%b/%m 语义与 UTC 时间;P1 -f 占位;P2:-L/--printf、Debug 输出、terse 硬编码。

#### file
- 功能:读文件前 256 字节,对约 45 条硬编码魔数规则做前缀匹配,输出类型名或 MIME(-i)。
- 移植完整性:缺失 -L、-z、-f 清单文件;最常见场景缺失——无文本/编码检测,纯 ASCII/UTF-8(无 BOM)文本一律报 "data"(main.rs:425-427)。
- 技术评估:① P0:`-b/--brief` 已定义但 main 从未使用(cli.rs:16),`file -b x.txt` 仍输出 "x.txt: ...";② P0:zlib 魔数 78 01/9C/DA 被标为 "GZIP compressed"(gzip 实为 1F 8B)(main.rs:50-69),文本文件以 "x"+非 ASCII 开头会被误判;③ P0:遇到打不开的文件立即 exit(1) 中止剩余文件(main.rs:431-439),GNU 继续处理并在结尾返回 1;④ WebP 规则要求文件以 "WEBP" 开头(main.rs:141-146),真实 WebP 的标记在偏移 8,永不命中;单次 read() 不保证读满 256 字节(main.rs:392);无操作数时不读 stdin(main.rs:405-408)。
- Windows 合理性:纯字节匹配,无路径/编码耦合;对目录/特殊文件无类型识别。
- 结论:P0 修 -b 无效、zlib 误标、遇错中止;P1 补文本检测;P2 其余。

#### which
- 功能:沿 PATH 逐目录查找命令,候选扩展名 ["", .exe, .cmd, .bat, .com],支持 -a。
- 移植完整性:基本可用;GNU 的 --skip-dot/--quiet 属冷门。
- 技术评估:P0:对每个命中路径做 fs::canonicalize() 后输出(main.rs:45-46),Windows 上输出 `\\?\C:\...` verbatim 反斜杠路径,难看且不可直接复用。
- Windows 合理性:扩展名硬编码而忽略 PATHEXT(main.rs:38);不搜当前目录属 Unix 习惯,与定位一致。
- 结论:P0 修 verbatim 输出;P2 改用 PATHEXT、模仿 `where` 的净路径输出。

#### whereis
- 功能:在 PATH 各目录及一个硬编码目录中查找命令,按 GNU 风格输出 `cmd: path...`。
- 移植完整性:缺失 -b/-m/-s(二进制/手册/源码分类)及 -u/-B/-M/-S/-l。
- 技术评估:P2:with_extension("exe") 会把带点命令名改写——`whereis a.b` 实际找 a.exe(main.rs:17-18);硬编码 `D:\develop\rust-tools` 为作者机器专属路径(main.rs:29-34,文档已声明待修正);未找到时输出 "not found"(GNU 仅输出 `cmd:`);无参数时静默退出 0。
- Windows 合理性:按 PATH 查 .exe 思路合理,同样不查 PATHEXT。
- 结论:无需紧急调整;P1 缺 -b/-m/-s;P2 with_extension、无参数行为、硬编码路径清理。

#### readlink
- 功能:读取单个符号链接的目标,-f/--canonicalize 解析为绝对路径,-n 不输出换行;出错退出 1。
- 移植完整性:缺失 -e/-m、-q/-s、-z 及多操作数。
- 技术评估:P0:`-f` 直接输出 fs::canonicalize() 结果(main.rs:12-16),`\\?\C:\...` verbatim 路径——脚本最常用形式基本不可用;同族的 realpath/dirname 已示范 strip_prefix(r"\\?\") + 反斜杠转 `/`(crates/realpath/src/main.rs:50-54),此处未做。非符号链接报错退出 1 正确。
- Windows 合理性:未处理 verbatim 前缀与路径分隔符是主要问题。crate 根目录遗留 test_target.txt 已提交,建议删除。
- 结论:P0 修 -f verbatim 输出;P2 缺 -e/-m/多操作数、清理杂物。

#### realpath
- 功能:将单个路径 canonicalize 为绝对路径输出,--strip-components=N 剥离前导组件;正确清理 `\\?\` 并统一为 `/`(main.rs:50-54),本组中 Windows 路径处理最好。
- 移植完整性:缺失 -e/-m、-q、-z、多操作数;`-s` 短选项在 GNU 里是 --strip,此处给了 --strip-components(cli.rs:8),语义撞车。
- 技术评估:P0:strip 计数把 Prefix(盘符)与 RootDir 也算作组件——`C:\a\b` 的 --strip-components=1 输出 `/a/b`,GNU 输出 `a/b`,整体偏移 1(main.rs:31-47);无操作数时默认 "."(cli.rs:16-17),GNU 报 missing operand 退出 1。
- Windows 合理性:format_path 处理正确;问题集中在组件计数语义。
- 结论:P0 修 strip-components 偏移;P2:-s 撞车、无操作数行为、缺 -e/-m/-q/-z。

#### dirname
- 功能:对每个操作数输出去掉最后一个非目录成分后的父路径(支持多操作数),根/空路径回退 "."。
- 移植完整性:缺失 -z/--zero;多操作数已覆盖。
- 技术评估:P0:`dirname /` 输出 "."——Windows 上 Path::new("/").is_absolute()==false 走回退分支(main.rs:41-44),GNU 输出 "/";根分支直接 to_string_lossy 不经 format_path(main.rs:26-33),UNC 与反斜杠原样输出,风格不一致;`dirname C:/x` → "C:/"、`a` → "." 均正确。
- Windows 合理性:format_path 的 verbatim 剥离与 `/` 转换在非根路径上正确。
- 结论:P0 修根路径 case;P2 根分支格式统一、缺 -z。

#### basename
- 功能:输出路径最后成分;-s/--suffix 去后缀,-a/--multiple 多操作数。
- 移植完整性:P1:缺失 POSIX 传统双操作数形式 `basename NAME SUFFIX`(如 `basename foo.c .c`)——第二个位置参数被当作另一个 NAME 输出,这是最经典的用法;缺失 -z。源码的 --suffix/--multiple 长选项 compatibility.md 未列,建议同步。
- 技术评估:P0(边缘):尾部为 `.`/`..`/`./` 时 file_name() 返回 None,fallback 输出整条原路径——`basename ./` 输出 `./`(GNU 输出 ".")(main.rs:26-31);`basename //` 输出 "//"(GNU 输出 "/")。后缀字节切片无 panic 风险(已验证);`basename foo foo` 不删除与 GNU 一致(main.rs:33-39)。
- Windows 合理性:基于 Path 的实现合理。
- 结论:P1 补双操作数后缀形式;P0(边缘)修尾部 ./. 输出;P2 -z、文档同步。

### 第 3 组:文件操作——破坏性命令(cp / mv / rm / ln / mkdir / touch / open / mkfifo)

依赖核查:8 个 crate 均只依赖 clap;mkfifo 额外用 winapi 0.3(合理);无 shell out,唯一例外 open(属其功能本质)。compatibility.md 声明的显式长选项与各 cli.rs 逐一核对全部一致;测试声明与 tests/ 实际一致。

#### cp
- 功能:单文件/多源复制与 -r 目录递归复制,支持 -f/-i/-s/-v。
- 移植完整性:已覆盖 -r/-R -f -i -s -v;缺失 -a/--archive、-n/--no-clobber、-u/--update、-t/-T、-l/--link、-b/--backup。
- 技术评估:① P0:`cp -r . dest` / `cp -r .. dest` 在 dest 为已存在目录时 `source.file_name().unwrap()` panic(main.rs:104);② 多源复制到非目录目标时无校验,`cp a b c`(c 为文件)把 a、b 依次静默覆盖写入 c(main.rs:47-51);③ `--preserve` 仅为 cli.rs:21 声明的空壳,main.rs 全程未读 args.preserve,静默 no-op;④ 与 GNU 相反,默认(无 -f/-i)拒绝覆盖已存在文件(main.rs:53-64),未声明为有意差异;⑤ `cp -r src src` 无自复制检测,递归创建 src/src/src/… 直到路径过长(main.rs:110/114);⑥ `-s` 在目录递归中被忽略(main.rs:143)。
- Windows 合理性:fs::copy 复制只读属性,目标只读时即使 -f 也直接失败,未先清属性(GNU -f 语义)。
- 结论:P0 `.`/`..` panic(main.rs:104);P1 多源目标覆盖、--preserve no-op;P2 默认不覆盖语义(建议改为 GNU 语义或声明)、自复制、只读目标、-s+-r。

#### mv
- 功能:单/多源移动或重命名,dest 为目录时移入其中,支持 -f/-i/-n/-v。
- 移植完整性:缺失 -t/-T、-u/--update、-b/--backup、-S/--suffix。
- 技术评估:① P1:`mv -f file nonempty_dir` 先 remove_file 失败则 remove_dir_all 整树删除 dest 再 rename(main.rs:65-67),GNU 拒绝 "cannot overwrite directory with non-directory"——本组最危险的静默删树路径;② 跨盘移动无 copy+delete 回退,fs::rename 跨卷直接失败(main.rs:70-77);③ `.`/`..` 源且 dest 为已存在目录时 file_name().unwrap() panic(main.rs:37);④ 默认拒绝覆盖(main.rs:56-62),未声明;⑤ 目录源覆盖文件目标时不报错(main.rs:65-77)。
- Windows 合理性:只读目标文件 remove_file 失败,-f 不能先清只读属性;跨盘/占用句柄无回退。
- 结论:P1 ① 整树删除、② 跨盘回退、③ panic;P2 ④⑤、只读属性。

#### rm
- 功能:删除文件/符号链接,-r 递归删树、-d 删空目录,含 preserve-root(Windows 化为保护 `C:\`)、-I 单次确认、--one-file-system(按盘符)。
- 移植完整性:覆盖面在本组最好(-f -i -I -r/-R -d -v --one-file-system --no-preserve-root/--preserve-root);缺失 --recursive/--interactive 长名、--interactive=never。
- 技术评估:① P1:`-f` 下所有 remove 失败被吞并返回 Ok——只读/占用文件删除失败时静默且退出码 0,报告成功但文件仍在(main.rs:239-249),GNU -f 对 EPERM 仍打印错误并退出 1;② `-f` 同时压制所有错误输出,含 root 保护拒绝信息(main.rs:52-57);③ preserve-root 可被 verbatim 路径绕过:`rm -rf \\?\C:\` 两处检查均匹配不到 `\\?\C:`(main.rs:71-92);④ 递归 -i 无 "descend into directory" 提示(main.rs:307-334);⑤ 拒绝个别文件后末尾 remove_dir 报 "Directory not empty" 造成误导(main.rs:338)。
- Windows 合理性:保护 C:/ 的映射合理;one-file-system 用盘符近似合理;只读文件无法删除且未清 FILE_ATTRIBUTE_READONLY;`-v` 输出走 stderr(main.rs:180/235/300),GNU 为 stdout。
- 结论:P1 ① -f 假成功退出码;P2 ②③④⑤、只读属性、verbose 流向。

#### ln
- 功能:创建硬链接(默认)或 -s 符号链接,-f 覆盖已存在链接。
- 移植完整性:已覆盖 -s -f;缺失单操作数形式(`ln target`)、-t/-T、-i/-n、-r/--relative、-b。
- 技术评估:① P1:fs::canonicalize(target)(main.rs:17)把链接目标解析为 `\\?\C:\...` verbatim 绝对路径存入新建链接,GNU 按用户书写形式存储,readlink 等工具看到的是带前缀的脏路径;② `-v` 被绑定为 --version(cli.rs:6/25),GNU ln 的 -v 是 --verbose,`ln -v a b` 会打印版本而不创建链接;③ link_name.is_dir() 分支 file_name().unwrap() 因 target 先被 canonicalize 实际安全(main.rs:20)。
- Windows 合理性:硬链接走 NTFS、软链需特权/开发者模式,compatibility.md 已声明;verbatim 目标是未声明的实际问题。
- 结论:P1 ① canonicalize 污染链接目标;P2 ② -v 语义冲突、③ 单操作数形式、缺 -t/-T/-i/-n/-r。

#### mkdir
- 功能:创建目录,-p 递归、-v 打印、-m 接受但忽略。
- 移植完整性:与声明一致,常用项无实质缺失。
- 技术评估:未发现明显缺陷;-m 不做格式校验(cli.rs:12-14);`-pv a/b/c` 只打印最终目录,GNU 逐级打印(main.rs:29-35)。
- Windows 合理性:纯 std 实现,无问题。
- 结论:无需调整(P2:-m 格式校验、-pv 逐级 verbose)。

#### touch
- 功能:文件不存在时创建空文件,-c 禁止创建;但对已存在文件不做任何时间戳更新。
- 移植完整性:仅 -c/--no-create;缺失 -a、-m、-d/--date、-t、-r/--reference、--time、-h。
- 技术评估:P0:对已存在文件仅 OpenOptions::append(true).open() 后立即关闭(main.rs:29-31),不更新 mtime/atime——"touch 已存在文件"是静默 no-op,与 cli.rs:4 "update file timestamps" 的声明直接矛盾;对目录 touch 报 Access denied(main.rs:29);只读文件报错。
- Windows 合理性:应走 SetFileTime(FILE_WRITE_ATTRIBUTES)而非打开写句柄,可同时解决目录/只读问题;当前实现选错了 API 层面。
- 结论:P0 时间戳更新未实现,需按 SetFileTime 重写;P2 目录/只读行为,补齐 -a/-m/-d/-t/-r。

#### open
- 功能:用 shell 关联程序打开文件/目录/URL,-a 指定程序、-e 记事本、-W 等待、-R 定位。
- 移植完整性:已覆盖 macOS open 主要项;缺失 -t、-b、--args;无参数时默认打开当前目录属合理移植选择。
- 技术评估:① P1:`-R` 必然报错——explorer.exe 惯以退出码 1 结束,execute_command 视为失败(main.rs:127-142 + 108-121);② PowerShell 等待路径引号错误:转义单引号却拼进双引号 `-FilePath "{}"`(main.rs:68-71、90-93、123-125),含 `$(...)` 的文件名会被 PowerShell 执行(文件名注入);③ `cmd /c start "" <path>` 中含 &^% 等元字符的路径/URL 未加引号:`open "https://x.com/?a=1&b=2"` 被 cmd 在 & 处截断(main.rs:73、95、101-105);④ -W 对 URL 无效(main.rs:101-106)。
- Windows 合理性:cmd start/powershell Start-Process 可行但脆弱(①②③ 均源于此),更稳的是 ShellExecuteW;关联打开本身符合 Windows 习惯。
- 结论:P1 ① -R 假失败、② PowerShell 注入/引号、③ URL & 截断;P2 ④、改 ShellExecuteW、缺 -t/-b/--args。

#### mkfifo
- 功能:调用 Win32 CreateNamedPipe 在 `\\.\pipe\` 命名空间创建命名管道后立即关闭句柄;-m 打印不支持警告。
- 移植完整性:与声明一致;mkfifo 选项面即窄。
- 技术评估:① P2:管道名仅取 file_name()(main.rs:40-45),不同目录下同名 fifo 冲突且报误导性 "permission denied"(ERROR_ACCESS_DENIED 硬编码,main.rs:61-64);② 管道在进程退出后即销毁,创建的 FIFO 无法被后续进程按 POSIX 语义使用——"Windows 管道语义不等价" 已声明,不算缺陷;③ FILE_FLAG_FIRST_PIPE_INSTANCE 重名检测思路正确。
- Windows 合理性:用 CreateNamedPipeA(ANSI 版,main.rs:48),非 ASCII 文件名乱码,应改 W 版;管道不在文件系统路径出现,属已声明差异。
- 结论:P2 ① 重名误报、② 改 CreateNamedPipeW。

### 第 4 组:文本处理 I(grep / sed / awk / cut / paste / join / comm / cmp / diff)

#### grep
- 功能:按正则递归/非递归搜索单文件、多文件与 stdin,支持 -c/-n/-q/-v/-r/-i/-x/-o,退出码 0/1/2。
- 移植完整性:缺失 -E/-F(-F 固定串完全缺失)、-e 多模式、-w、-l/-L、-h/-H、-A/-B/-C 上下文。正则用 regex crate。
- 技术评估:P0:`grep -c pat 单文件` 无任何输出(main.rs:59 打印总数要求 multiple_files);P1:非 UTF-8 输入 read_line 出错被 unwrap_or(0) 吞掉,静默截断整个文件(main.rs:104);单文件输出仍带 `file:` 前缀(main.rs:117-131);P2:trim_end() 把行尾正常空格/Tab 一并剥掉,影响 $ 锚定与原样输出(main.rs:106);打不开文件不置退出码 2(main.rs:50-53);正则错误 exit 2 正确(main.rs:21-27)。tests/cli.rs 仅测 stdin 与退出码。
- Windows 合理性:UTF-8 假设,GBK/二进制静默截断;trim_end 顺带消 \r 使 CRLF 匹配正确;无 BOM 处理。
- 结论:P0 单文件 -c 无输出;P1 非 UTF-8 截断、单文件前缀;P2 trim_end、文档漏声明 --recursive/--quiet。

#### sed
- 功能:仅支持 `s/旧/新/[g]` 子串替换(非正则),多 -e 顺序执行,-i 带备份后缀。
- 移植完整性:缺失 p、d、y 命令,地址/地址范围,`&` 与 `\1` 反向引用,w/q/a/i/c;s 为纯子串匹配(无 regex 依赖),且 compatibility.md 未声明"子串简化"。
- 技术评估:P1:非 s 命令被静默忽略,`sed '/foo/d'` 原样输出不报错(main.rs:67-69);GNU 最常见调用形式 `sed 's/a/b/' file` 不支持,'s/a/b/' 被当作文件名(cli.rs:20-22、main.rs:123-126);P2:file.file_name().unwrap() 对 `..` panic(main.rs:104);-i 直接 fs::write 覆盖,非临时文件+改名,写一半即损坏(main.rs:111-112);无尾换行文件被追加 \n(main.rs:91-98)。
- Windows 合理性:P1:-i 时 CRLF 被改写为 LF(main.rs:91-98、76-81),原地编辑静默改变行尾;BOM 保留在首行参与匹配;crate 根目录遗留 test_sed..bak 已提交。
- 结论:P1 非 s 命令静默无效、位置参数脚本形式缺失、-i 改写 CRLF、子串简化未声明;P2 unwrap panic、-i 非原子写、无尾换行补齐、清理杂物。

#### awk
- 功能:极简 awk 解释器:BEGIN/END、`/子串/` 模式、print/printf、变量赋值、-v/-F/-f,数值四则与比较的栈式求值。
- 移植完整性:缺失正则匹配(已声明)、else/for、数组、内建函数(length/sub/gsub/index/split)、字符串比较、printf 宽度/精度、重定向。
- 技术评估:P1:if 条件必然错算——条件 token 用 format!("{:?}", t) 拼成 Debug 串再重新 tokenize 求值(main.rs:627, 633-634),数值比较恒假/乱值;字段残留:前一行 $5 泄漏到字段较少的下一行(main.rs:115-120);`print $1, $2` 只取第一个参数、`print 42` 整行丢失(main.rs:521-528);else/for 落入 _ 分支被静默丢弃(main.rs:643-645);P2:非数值字符串模式判为真(awk 应为假,main.rs:374-380);next 只退出当前 action(main.rs:640-642);多文件时 NR 归零、BEGIN/END 重复执行(main.rs:90, 99-104)。
- Windows 合理性:每行 trim_end 剥 \r 及尾随空白,CRLF 可用但 $0 被改;UTF-8 假设,无 BOM。
- 结论:P1 if 求值损坏、字段残留、print 丢失、else/for 静默无效;P2 其余。

#### cut
- 功能:按字段(-d/-f/-s/-o)、字节(-b)、字符(-c)及区间列表截取行。
- 移植完整性:已覆盖 -b/-c/-f/-d/-s/--output-delimiter 及 N-/-M/N-M/逗号列表;缺失 --complement、-n、--zero-terminated。
- 技术评估:P1:`-b` 把原始字节 as char 重编码,UTF-8 多字节文本输出乱码且字节序列错误(main.rs:86);非 UTF-8 输入静默截断(main.rs:44);P2:打不开文件退出码恒 0(main.rs:30);同时给 -f/-b 或都不给不报错(main.rs:52-60);列表不排序不去重(main.rs:101-126);-s 在 -b/-c 模式也生效(main.rs:47);trim_end 剥掉行尾合法空格/Tab(main.rs:45);区间 5-2 静默输出空(main.rs:114)。
- Windows 合理性:CRLF 经 trim_end 处理正确;UTF-8 假设,无 BOM;-d 限单字符。
- 结论:P1 -b 多字节乱码、非 UTF-8 截断;P2 其余。

#### paste
- 功能:并行模式逐行横向合并多文件,串行模式(-s)把单文件各行连成一行,支持 `-` 与 -d。
- 移植完整性:缺失 -d 接受字符列表并按列循环(现把整个串当一个分隔符,main.rs:10, 91, 118)。
- 技术评估:P1:CRLF 行尾 `\r` 未剥离——trim_end_matches('\n') 只去 \n(main.rs:75, 110),Windows 文本文件的 \r 进入输出;任一文件打开失败则整体中止退出 1,不处理其余合法文件(main.rs:38-40);EOF 列按空串处理正确(main.rs:88)。
- Windows 合理性:除 \r 残留外合理;无 BOM。
- 结论:P1 CRLF 残留 \r;P2 -d 列表不循环、单文件失败全盘中止。

#### join
- 功能:以第 -1/-2 字段为键做 hash 内连接(不要求排序),-t 指定分隔符。
- 移植完整性:缺失 -a、-v、-o 输出格式、-e、-j;默认无 -t 时 GNU 按空白切分,此处默认 Tab(main.rs:13),空格分隔文件无法 join。
- 技术评估:P1:输出形状错误——输出 `整行1 + 分隔符 + 整行2`,file2 的键字段重复出现(main.rs:54),GNU/POSIX 应为 `键 + file1 剩余字段 + file2 剩余字段`;P2:`-1 0` 时 field1-1 对 usize 下溢(main.rs:11-12);字段不足的行静默跳过(main.rs:42, 50);crate 根目录遗留 test1-4.txt 已提交。
- Windows 合理性:BufRead::lines 正确剥 \r\n;UTF-8 假设,无 BOM。
- 结论:P1 输出含重复键;P2 字段号 0 下溢、默认分隔符、清理杂物。

#### comm
- 功能:双指针归并比较两个有序文件,三列输出(独有1/独有2/共有),-1/-2/-3 抑制。
- 移植完整性:缺失 --total、-z、--check-order/--nocheck-order。
- 技术评估:未发现明显缺陷:归并逻辑、Err 传播(读错误 exit 1,main.rs:97-104)、重复行处理均正确;P2 手写必填参数校验与 clap 重复(main.rs:17-38)。
- Windows 合理性:lines() 剥 \r\n,CRLF 安全;无 BOM。
- 结论:无需调整(P2 可补 --total/-z)。

#### cmp
- 功能:逐字节比较两文件,首差异即报;-l 列出字节号与值,-s 静默仅用退出码。
- 移植完整性:缺失 -n/--bytes、`-` stdin、--ignore-initial。
- 技术评估:P1:"differ: byte X, line Y" 中行号直接用了字节号(main.rs:93-96),第 25 字节(实际第 3 行)报 "line 25";P2:-l 输出十进制,GNU 为八进制(main.rs:91);缺参数/打不开文件退出 1,GNU 惯例为 2(main.rs:25, 36, 44, 52);EOF 提示缺偏移(main.rs:81-84)。
- Windows 合理性:纯字节比较,对二进制/CRLF/编码天然安全,本组中 Windows 合理性最好。
- 结论:P1 行号错误;P2 进制、退出码。

#### diff
- 功能:LCS 动态规划对两文件做行级 diff,输出普通格式(默认)或统一格式(-u),附 -i/-b/-B。
- 移植完整性:缺失 -q/--brief、-r 递归目录、-c 上下文格式、-w、--strip-trailing-cr、上下文行数可调(-U N)。
- 技术评估:P0:退出码恒 0,文件不同仍 exit 0,破坏脚本/CI 惯例(main.rs:58-62);P0:普通格式行号全错——公共行不入 diffs 且打印循环的 i/j 只在 d/a/c 分支递进(main.rs:115-117, 140-163),任何不始于首行的差异行号全错;P1:统一格式只产出一个巨型 hunk,in_hunk 置 true 后永不复位,无 3 行上下文截断(main.rs:240-267);hunk 头非规范:纯新增输出 `@@ -2 +2,1 @@`,GNU 为 `@@ -1,0 +2 @@`(main.rs:251, 274-281);DP 为 O(n·m) 内存,万行级文件数百 MB(main.rs:83, 170);P2:-b 直接删除全部空白(GNU 是压缩连续空白,main.rs:72-74);无 "\ No newline at end of file" 处理(main.rs:55-56);-B 的启发式可产生错误对齐(main.rs:95-96, 182-183)。
- Windows 合理性:lines() 剥 \r,CRLF 差异被视作相同(合理但与 cmp 字节级语义不一致);非 UTF-8 报错而非 "Binary files differ"。
- 结论:P0 退出码、行号;P1 单 hunk、hunk 头、内存;P2 其余。

### 第 5 组:文本处理 II(sort / uniq / tr / expand / unexpand / fold / fmt / nl / tac / rev)

#### sort
- 功能:读取全部行,支持 -r/-n/-f/-h/-u/-c 排序后输出。
- 移植完整性:缺失 -o、-m、-z、-b、-g/-M、--parallel;-k/-t 虽声明但完全未实现。
- 技术评估:三个 P0:① `-k`/`-t` 被 clap 解析但 main.rs 从未引用,静默无操作,而 compatibility.md:89 声称支持(cli.rs:33-38);② `-u` 在排序前去重,只删相邻重复,`b\na\nb` 排序后仍输出两个 b(main.rs:35-43);③ `-c -r` 的 reverse 分支与非 reverse 分支代码相同(死代码),逆序输入被误报 disorder(main.rs:67-75);另有:非 UTF-8 静默截断且退出码 0(main.rs:58);`-n` 仅整行是合法 f64 时生效,无"前导数字前缀"语义(main.rs:107-109);trim_end 剥掉行尾合法空格/Tab(main.rs:59);`-h` 用 1000 进制且缺 P/E/Z/Y,GNU 为 1024 进制(main.rs:113-118)。
- Windows 合理性:全量读入 Vec<String>,无外部排序,大文件内存受限;文件打开失败退出 0(GNU 为 2);不支持 `-` stdin(main.rs:17-23)。
- 结论:P0 修 -k/-t、-u、-c -r;P1 UTF-8 截断、-n 前缀、trim_end、-h 进制;P2 退出码/`-`/内存/补测试。

#### uniq
- 功能:流式比较相邻行,支持 -c/-d/-u/-i/-f N,计数格式 %7ld 与 GNU 一致(main.rs:94)。
- 移植完整性:缺失 -s/--skip-chars、-w/--check-chars、-z、--all-repeated、--group 及第二操作数(输出文件)。
- 技术评估:P0:prev_line 存的是 make_key 产生的比较键而非原始行,print_line 打印的也是键(main.rs:44-47, 55, 61-62)——`uniq -i` 把输出全部小写化,`uniq -f 1` 把输出截成残行;P1:`-f` 用 split_whitespace().join(" ") 归并空白再比较,GNU 按原样比较剩余部分(main.rs:69-75);P2:-d 与 -u 同给时静默输出空(main.rs:85-91)。
- Windows 合理性:流式内存友好;lines() 剥 \r\n;无 panic。
- 结论:P0 修"打印键而非原行";P1 -f 空白归并;P2 其余。

#### tr
- 功能:字符(UTF-8 char)级转换,支持 a-z 区间与 -d 删除、-s 挤压、-c 补集。
- 移植完整性:明显缺失转义序列(`\n` `\t` 八进制,经典 `tr '\n' ' '` 不可用)、字符类 [:alpha:]、[c*n] 重复语法(main.rs:8-28)。
- 技术评估:两个 P0:① 未提供 SET1 时 expect panic(main.rs:112);② SET2 为空串时 set2[set2_len - 1] 下溢 panic(main.rs:71);P1:`-c` 在替换模式下完全无效——补集只写入 set1_set,替换用的 set1_vec 未取补(main.rs:113-118, 127-132);`-s` 与双集合同用时按 SET1 挤压,GNU 是翻译后按 SET2 挤压(main.rs:134-136);P2:反序区间 z-a 静默生成空集(main.rs:14-25);-c 补集建约 110 万条目 HashSet(main.rs:52-60);全部输出积攒在一个 String(main.rs:121);无换行结尾输入被补 \n(main.rs:139)。
- Windows 合理性:按 char 处理使 UTF-8 不会被截半(优于 GNU 字节语义);内存无界。
- 结论:P0 修双 panic;P1 -c 替换无效、-s 集合错、缺转义/字符类;P2 其余。

#### expand
- 功能:把 Tab 按 tabstop 展开为空格,支持 -t 列表、-i 仅行首、`-` 读 stdin。
- 移植完整性:与声明一致;GNU expand 选项面本就小,无缺失。
- 技术评估:未发现 panic 级缺陷;P2:`-i` 下未转换的 Tab 走 col += 1,GNU 完全不推进列(main.rs:123-131);无换行结尾输入补 \n(main.rs:134);-t 0 报 invalid(GNU 允许 0 表示不展开,main.rs:61-66);列按 char 计非显示宽度。
- Windows 合理性:流式,CRLF 归一,`-`/多文件/错误退出码 1 均正确(main.rs:23-53)。
- 结论:无需调整(低危 P2)。

#### unexpand
- 功能:把空白串压缩为 Tab,默认仅行首,-a 全行,-t 单一 tabstop。
- 移植完整性:`-t` 不接受列表(cli.rs:15)、无 --first-only。
- 技术评估:P1:单个空格恰好落在 tab stop 上也被替换为 \t——space_run >= 1 恒真(main.rs:65),GNU 要求至少两个空白才换 Tab;P2:-t 0 静默按 8 处理(main.rs:16);无尾换行补 \n(main.rs:95)。
- Windows 合理性:流式;错误退出码正确(main.rs:18-48)。
- 结论:P1 修单空格换 Tab;P2 其余。

#### fold
- 功能:按宽度折行,-w(默认 80)、-b 按字节、-s 空白处断。
- 移植完整性:GNU fold 选项面即此三项,无缺口。
- 技术评估:P1:非 UTF-8 输入在 read_line 处直接报错退出(main.rs:56),-b 的主要用途(二进制/未知编码)完全不可用;-b 模式对合法 UTF-8 按字节切分后 from_utf8_lossy 输出 U+FFFD,内容被破坏(main.rs:150-151);P2:列按 char 计非显示宽度;-s 可能把行尾 \r 单独切成一段;宽度 0 静默不折叠(main.rs:81-84)。
- Windows 合理性:流式;CRLF 保留(trim_end_matches('\n'),main.rs:61-66),是本组少数不破坏 CRLF 的;退出码正确。
- 结论:P1 非 UTF-8 不可用、-b 破坏多字节;P2 其余。

#### fmt
- 功能:段落重排:按空行分段,词流重排到 -w(默认 75)宽度,支持 -s 仅拆分。
- 移植完整性:缺失 -u、-p、-t、-g。
- 技术评估:三个 P1:① `-c/--crown-margin` 被实现为必须带值的 Option<usize>,GNU 的 `fmt -c`(无值)在 clap 处直接报错,且语义与 crown margin 无关(cli.rs:12-13, main.rs:97-98);② args.width - first_line_indent 在 crown > width 时 usize 下溢,release 静默不换行(main.rs:104-108);③ --split-only 按字符边界硬切、不在空白处断词,单词被拦腰截断(main.rs:162-174);P2:行宽用字节长度,CJK 提前折行(main.rs:112);词间一律单空格。
- Windows 合理性:流式;多文件与 `-` 处理正确(main.rs:17-40)。
- 结论:P1 -c 形态与语义、宽度下溢、-s 断词;P2 其余。

#### nl
- 功能:按 -b(all/nonblank)给行编号,-w 宽度、-s 分隔符。
- 移植完整性:缺失 -n(rn/rz/ln 格式)、-v 起始号、-i 增量、-l、-d、-h/-f、pRE 样式。
- 技术评估:P0:line_content.push(byte as char) 逐字节 Latin-1 转字符,非 ASCII 输入全部乱码(main.rs:116);P1:`-b n` 被映射为 nonblank,而 GNU 样式 n=none(不编号)、t 才是非空行,语义完全错位(cli.rs:22-26);默认 body numbering 为 all,GNU/POSIX 默认 t(main.rs:39);P2:全文件与全输出各缓存一份内存(main.rs:58-61);无条件丢弃所有 \r 字节(main.rs:72-75)。
- Windows 合理性:CRLF 归一方式是丢 \r;一次性 read_to_end,大文件不友好(main.rs:59)。
- 结论:P0 修 byte-as-char 乱码;P1 -b n 语义错位、默认值;P2 其余。

#### tac
- 功能:按分隔符(默认 \n,-s 多字符、-b 前置)切记录后逆序输出。
- 移植完整性:GNU tac 选项面即此;但核心记录语义有缺陷。
- 技术评估:两个 P0:① 输入以分隔符结尾时循环外总再 push 一个空尾段(main.rs:114-116),`a\nb\n` 输出 `\nb\na\n`(GNU 为 `b\na\n`),空输入输出一个 \n;② 分隔符是"每段后统一补发"而非随记录携带,无换行结尾输入 `a\nb` 输出 `b\na\n`(GNU 为 `ba\n`)(main.rs:105-116, 78-86);P2:多文件时所有文件的段混入同一个 Vec 后整体逆序,GNU 按文件分别逆序(main.rs:22, 32-54);read_to_string 拒绝非 UTF-8(main.rs:99);全量 Vec<char> 驻留内存(main.rs:100)。
- Windows 合理性:CRLF 保留(按 \n 切分,\r 留在段尾),对 Windows 文件反而正确;内存无界。
- 结论:P0 修空尾段与分隔符重挂语义;P2 其余。

#### rev
- 功能:逐行反转字符后输出,支持文件列表与 `-` 读 stdin。
- 移植完整性:GNU rev 无功能选项,已完整覆盖。
- 技术评估:未发现明显缺陷:反转逻辑正确(main.rs:61-67);CRLF 输入产生 `\rcba\n` 与 GNU 按字节反转一致;chars() 反转对 UTF-8 标量安全。唯一 P2:非 UTF-8 输入报错退出(main.rs:56)。
- Windows 合理性:流式内存友好;行尾与 GNU 一致;退出码正确。
- 结论:无需调整(P2 非 UTF-8;crate 根目录遗留 test.txt 应清理)。

### 第 6 组:文本处理 III(wc / head / tail / tee / split / csplit / column / pr / shuf)

#### wc
- 功能:统计行数/词数/字节数,可选 -m 字符数与 -L 最大行长,多文件时输出 total。
- 移植完整性:声明与代码一致;stdin 附 `<stdin>` 标签为已声明差异;缺 --files0-from/-0、--total。
- 技术评估:三个真实缺陷:① P0:`-m` 是伪实现,counts.chars += 1 在字节循环里逐字节累加(main.rs:117-118),UTF-8 多字节文件 -m 与 -c 恒相等;② P0:`-L` 把换行符计入行长(main.rs:119, 123),`ab\n` 报 3 而 GNU 报 2;③ P1:打不开文件仅 continue,进程仍退出 0(main.rs:55-58);读错误被 Err(_) => break 静默吞掉(main.rs:134)。
- Windows 合理性:纯字节统计对 ASCII 正确;Windows 常见的 UTF-16LE/GBK 文件 -m 全错。
- 结论:P0 -m、-L;P1 文件失败 exit 0;P2 读错误静默。

#### head
- 功能:输出前 N 行或前 N 字节,多文件时 `==> name <==` 头及 -q/-v。
- 移植完整性:缺失 GNU 负值语义(`-n -5` 输出除末尾外全部)、-z。
- 技术评估:P1:行模式用 reader.lines()+writeln(main.rs:79-84):CRLF 被改写为 LF、末行无换行补 \n、非 UTF-8 报错中断(GNU 按字节透传);P2:`-c` 一次性分配 vec![0u8; bytes](main.rs:67),巨值直接 OOM;`-n` 与 `-c` 同给不报错(main.rs:65);`-v` 对 stdin 不打印头(main.rs:13)。
- Windows 合理性:`-c` 模式对二进制/UTF-16 安全;行模式 CRLF→LF 改写是实际数据改动。
- 结论:P1 行模式语义;P2 预分配、互斥、-v。

#### tail
- 功能:输出末尾 N 行或 N 字节,-f 以 100ms 轮询追加并处理收缩。
- 移植完整性:缺 stdin/管道支持(main.rs:12-15 直接报 missing file operand,`cat x | tail` 失败)、`-n +N`/`-c +N`、-c 与 -f 组合(follow 在字节分支被忽略,main.rs:50-70)、-F/--retry/-s/-z。
- 技术评估:① P0:`tail -n 0 -f` 不跳到文件尾,先吐出整个文件再 follow(main.rs:74-78);② P0:多文件 -f 时第一个文件的 follow_file 永不返回(main.rs:30-38 + 120-151),后续文件永远不输出;③ 行模式 lines() 全量载入内存(main.rs:82-87),CRLF→LF、末行补 \n、非 UTF-8 报错(main.rs:84-98),follow 段输出原始字节与前段风格混用;`-c 0` 语义碰巧正确。
- Windows 合理性:100ms 轮询 + 收缩回卷检测(main.rs:135, 140-143)是 Windows 上合理可行的 -f 实现;Rust std 默认 FILE_SHARE_DELETE 使日志轮转通常不阻塞。
- 结论:P0 无 stdin、-n 0 -f;P1 -c -f、多文件 -f、行模式语义、-n +N;P2 内存。

#### tee
- 功能:读 stdin,逐行写到 stdout 与全部目标文件,-a 追加、-i 忽略中断,出错置退出码 1。
- 移植完整性:缺 -p/--output-error。
- 技术评估:P0:逐行处理而非字节透传——reader.lines() + writeln(main.rs:48-61),CRLF 被统一改写为 LF、末行补 \n、二进制/非 UTF-8 触发 read error 提前退出(main.rs:63-67),GNU tee 是字节透明的;`-i` 在 Windows 为空操作且 cfg(unix) 块引用了未声明的 libc 依赖(main.rs:13-22),该 crate 无法在 unix 目标编译(本项目仅 Windows 构建故未暴露)。
- Windows 合理性:追加/截断用 OpenOptions 正确表达(main.rs:30-35);crate 根目录残留 test_tee.txt 已提交。
- 结论:P0 非字节透明;P2 -i no-op + 缺失依赖、清理杂物。

#### split
- 功能:按行数(-l)、字节数(-b,支持 k/M/G)或近似 -n 均分切成多个文件。
- 移植完整性:GNU 的位置参数前缀(`split f.txt out`)不支持(cli.rs:33 用 -o 代替,已声明);stdin 不支持(cli.rs:37 required);缺 -C/--line-bytes、-t、-e、--filter。
- 技术评估:① P0:字母后缀生成错误——generate_suffix 对 index 0-25 得到 "a".."z" 后用 {:0>width$} 以 '0' 左填充(main.rs:96-109),默认输出 x0a..x0z 而 GNU 为 xaa..xaz,前 26 个文件名错误且整体错位;后缀超宽时不报 "output file suffixes exhausted" 而是静默加长(main.rs:109);② P0:行模式 lines()+writeln(main.rs:166-180):CRLF→LF、末行补 \n、非 UTF-8 中途报错留下半成品文件;③ P1:`-l 0` 不报错,退化为每行一个文件(main.rs:183);④ parse_size 乘法溢出无检查(main.rs:88)、缺 T/P/KB 后缀(main.rs:79-83);⑤ `-b` 巨值全量预分配(main.rs:122);⑥ `-n` 用 ceil 字节切块近似(main.rs:209-210)。
- Windows 合理性:生成文件名合法;crate 根目录残留 test_split.txt(带 BOM)。
- 结论:P0 后缀序列、行模式非字节透明;P1 -l 0、后缀耗尽;P2 其余。

#### csplit
- 功能:按 `/子串/` 行号或非标准 `模式N` 重复切分为 xx00、xx01…,输出字节数,支持 -k/-z/-s/-f/-n。
- 移植完整性:声明的选项全部存在,但 --suffix-format 被接受后完全忽略(cli.rs:18, format_filename 只用 digits,main.rs:184-186);GNU 模式语法大面积缺失:正则退化为字面子串(main.rs:175-182)、`{N}` 重复完全不支持(main.rs:59-84)、`%re%` 跳过不支持、`/re/+N|-N` 偏移不支持。
- 技术评估:① P0:重复路径每次迭代 cur = end + 1(main.rs:46-52)把匹配行本身从所有输出文件中丢掉(非重复路径 main.rs:62-64 正确保留),同程序内语义自相矛盾且有数据丢失;② P0:自造的 `模式N` 尾数被 try_repeat_tail 当作重复次数(main.rs:150-162),GNU 的 `/re/+3` 偏移在此解析失败后静默 break(main.rs:40-43),整个模式无声无息不产生任何切分;③ P1:模式不匹配时 GNU 报错退出 1(无 -k 时删文件),这里静默把剩余内容写入文件并退出 0(main.rs:66-70);行号越界同样静默(main.rs:72-79);--suffix-format 无效但仍声明(main.rs:184-186);④ P2:全量读入 Vec<String>(main.rs:141-146);字节统计按剥离后的行 len()+1(main.rs:103-106),与磁盘实际字节不符。
- Windows 合理性:xxNN 文件名合法。
- 结论:P0 丢匹配行、+N 静默失效;P1 无匹配 exit 0、--suffix-format 假声明;P2 其余。

#### column
- 功能:-t 表格模式按分隔符切列、按最大显示宽对齐,支持 -s/-o/-R/-H;多文件与 "-" 均可。
- 移植完整性:GNU 默认(无 -t)的 fill 模式——按终端宽度把词条折成多列——完全未实现(main.rs:54-62, 100-116);缺 -x、-c。
- 技术评估:① P0:`-t` 且未显式给 -s 时(默认 sep=" \t")不折叠连续空白:split_by_any 对每个分隔字符都切分(main.rs:121-139),`a  b` 被切成 ["a","","b"],与 GNU 把连续空白视为单一分隔符不符,默认用法列即错位;② P1:默认 fill 模式缺失;③ P2:最后一列补齐尾随空格(main.rs:162-177);display_width 把所有非 ASCII 一律按 2 计(main.rs:148-152),西文重音字符宽度错。
- Windows 合理性:宽度按"非 ASCII 即 2"近似,对 CJK 表格可用。
- 结论:P0 连续空白不折叠;P1 fill 模式缺失;P2 其余。

#### pr
- 功能:为文本分页,默认 66 行/页,5 行页眉,支持 -d/-l/-o/-W/-h/-f/-n/-t/-2。
- 移植完整性:缺 -m(合并多文件)、-N、-s/-S、-J。
- 技术评估:① P0:页眉日期是手搓估算式:y=1970+secs/31536000(忽略闰年)、月按 30 天、日按 %30(main.rs:127-136),系统性偏差且无时分;② P0:`-n` 解析与 GNU 相反:按"先数字宽度、余下全为分隔符"解析(main.rs:145-152),`-n:5` 变成分隔符 ":5";number_lines 是必需取值的 Option(cli.rs:33),裸 `pr -n` 直接 clap 报错,`pr -n 文件` 会把文件名吞成分隔符;③ P2:死代码空循环 for _ in 0..=page_no.max(1)(main.rs:228-230);页眉布局与 GNU 不同(文件名单独居中一行 + ==== 划线,main.rs:244-251);双栏补空格用字节长度(main.rs:184)而截断按显示宽,非 ASCII 错位;表头居中按字节长(main.rs:257-264)。
- Windows 合理性:分页/换页符 \x0C 正常;日期应换 chrono;行读取走 lines(),继承 CRLF/非 UTF-8 问题。
- 结论:P0 页眉日期、裸 -n;P2 其余。

#### shuf
- 功能:将 -i 范围或文件/stdin 的行随机洗牌后输出,支持 -n、-o、--random-source、多文件拼接。
- 移植完整性:`-r/--repeat` 完全缺失(cli.rs 无该字段),`-e/--echo`、`-z` 也缺。
- 技术评估:P0:`--random-source` 语义错误——GNU 从该路径文件读随机字节,这里把参数字符串本身 FNV 哈希成种子(main.rs:16-17, 116-123),`shuf --random-source=/dev/urandom` 每次运行结果完全相同;P1:-r 缺失;P2:`-i` 负数下界不支持(main.rs:109-114);`-e`/-z 缺失;无 -r 时 Fisher-Yates 结构正确,(s>>8)%range 有轻微模偏差;多文件拼接比 GNU 更宽松(main.rs:66-77)。
- Windows 合理性:种子用纳秒+进程 ID(main.rs:19-23)足够。
- 结论:P0 --random-source 确定性;P1 -r 缺失;P2 其余。

本组总结:9 个命令均与 compatibility.md 声明的选项清单一致(仅 csplit 的 --suffix-format 是"接受但无效"的假声明);系统性共性问题是以 BufRead::lines() 为核心的行处理(CRLF→LF、无尾换行补齐、非 UTF-8 中断)波及 head/tail/tee/split/csplit/column/pr/shuf。

### 第 7 组:编码校验与数值格式(base64 / md5sum / sha256sum / od / hexdump / strings / iconv / numfmt / bc / factor)

#### base64
- 功能:自实现标准 Base64 编码/解码,-w 换行(默认 76,与 GNU 一致)、-d 解码、stdin/文件输入。
- 移植完整性:缺失 -i/--ignore-garbage;GNU 只接受至多一个 FILE,此处多文件按字节流拼接(main.rs:22-41)。
- 技术评估:**P0:解码对任何含 = 填充或长度非 4 倍数的输入输出错误字节**——= 分支只加 bits 不移位 acc(main.rs:135-154),尾部提取按 24 位左对齐取 acc>>16(main.rs:178-193)。已按源码逐行模拟验证:`"QQ=="` → 0x00(应为 "A")、`"SGVsbG8="` → `"Hel\x01±"`(应为 "Hello"),仅长度恰为 4 倍数且无填充时正确,base64 -d 与自身 -e 的往返失败。P2:非法字符时写中途 exit(1)(main.rs:159-160);全量 read_to_end(main.rs:61、71)。编码路径未发现问题。
- Windows 合理性:纯字节处理,无 CRLF/编码影响。
- 结论:**P0 解码错误**;P2 其余。

#### md5sum
- 功能:md5 crate 流式(8KB)计算 MD5,-c 校验文件,-q 静默。
- 移植完整性:缺失 --tag、--status、--strict、-z、-b/-t。
- 技术评估:P1:check 模式不识别 `*` 二进制标记——只剥前导 `\`(main.rs:103-107),split_whitespace 后 `*file` 直接当路径打开,GNU -b 产物或常见 Windows 校验清单整文件 FAILED;P2:读错误静默当 EOF(main.rs:69);畸形行静默跳过(main.rs:96-98);FAILED 打到 stderr(main.rs:120);crate 根残留 test.txt、md5sum.txt。BOM 处理正确(main.rs:84)。
- Windows 合理性:主动剥 UTF-8 BOM 是好适配;流式、二进制安全。
- 结论:P1 check 不识别 `*` 标记;P2 其余。

#### sha256sum
- 功能:sha2 0.10 流式计算 SHA256,-c 校验、-q。
- 移植完整性:同 md5sum 的缺失;且普通模式不支持 `-` stdin(main.rs:25-37,md5sum 支持,行为不一致)。
- 技术评估:P1 三处:① check 不识别 `*` 标记(main.rs:77-84);② **不剥 BOM——自带的 checksum2.txt(EF BB BF 开头)用 sha256sum -c 报 FAILED**(main.rs:62-77;md5sum 已有对应处理);③ 任一文件 mismatch 即 std::process::exit(1)(main.rs:107-109),其余校验文件被跳过;P2:读错误 unwrap_or(0) 静默截断(main.rs:48);畸形行静默跳过(main.rs:73-75)。
- Windows 合理性:BOM 问题正是 Windows(记事本)典型产物,互操作风险实际存在。
- 结论:P1 `*` 标记、BOM、提前退出;P2 其余。

#### od
- 功能:按地址 + 多进制词格式 dump,支持 -A/-t/-c/-d/-o/-x/-j/-N/-w。
- 移植完整性:默认输出与 GNU 一致;**`-t x1/d1/o1`(最常用格式)实际不可用**;缺重复行折叠(`*`)与 -v、-j/-N 的 b/K/M/0x 后缀、多文件拼接。
- 技术评估:P1 三处:① `-t x1` 等单字节规格静默落到 2 字节(cli.rs:112-117 `_ => WordSize::TwoBytes`);② `-t f` 为假实现,Float 直接按字节打十六进制(main.rs:116, 197-203);③ 所有数值模式永远追加 ASCII 列(main.rs:262-278),GNU 的 -t x1/-t o2 无此列;P2:`-t ''` 时 chars().next().unwrap() panic(cli.rs:91);未知格式字符静默回退 octal(cli.rs:98);`-v` 被版本占用、与 GNU od -v 冲突(cli.rs:60-61);全量 read_to_end(main.rs:216-217)。
- Windows 合理性:二进制安全。
- 结论:P1 -t x1 尺寸、-t f 假实现、恒附加 ASCII 列;P2 其余。

#### hexdump
- 功能:BSD hexdump 风格 dump,-C canonical、-d/-o/-x/-c 四种模式、-s/-n 截取。
- 移植完整性:默认格式与 BSD 一致;-C 格式正确;缺 -b、-e/-f 格式串、重复行折叠。
- 技术评估:未发现影响常规输入的缺陷;P2:start + len 可溢出回绕后 &buffer[start..end] panic(main.rs:47-53,需极端参数);全量读入(main.rs:15-43)。
- Windows 合理性:二进制安全。
- 结论:基本无需调整(可选加固)。

#### strings
- 功能:按最小长度(默认 4)提取可打印串,支持 -e 六种编码、-t 偏移、-f 文件名前缀。
- 移植完整性:缺 -o(-t o 别名,高频用法)、-w;不解析 ELF/PE 段表(-a 为 no-op,可接受)。
- 技术评估:未发现 panic;P2:`-n 0` 时每字节都触发空行输出(main.rs:136-165);-f 打印 basename 而非原样路径(main.rs:32-35);8-bit 可打印上限 0xFE 不含 0xFF(main.rs:84-86);全量 read_to_end(main.rs:97)。UTF-16/32 的代理项处理正确(main.rs:381-394)。
- Windows 合理性:UTF-16LE 提取对 Windows Unicode 二进制实用。
- 结论:基本无需调整(P2 补 -o 等)。

#### iconv
- 功能:基于 encoding_rs 0.8 全量解码→编码,-f/-t/-o/-l。
- 移植完整性:--list 只硬编码 28 项(main.rs:92-127);缺 //TRANSLIT、-c、--byte-subst。
- 技术评估:P1:**Encoding::decode 带 BOM 嗅探**(main.rs:74),`-f gbk` 读到 UTF-8 BOM 文件时静默按 UTF-8 解码,违背 iconv"严格按 -f 指定编码"语义(应用 decode_with_bom_removal);P2:encode 对不可映射字符静默输出 &#NNNN;(main.rs:75);多文件拼接后单次解码,状态型编码跨界出错;读文件失败仍继续。
- Windows 合理性:中文场景良好——GBK、GB2312、GB18030、Big5 均可用;本组中 Windows 适配最好的之一。
- 结论:P1 BOM 嗅探覆盖 -f;P2 其余。

#### numfmt
- 功能:数值与 SI/IEC 后缀串互转,--from/--to/--round/--padding/--suffix/--grouping/--header/-d/--field。
- 移植完整性:缺 --format、-z、--invalid、--debug;SI 后缀缺 Y(main.rs:10);--to=auto 是 GNU 不存在的超集选项(main.rs:213-219)。
- 技术评估:P1 两处:① 自定义分隔符输出被硬编码空格替换——joined.push(' ')(main.rs:63),`-d ,` 输入 `a,4096` 输出 `a 4.1K`;且 split 对显式 -d 仍折叠连续分隔符(main.rs:125-136);② --from=auto 把 1K 按 1024 解析(main.rs:162, 171-196),GNU 语义为 1K=1000、1Ki=1024;P2:--round 默认 up(cli.rs:46,GNU 默认 from-zero);无效输入回显原行 exit 1(main.rs:43-50),GNU 默认立即 exit 2;首参数"路径存在即当文件"启发式(main.rs:22)会把同名数字误读为文件。
- Windows 合理性:lines() 剥 CRLF 正常;grouping 固定 , 可接受。
- 结论:P1 分隔符输出、from=auto 基数;P2 其余。

#### bc
- 功能:号称 bc,交互 banner + 逐行"求值"。
- 移植完整性:与 GNU bc 差距最大:f64 实现,非任意精度;无 scale(main.rs:63-65 读到即静默丢弃)、无 ibase/obase、无 sqrt/函数/define/if/while/数组、无 -l 数学库;仅支持 + - * / ^ 与变量赋值。
- 技术评估:**P0:任何含运算符的表达式恒输出 0**——process_line 先删除全部空格(main.rs:62),eval_expr 只按空白分词(main.rs:81-82),`1+2` 成单 token 落入 _ 分支(main.rs:106),RPN 求值 parse 失败落运算符分支无匹配臂(main.rs:116-130),栈空返回 0(main.rs:134)。已模拟验证:"1+2"→0、"2+3*4"→0、"10/4"→0,仅裸数字正确。P1:f64 精度——`10^30` 输出 1000000000000000019884624838656,`1/3` 输出 0.3333333333333333(GNU 默认 scale=0 应为 0);赋值会打印值(main.rs:72);P2:banner 伪装 bc 1.07.1(main.rs:23)而 --version 报 0.1.0;文件模式出错仍 exit 0(main.rs:13-20)。
- Windows 合理性:无平台问题(问题全在算法层)。
- 结论:**P0 表达式求值失效,需重写求值器**;P1 f64 非任意精度 + scale/ibase 缺失;P2 其余。

#### factor
- 功能:试除法质因数分解(u64),输出格式与 GNU 一致,支持 stdin 与参数。
- 移植完整性:缺失大数支持——GNU factor 借 GMP+Pollard-Brent 可分解至 2^128,此处仅 u64,`factor 18446744073709551617` 报错(main.rs:51-53);缺 --exponents。
- 技术评估:未发现 panic/溢出(saturating_mul 保护到位,main.rs:68-73);stdin 每行只接受一个数(main.rs:19-36),GNU 按空白分隔;64 位大素数试除最坏秒级耗时(main.rs:67-74)。
- Windows 合理性:纯数值,无平台问题。
- 结论:P1 仅 u64 无大数;P2 一行一数、性能。可选增强,非正确性问题。

### 第 8 组:归档压缩(tar / gunzip / zcat / zip / unzip / shred / sync)

依赖:tar 0.4.46 + flate2(tar/gzip),zip 0.6.6(unzip);zip 与 sync 是外部程序外壳。

#### tar
- 功能:tar 的创建(-c)/解包(-x)/列表(-t),可选 gzip(-z),-f 必填,-v 输出条目名。
- 移植完整性:缺 -C/--directory、-O/--to-stdout、--exclude、--strip-components、-j/-J、-r/--append、-p、`-f -`、按成员名解包/列表(clap 接受 FILES 但 extract/list 完全忽略,main.rs:23 是唯一使用点)。
- 技术评估:**P0:解包无路径清洗**——entry.unpack(&path)(main.rs:71, 77)直接用条目原始路径;tar crate 的 `..`/绝对路径防护只在 unpack_in,unpack 不校验——`../../x`、`C:\evil` 条目可写出 cwd,GNU tar 默认剥前导 / 并拒绝 ..;P1:创建时 -f 归档位于待打包目录内会边写边读自身(main.rs:11 + 35);缺文件仅 eprintln 后 continue 退出码 0(main.rs:26-28);P2:-tv 的 mtime 输出裸 epoch 数(main.rs:109-112);创建硬编码 mode 0o644(main.rs:41)。
- Windows 合理性:tar crate 在 Windows 下把条目名 `\` 规范化为 `/`,归档跨平台;符号链接条目需特权,失败即整体报错(main.rs:77)。
- 结论:P0 路径穿越;P1 缺 -C/--exclude/-O/成员选择;P2 其余。

#### gunzip
- 功能:单文件解压:强制 .gz 扩展名校验,解压为去 .gz 同名文件,默认删除源(-k 保留),-f 覆盖,-v 进度。
- 移植完整性:缺 -c/--stdout、多文件输入、-l/--list、-t/--test、.tgz 识别、解压后未按 gzip 头恢复 mtime。
- 技术评估:P2:仅解单 gzip member——flate2 GzDecoder 只处理第一个 member,多成员流静默截断(main.rs:44);损坏流中途报错时已创建的目标文件残留(main.rs:45-47);错误信息为中文(main.rs:16, 21, 28, 34),与全仓 GNU 风格不一致。
- Windows 合理性:输出名处理无分隔符问题。
- 结论:P2 缺 -c/-l/-t、单成员截断、中文错误信息。

#### zcat
- 功能:等价 gzip -dc:若干文件(无参数时 stdin)解压写 stdout,多文件顺序输出,不删源。
- 移植完整性:核心语义覆盖;缺 -t、多成员流、GNU 风格带文件名的错误前缀。
- 技术评估:P2:单成员限制同 gunzip(main.rs:12, 19);多文件时某个失败立即 exit 1,剩余不再处理(main.rs:39-44);错误信息不含文件名(main.rs:35, 41)。
- Windows 合理性:纯流式 stdout,与 gunzip 职责清晰。
- 结论:无需调整(P2)。

#### zip
- 功能:纯外壳:powershell -Command "Compress-Archive ... -Force",无任何原生 zip 实现。
- 移植完整性:仅 -r;缺 -e/-P 加密、-u、-d、-j、-x、-0/-9;固定 -Force 无条件整体覆盖(main.rs:23)。
- 技术评估:**P0:命令注入/含空格即坏**——文件名与目标名未经任何引用转义直接 format! 进 PowerShell 命令(main.rs:19-28),`zip out.zip "my file.txt"` 直接失败;含 `;`、`$()` 的文件名可执行任意 PowerShell 代码;P1:PowerShell 5.1 的 ZipArchive 条目名使用 `\` 分隔而非 ZIP 规范的 `/`(Microsoft 官方文档),生成的归档在 Linux/Info-ZIP 解压时目录结构破坏,与自带 unzip 往返不兼容;P2:非 UTF-8 路径被静默替换为空串(main.rs:13-15);compatibility.md 未像 sync 那样披露外壳实现。
- Windows 合理性:固定调 Windows PowerShell 5.1,跨平台归档可移植性差,建议原生实现。
- 结论:P0 注入/空格;P1 条目名不可移植、建议原生实现;P2 其余。

#### unzip
- 功能:基于 zip 0.6.6 原生解包(-d)与 -l 列表,-o 覆盖,-q 静默;默认遇已存在文件跳过。
- 移植完整性:缺成员选择与 -x、-j、-p、-t、-P、-n;解压不恢复条目 mtime。
- 技术评估:**P0:zip slip**——dest_path.join(file.name())(main.rs:58)使用原始条目名,未用 zip 0.6.6 专为此提供的 enclosed_name/mangled_name;`../../x` 逃逸目标目录,绝对路径条目因 join 替换语义写到任意位置;P1:read_to_end 全量入内存,大文件/zip bomb 内存放大(main.rs:84-86);P2:损坏流整体退出并残留部分文件;覆盖只读文件失败即整体报错。
- Windows 合理性:条目名解码后 Windows Path 可用;不处理 Unix symlink 扩展条目可接受。
- 结论:P0 zip slip;P1 内存放大;P2 其余。

#### shred
- 功能:对每个文件覆写 iterations 次:前 n-1 次写由迭代序号推导的固定单字节模式,最后一次全 0,随后 sync_all,-u 删除。
- 移植完整性:缺 -s/--size;-z 应为可选(此处末次恒 0)、-x;无 GNU 的改名/截断元数据清理;GNU 默认为随机数据覆写(此处模式确定可预测,main.rs:51)。
- 技术评估:P1:unwrap panic 链——seek().unwrap()(main.rs:45)、两次 write_all().unwrap()(:49, :53)、sync_all().unwrap()(:57),磁盘满/权限丢失/占用都会 panic;P2:--force 声明但 main.rs 全文未使用,纯 no-op;按文件大小全量分配内存(vec![0u8; size],main.rs:48, 52);失败时退出码恒 0(main.rs:11-14);-n1 时唯一一次覆写是全 0 而非随机(:47-49);remove_file 失败被 unwrap_or_default 静默吞掉(:31, :64)。
- Windows 合理性:原地同尺寸覆写 + sync_all 在 NTFS 上只是"尽力而为"(日志、VSS、SSD FTL),文档已覆盖该局限。
- 结论:P1 unwrap panic 链;P2 --force no-op、内存、退出码、确定性模式。

#### sync
- 功能:外壳:执行外部 sync 并转发;外部不存在时回退 rundll32 advapi32.dll,ProcessIdleTasks。
- 移植完整性:GNU 的 FILE 参数与 -f/-d 全部缺失(文档只声明 --version,一致)。
- 技术评估:P1:**PATH 自递归**——main.rs:10 Command::new("sync") 按 PATH 解析,而本仓库自身就产出 sync.exe;安装目录进 PATH 后会递归拉起自身直至资源耗尽;P2:未检查外部命令退出码,恒 exit 0(main.rs:13-20);ProcessIdleTasks 不等于 flush 文件系统缓冲,语义名不符实(main.rs:22-24)。
- Windows 合理性:外壳是已声明的有意差异,但自递归风险未被声明覆盖;Windows 原生正解是 FlushFileBuffers。
- 结论:P1 PATH 自递归;P2 回退语义、退出码。

### 第 9 组:进程管理(ps / kill / pkill / pgrep / nice / timeout)

#### ps(ps-sys + ps-core + ps-bin 合并)
- 功能:枚举全系统进程并以 procps 风格列(PID/TTY/TIME/COMM 默认,-f/-F/-l/-j/-o/--sort/--forest/-p/-u/-C 等)渲染;ps-sys 负责 Win32 FFI(EnumProcesses+Toolhelp 快照、PEB 读 cmdline、token/内存/线程/时间),ps-core 做数据模型/过滤/排序/树,ps-bin 为 clap 入口,三层职责清晰,分层合理。
- 移植完整性:覆盖 -e/-A/-a/-x、-f/-F/-l/-j、-o、-p、-u/-U、-C、--sort、--forest、--no-headers、--no-cmdline;缺失 `ps aux`(BSD 连写形式,clap 无 positional 定义直接报错)、-o %cpu/%mem 别名、-o pid=NEWNAME 改表头、--ppid/-t/-N;**README 宣称的 sz/psr/wchan/cputime 列在 ps-core/src/columns/mod.rs:46-91 注册表中不存在**,导致 ps -F/ps -l 每次运行都向 stderr 打 "unknown column" 警告并丢列(ps-bin/src/cli.rs:193, 202 + main.rs:110-116)。
- 技术评估:P0:pmem 硬编码总内存 8GB——(working_set/8.0e9)*100(ps-core/src/columns/pmem.rs:20),16/32/64GB 机器上 %MEM 静默偏小 2-4 倍;P1:pcpu/c/cp 三列恒 "?":均要求 ctx.prev 采样,而 render.rs:12-15、sort.rs:50-53、filter.rs:76-80 永远传 prev: None,make_sample 是死代码;stat 恒 "R":assemble.rs:64 硬编码 Running,README 声称由 GetExitCodeProcess+主窗口合成,文档与代码不符;token.rs 两条错误路径泄漏句柄(ps-sys/src/token.rs:31, 41-48);P2:--headers-repeat 已声明但从未使用;filter.rs:32-59 完全忽略 include_all/include_no_ttys,-a/-x 与 -e 无差别;裸 ps 不过滤列出全部进程(GNU 为"本人+tty");-C 是子串匹配而 procps 为精确(filter.rs:47-57);排序按格式化字符串再 parse::<u64>(sort.rs:77-83),浮点列与负值按字典序排错;stime/start 用 UTC(stime.rs:8-17)。
- Windows 合理性:EnumProcesses 缓冲区翻倍重试正确(snapshot.rs:34-54);Toolhelp 句柄全路径关闭;OpenProcess 用 PROCESS_QUERY_LIMITED_INFORMATION 恰当;PEB 读 cmdline 全路径 CloseHandle;TOCTOU 处理得当(先快照后逐个 OpenProcess,进程退出→"?")。整体 Win32 用法正确,问题集中在 ps-core 数据合成层。
- 结论:P0 pmem 硬编码;P1 pcpu/c/cp、stat 恒 R、未注册列警告、句柄泄漏;P2 其余。

#### kill
- 功能:按 PID 打开进程并 TerminateProcess(-s 选信号,-l 列信号)。
- 移植完整性:缺 `kill -9 PID`/`kill -SIGNAME PID` 连写形式(clap 把 "-9" 当未知短旗标报错)、-n、`kill -l <信号>` 互转、SIG 前缀/数字/大小写归一化;-l 仅列 TERM/KILL 两项(main.rs:30-34)。
- 技术评估:P1:退出码恒为 0——main.rs:22-27 失败只打印而不返回非零;信号串完全不校验:main.rs:46-49 仅 "KILL" => 9, _ => 15,拼错的信号一律静默按 TERM 处理;P2:pids 可接受负值。句柄无泄漏(main.rs:40-53)。
- Windows 合理性:OpenProcess(PROCESS_TERMINATE)+TerminateProcess+CloseHandle 用法正确;TERM/KILL 都映射 TerminateProcess 是合理 Windows 语义。
- 结论:P1 失败退出码、信号串校验、-9 连写;P2 -l 扩展。

#### pkill
- 功能:用 tasklist 枚举进程,按名称子串(或 -f 时命令行子串)匹配后逐个 taskkill /PID x /F。
- 移植完整性:缺 -x 精确、-i、-n/-o、-u/-P/-t/-v/-c;procps 的 ERE 正则完全未实现,只有子串。
- 技术评估:P1:taskkill 执行结果不校验——main.rs:37-41 Ok(_) => killed = true,而 Ok 只表示子进程启动成功,taskkill 因权限不足未杀掉目标时仍计为成功并退出 0;`-f` 依赖 wmic(main.rs:55-75),wmic 在 Windows 11 24H2+ 已移除,届时 -f 恒为空匹配且无报错;P2:tasklist CSV 按逗号裸 split(main.rs:19),镜像名含逗号时错位;不排除自身,pkill pkill 会杀掉自己(procps 明确排除);匹配大小写不一致:默认分支不敏感(main.rs:31)、-f 分支敏感(main.rs:27),均与 procps 相反;main.rs:13 的 expect 在 tasklist 缺失时 panic。
- Windows 合理性:完全依赖外部命令而非项目已有的 ps-sys,架构不一致且性能差;taskkill /F 未带 /T。
- 结论:P1 taskkill 假成功、wmic 依赖;P2 其余,建议改用 ps-sys。

#### pgrep
- 功能:同样基于 tasklist(+ -f 时 wmic)按名称/命令行子串匹配,输出 PID(-l 时 "PID 名字")。
- 移植完整性:缺 -x、-i、-n/-o、-c、-a、-P、-u/-t/-v;正则未实现。
- 技术评估:P1:无匹配时退出码恒 0——main.rs:7-44 全文没有任何非零返回,违反 procps "无匹配退出 1" 的核心脚本契约;其余与 pkill 同源:CSV 裸 split(main.rs:18)、不排除自身、-f 依赖 wmic(main.rs:47)、默认分支大小写不敏感(main.rs:30)。
- Windows 合理性:同 pkill。
- 结论:P1 无匹配退出码、wmic;P2 其余。

#### nice
- 功能:以调整后的优先级启动命令——当前实现不能正确完成此目标。
- 移植完整性:缺 nice -N 旧式连写形式。
- 技术评估:P0(本组最严重):整个 wmic 主路径不可用——① 命令参数 args.command[1..] 被完全丢弃(main.rs:16 取出后未使用),`nice -n 5 foo --flag bar` 实际执行 `cmd /c foo`;② 传的 priority=8+adjustment 既不是 Win32_Process.Create 的合法参数(该方法无 priority 参数),语义也与 nice 相反(正 adjustment 应降低优先级,main.rs:18-19);③ wmic process call create 异步返回,不等待子进程,退出码不传递,main.rs:31-39 恒退出 0;④ wmic 在 Windows 11 24H2+ 已移除,fallback 路径(main.rs:41-49)直接 spawn 不设置任何优先级——无 wmic 时 nice 是静默 no-op。另有 main.rs:48 wait().unwrap() panic 风险。
- Windows 合理性:未使用任何 Win32 优先级 API;正确做法是 CreateProcess 传 PriorityClass 并 wait 传递退出码。
- 结论:P0 主路径整体失效,建议按 SetPriorityClass 重写。

#### timeout
- 功能:限时运行命令,超时后用 taskkill /F 杀掉直接子进程。
- 移植完整性:-k/--kill-after 仅声明;缺 -s、--preserve-status、--foreground;时长格式只支持整数 s/m/h,缺小数/ms/us/d。
- 技术评估:P0 两个:① 子进程提前退出后仍阻塞满额时长——killer 线程先 sleep(duration) 再 taskkill,child.wait() 返回后 handle.join().unwrap() 无条件等 killer 睡满,`timeout 3600s echo hi` 在 echo 完成后挂起近一小时(main.rs:32-41);② 退出码 124 永不触发——用 status.code().is_none() 判定超时,但 Rust 在 Windows 上 ExitStatus::code() 恒返回 Some,taskkill /F 杀掉的子进程退出码为 1,超时路径恒走 exit(1)(main.rs:43-46);正确做法是基于 try_wait() 轮询区分"谁先结束";P1:-k 声明但从未实现(cli.rs:11,main.rs 未读 args.kill_after);时长解析失败静默归 0(main.rs:54/57/60/63 unwrap_or(0)),输入 "1.5s" 或非法串会立即 taskkill 杀死子进程;只杀直接子进程,taskkill 无 /T,孙进程存活(main.rs:34-36);P2:未用 child.kill()/Job Object;wait().unwrap() panic 风险。
- Windows 合理性:未使用 Job Object(Windows 上做进程树限时终止的标准方案)。
- 结论:P0 阻塞满时长、124 失效;P1 -k 未实现、解析归 0、不杀树;P2 改 try_wait/child.kill()/Job Object 可同时消除。

### 第 10 组:系统信息 I(top / free / uptime / w / who / whoami / id / groups / uname)

#### top
- 功能:非交互式监控:每轮打印内存头部(GlobalMemoryStatusEx)+ 进程计数与前 20 个进程名(Toolhelp 快照),共 iterations 轮、间隔 delay 秒。
- 移植完整性:缺 CPU 占用(总体与每进程均硬编码 0,main.rs:80、130)、VIRT/RES/SHR/USER 列(硬编码 0/"SYSTEM",main.rs:130)、up 时间/用户数/负载(全部假值,main.rs:72)、状态区分、全部交互式按键与整屏刷新。
- 技术评估:P0:快照失败时 get_process_count 返回 0(main.rs:37-39),main.rs:78 `processes - 1` 对 u64 下溢,debug panic;GlobalMemoryStatusEx 返回值被忽略(main.rs:15);头部把硬编码假数据当真实数据输出。
- Windows 合理性:内存/进程数据源合理;uptime/负载应映射 GetTickCount64/GetSystemTimes 但未实现。
- 结论:P0 修下溢(main.rs:78);P1 至少实现真实 CPU% 并标注假字段;P2 -i 与 procps 的 idle 撞名、未使用的 Performance feature。

#### free
- 功能:手写 FFI 调 GlobalMemoryStatusEx,按 -b/-k/-m/-g/-h 输出 Mem 与 Swap(pagefile commit)两行。
- 移植完整性:缺 -t/--total、-s、-c、-w、"available" 列(shared/buff/cache 打 "-",main.rs:48-49)。
- 技术评估:未发现明显缺陷(total-avail 无下溢;FFI 失败返回 None 并 exit 1,main.rs:79-81)。
- Windows 合理性:合理;ullTotalPageFile 是 commit limit(含物理内存),标为 "Swap" 会大于真实 pagefile 用量,属应注释的近似;-h 输出 "GB" 而 procps 为 "Gi"(main.rs:103)。
- 结论:P1 补 -t/-s/-w 与 available 列;P2 后缀样式、Swap 语义注释。

#### uptime
- 功能:Windows 分支打印"自 1970-01-01 至今的秒数",-p 打印中文时长。
- 移植完整性:缺 GNU 默认格式("HH:MM:SS up …, N users, load average …")、用户数、负载行、-s/--since。
- 技术评估:P0——Windows 分支 main.rs:32-38 返回的是 epoch 至今时长(约 5.6 万天),不是系统开机时长,输出完全错误,正确 API 是 GetTickCount64(Linux 分支 main.rs:24-29 是对的);main.rs:36 expect 在时钟早于 1970 时 panic(理论性)。
- Windows 合理性:负载均值无对应物,正确做法是省略或标注,目前连 uptime 本身都错。
- 结论:P0 改 main.rs:32-38 为 GetTickCount64;P1 默认输出改 procps 格式;P2 版本输出缺程序名。

#### w
- 功能:shell 出 systeminfo 解析启动时间、query user 解析当前登录用户名并逐行打印;-h 打印 "系统启动时间: …"。
- 移植完整性:缺 procps w 的标准头部与 USER TTY FROM LOGIN@ IDLE WHAT 全部列、-s/-f/-i。
- 技术评估:P0:main.rs:10、28、46 三处 expect——systeminfo/query/whoami 启动失败即 panic(Windows Home 无 query.exe 时 w 必崩);systeminfo 输出为 OEM 代码页,中文系统 GBK 经 from_utf8_lossy(main.rs:12)后中文变 U+FFFD,"系统启动时间" 分支永不命中(main.rs:14);systeminfo 单次运行数秒。
- Windows 合理性:query user 取会话方向正确,但应改 WTSEnumerateSessions/WTSQuerySessionInformation API;启动时间应改 GetTickCount64。
- 结论:P0 三处 expect 改优雅降级;P1 -h 语义与 procps -h/--no-header 相反(cli.rs:11),-u 会把输出变空(cli.rs:14 + main.rs:82-86);P2 OEM 解码、去掉 systeminfo。

#### who
- 功能:基于 USERNAME/SESSIONNAME/USERDOMAIN/COMPUTERNAME/LOGONSERVER 环境变量打印当前用户;-b/-a 附带 systeminfo 解析的启动时间。
- 移植完整性:仅当前用户一条记录;缺多会话枚举、TTY/登录时间/来源/PID 列、-H、-q/-u/-l。
- 技术评估:main.rs:41 仅匹配小写英文 "system boot time",中文系统(GBK + 中文表头)必落空 → 恒输出 "system boot time: unavailable"(main.rs:55-59);无 panic(均 .ok() 回退)。
- Windows 合理性:会话来源用环境变量只反映当前进程会话,非真实会话列表;-d 显式声明不支持属有意差异。
- 结论:P1 boot time 非英文系统必失败、未实现会话列表(建议 WTSEnumerateSessions);P2 换 GetTickCount64 免 systeminfo。

#### whoami
- 功能:打印 USERNAME 环境变量;-h/--hostname 打印 COMPUTERNAME,-u 同默认。
- 移植完整性:GNU whoami 只需裸用户名,已覆盖;-h/-u 为自有扩展(GNU 里 -h 是 help),文档已声明。
- 技术评估:未发现明显缺陷(无 panic,env 缺失有回退 main.rs:9-25)。
- Windows 合理性:身份取自环境变量而非访问令牌——runas/服务场景不反映真实 token;应用 GetUserNameExW;默认不输出 Windows 惯例的 domain\user。
- 结论:无需调整;P2 建议改用 GetUserNameExW 并考虑 domain\user。

#### id
- 功能:用 USERNAME/USERDOMAIN 环境变量拼出 uid=/gid=/groups= 格式字符串;-u/-g/-G/-n 切换字段。
- 移植完整性:缺数字 ID(GNU id -u 默认打数字,此处无 -n 时打 "DOMAIN\user" 字符串,与 GNU 相反)、-Z、以用户名作参数查询。
- 技术评估:P0:get_groups(main.rs:30-47)把用户名/机器名当组并硬编码 "Everyone"/"Authenticated Users"——输出是编造数据;main.rs:79-84 与 85-91 两分支代码完全相同(-n 在 -g/-G 下无任何效果);main.rs:97-116 两分支输出也完全相同(死分支)。
- Windows 合理性:完全未用 Token/ACL API:应 OpenProcessToken + GetTokenInformation(TokenGroups) + LookupAccountSidW,以 SID/RID 作数字 ID;"gid" 映射到用户自身也不对。
- 结论:P0 -G/groups 输出编造数据;P1 -u/-g 无数字输出且 -n 无效;P2 清理死分支。

#### groups
- 功能:shell 出 net user <当前用户>,按英文 "Local Group" 或中文"本地组成员"表头解析组名;失败则回退 env 拼凑。
- 移植完整性:GNU 的 `groups [USER]` 用法已声明(cli.rs:12-13)但未实现。
- 技术评估:P0:main.rs:15 恒用 env USERNAME 查询,cli.rs 的 USER 位置参数只用于显示,`groups otheruser` 会把当前用户的组安到他人头上;main.rs:43 中文分支在 GBK 代码页下永不命中(死分支);Err 回退(main.rs:61-75)输出编造组。
- Windows 合理性:net user 间接、慢且依赖网络服务,应改 CheckTokenMembership/GetTokenInformation+LookupAccountSidW;按 OEM 代码页解码。
- 结论:P0 让 USER 参数真正生效;P1 组数据编造/解析脆弱;P2 OEM 解码。

#### uname
- 功能:聚合系统信息:-s "Windows_NT"、-n shell 出 hostname、-r 解析 cmd /c ver、-v shell 出 PowerShell 读注册表、-m 编译期 ARCH、-o "Windows"。
- 移植完整性:-a/-s/-n/-r/-v/-m/-o 全覆盖;缺 -p/--processor、-i/--hardware-platform。
- 技术评估:未发现 panic/错误(全部 .ok() + 合理回退);P2:main.rs:114-118 为读一个注册表值启动整个 PowerShell(秒级),-a 还会连跑 2 次 cmd /c ver + 1 次 hostname,性能差;-m 用 consts::ARCH 是编译期目标架构,ARM64 上跑 x64 二进制会误报(应用 IsWow64Process2)。
- Windows 合理性:字段来源映射合理;PowerShell 应换 RtlGetVersion 或注册表 API。
- 结论:P2 去掉 PowerShell 与重复 ver 调用、ARCH 语义、compatibility.md:106 漏列 --operating-system。

### 第 11 组:系统信息 II / 性能(date / cal / df / du / iostat / mpstat / sar / vmstat / time)

#### date
- 功能:显示当前本地/UTC 时间(支持 chrono 格式串)及 epoch 秒/纳秒;-d、-s 已声明但未实现。
- 移植完整性:缺 -d/--date 解析(声明了却是 no-op)、-s/--set(同上)、-I/--iso-8601、--rfc-3339、-R、-r/--reference、%N 真实值。
- 技术评估:P0:`-s` 短选项被 --set(cli.rs:12)和 --seconds(cli.rs:16)同时声明,clap debug 构建下任何调用(含 --help)直接 panic("Short option names must be unique");release 下 -s 被 --set 遮蔽;P0:args.date_string(cli.rs:9)与 args.set_time(cli.rs:13)在 main.rs 中零引用——`date -d "yesterday"` 静默输出当前时间,`date -s "..."` 静默不设置,比报错更糟;P2:%N 恒为 000000000(main.rs:36-38 丢弃亚秒精度);默认格式用 %d(main.rs:27),GNU 是 %e(空格填充)。
- Windows 合理性:chrono Local 取注册表时区正确;%Z 输出 Windows 时区全名属可接受差异;无 panic 风险(unwrap_or 兜底)。
- 结论:P0 修 -s 双声明,实现或移除 --date/--set(至少报错);P2 %N、%d→%e。

#### cal
- 功能:按日/三月/整年(-y)打印月历,支持 -j 儒略日,周日开头。
- 移植完整性:缺单参数按"年"解释(GNU `cal 2024` 显示整年,这里崩溃)、-1、-A n/-B n、高亮今天。
- 技术评估:P0:`cal 2024`(GNU 最常见用法)被解析为 month=2024,NaiveDate::from_ymd_opt 返回 None 后 unwrap panic(main.rs:25);cal 13、cal 0 同样 panic(main.rs:39, 79, 136 的 unwrap 链);P2:-3/-y 多列并排时表头按固定填充,"May" 与 "January" 宽度不同导致后续列错位(main.rs:68, 125);-j 下 3 位序数撑破 3 字符单元格(main.rs:41, 92)。
- Windows 合理性:纯 chrono 计算,无平台问题。
- 结论:P0 校验 month 1-12 并把单参数按年份解释;P2 并排版对齐。

#### df
- 功能:用 GetDiskFreeSpaceExW 报告各盘符/指定路径的总量、已用、可用与使用率,支持 -h。
- 移植完整性:缺 -T/--print-type、-a、-i(inode 在 Windows 无对应,可声明差异)、-l、-t/-x、--output、-P。
- 技术评估:P1:文件系统类型硬编码 "NTFS"(main.rs:74),FAT32/exFAT/ReFS/网络盘全部误报;且 "Filesystem" 列放的是类型而非设备/卷名,应改用 GetVolumeInformationW;P2:GetDiskFreeSpaceExW 失败(路径不存在、断连网络盘)时静默跳过且退出码恒 0(main.rs:67-69);Use% 用 as u64 截断(main.rs:24-27,POSIX 向上取整);-h 下 KB 级无单位后缀(main.rs:137-138);未发现除零与 panic;UNC 路径原生可用,df //server/share 可用。
- Windows 合理性:数据源与盘符枚举(GetLogicalDriveStringsW,main.rs:87-116)正确;UNC 共享(无盘符)不在枚举内,属合理差异但未声明。
- 结论:P1 真实文件系统类型 + -T;P2 失败报错+退出码、Use% 取整、-h 后缀。

#### du
- 功能:递归累加文件 apparent size(字节)并输出每个参数一行合计。
- 移植完整性:缺 --max-depth、-a(默认模式也不逐子目录列出,GNU du 默认列出每个子目录一行,main.rs:18-26 只打印参数本身)、-b、-L、--exclude、-k/-m;默认单位是字节而非 GNU 的 1K 块。
- 技术评估:P0:du -s 结果错误——main.rs:19 du(path, !args.summarize) 使 -s 时 recursive=false,递归仅在 is_dir() && recursive 发生(main.rs:44),于是 du -s 目录只返回目录自身 metadata.len()(NTFS 上通常为 0);P1:无硬链接去重(main.rs:33-56 无 file ID 跟踪);符号链接/联接无循环风险:main.rs:34 用 symlink_metadata,正确;P2:错误只打印 stderr 后继续,退出码恒 0。
- Windows 合理性:统计的是 metadata.len()(apparent size),非磁盘占用,与 GNU du 语义系统性偏离且未声明;目录遍历可接受。
- 结论:P0 修 -s;P1 --max-depth、声明或实现磁盘占用语义、硬链接去重;P2 退出码、默认逐子目录输出。

#### iostat
- 功能:按间隔/次数循环打印各盘符统计行——全部数值为硬编码 0,纯壳。
- 移植完整性:缺 avg-cpu 段、-k/-m、-x 真实扩展输出、-t、-y、Average 汇总行、纯 iostat 2 应无限采样(这里 count 默认 1,cli.rs:18-19)。
- 技术评估:P1:get_disk_io 直接 return (0,0,0)(main.rs:19-21),kB_read/kB_wrtn 再硬编码两个 0(main.rs:34)——输出全是 0 且无任何"无数据"提示;Cargo.toml:12 已声明 Win32_System_Performance(PDH)但代码零使用;`-d/-x`(cli.rs:9-13)在 main.rs 零引用,静默 no-op;除零/panic 未发现。
- Windows 合理性:盘符发现用 Path::exists() 探测 A:-Z:(main.rs:8-17)可用但会把光驱也列出;正确数据源应为 PDH LogicalDisk/PhysicalDisk 计数器——架构选型合理,实现缺位。
- 结论:P1 接入 PDH 或至少不打印全 0 假数据、实现/移除 -d/-x;P2 avg-cpu、Average、无限模式。

#### mpstat
- 功能:按间隔/次数打印每 CPU 统计表——所有百分比硬编码 0,仅 CPU 枚举是真的。
- 移植完整性:缺 all 汇总行、Average 行、-A、-P ON/AUTO、逗号列表、纯 mpstat 1 无限模式(cli.rs:15-16 count 默认 1)。
- 技术评估:P1:print_cpu_stats 全部硬编码 0(main.rs:20-24);PDH 声明未用,GetSystemTimes 也未用;P2:-P 999 不做上界校验(main.rs:46-48);-P abc 解析失败时静默只打表头;无 panic 风险。
- Windows 合理性:数据源建议 GetSystemTimes(系统级)+ PDH Processor Information(每核);num_cpus 取核数正确。
- 结论:P1 接入真实数据或提示;P2 -P 校验、all/Average 行。

#### sar
- 功能:按间隔/次数采样内存(GlobalMemoryStatusEx,真实数据)并打印;CPU 行硬编码 0;-d 无输出。
- 移植完整性:缺 CPU 应为 -u(这里是 -c/--cpu,sysstat 无此短选项,cli.rs:8-9)、-b/-B/-W/-n/-q、时间戳、表头、Average、-f 历史。
- 技术评估:P1:sar -d 完全无输出——main.rs:59-60 仅给 -d 时 show_cpu/show_memory 均为 false,循环空转、退出码 0,静默吞掉已声明选项(cli.rs:15-16);CPU 行硬编码(main.rs:65);P2:GlobalMemoryStatusEx 返回值未检查(main.rs:14),失败时静默打全 0(对比 vmstat 有检查)。
- Windows 合理性:内存源合理;sar 历史(sa 日志)缺失对 Windows 是合理差异,建议 --help 注明仅实时模式。
- 结论:P1 实现/移除 -d、CPU 真实数据;P2 返回值检查、-c→-u、时间戳。

#### vmstat
- 功能:打印 procps 风格 17 列表头 + swpd/free/id 三列近似真实值,其余列硬编码 0。
- 移植完整性:缺 -w 宽输出(现有 -w 被挪用为 header_once)、-a、-s/-m/-d/-p/-f、纯 vmstat 2 无限模式。
- 技术评估:P0:内存只在循环外采样一次(main.rs:150),vmstat 1 5 的 5 行完全相同——采样类工具核心语义错误;P1:r 列用 Toolhelp 快照的进程总数冒充运行队列长度(main.rs:116),语义错误且每行做一次全系统快照;P2:id 列 = 100 - memory_load(main.rs:136),把内存指标放进 CPU idle 列;buff/cache/si/so/bi/bo/in/cs/st 全 0;main.rs:22-27 手写 extern 块中 GetProcessId、CreateToolhelp32Snapshot 未使用,死代码。
- Windows 合理性:数据源组合合理;si/so 可用 hard fault/sec 近似,缺位属实现缺口。
- 结论:P0 把 get_memory_info() 移入循环;P1 r 列语义;P2 id 数据源、死 extern、-w。

#### time
- 功能:前台执行子命令,测量 wall time 并打印 real/user/sys 三行,透传退出码。
- 移植完整性:缺 -p(POSIX 格式)、-f FORMAT、-o/-a、-v(--verbose,这里 -v 被占用为 version,cli.rs:6);输出格式 `\nreal\t0.003s` 非 GNU 亦非 bash 内建格式。
- 技术评估:P1:user/sys 两行硬编码 "0.000s"(main.rs:32-33),是编造数据——Windows 下可用子进程句柄 GetProcessTimes 取真实值;P2:命令不存在时退出 1(main.rs:24-27),GNU/shell 惯例是 126/127;无 panic。
- Windows 合理性:不经 shell、Instant(QPC)测 wall time 准确。
- 结论:P1 用 GetProcessTimes 填真实 user/sys;P2 -p/-f、格式对齐、127。

### 第 12 组:网络与权限(curl / wget / ping / nslookup / netstat / find / xargs / chmod / chown / test)

#### curl
- 功能:基于 reqwest(blocking + rustls-tls)发送 GET/POST/PUT/DELETE,支持自定义头、请求体、输出到文件;TLS 证书校验保持默认开启(处理正确)。
- 移植完整性:缺 -L(reqwest 默认跟随 10 次重定向,与 curl 默认不跟随相反且无法关闭,main.rs:20)、-f/--fail、-I、-u、--compressed、PATCH/HEAD(main.rs:22-31 白名单)。
- 技术评估:P1:默认打印 "HTTP/1.1 …"+全部响应头再打 body(main.rs:54-64),真实 curl 默认只输出 body,管道场景结果错误;`-d` 不隐含 POST,curl -d x url 会以 GET 带 body 发出(main.rs:15, 40-42);P2:头中无冒号时静默丢弃(main.rs:33-38);response.text() 全量读入内存(main.rs:66),大文件不流式;-s 只抑制头打印而非静默模式。
- Windows 合理性:rustls 免系统证书依赖;无 Windows 特有缺陷。
- 结论:P1 修默认输出与 -d 语义;P2 补 -L/-f、流式写盘。

#### wget
- 功能:HTTP GET 下载到 -O 指定文件或按 URL 文件名保存,-P 前置目录,非 2xx 退出 1。
- 移植完整性:缺 -c 断点续传、-t 重试、-i 批量、-r 递归、--no-check-certificate、-O - 输出到 stdout(现在会写名为 "-" 的文件,main.rs:79-82)。
- 技术评估:P1:`url.split('/').last()` 在 URL 以 / 结尾时返回 Some(""),文件名为空串导致 fs::write 失败退出,且 unwrap_or("index.html") 是死代码(main.rs:84-85);P2:查询串原样进入文件名(无 ? 截断与 URL 解码,main.rs:84);进度输出硬编码伪造时间戳 "2026-07-18 12:00:00"(main.rs:24);HTTP 错误退出码 1 而非 GNU 的 8(main.rs:48);body 全量进内存再落盘(main.rs:51, 69);未设超时,可能永久挂起(main.rs:27)。
- Windows 合理性:rustls 证书校验开启;重定向默认与 wget 一致。
- 结论:P1 修空文件名推导;P2 其余。

#### ping
- 功能:用 IcmpCreateFile + IcmpSendEcho 发 IPv4 echo,输出 Windows 风格逐条回复与统计;免管理员权限。
- 移植完整性:缺 -i(间隔固定 1s,main.rs:78)、-W(GNU 等待超时,此处被 -t 兼任)、-l(TTL)。
- 技术评估:P1:退出码恒为 0——全部丢包也正常返回,main.rs 全文统计输出后无任何非零 exit,依赖 ping 退出码的脚本全部误判成功;P2:reply.Status != 0 一律报 "Request timed out."(main.rs:156-165),主机不可达/TTL 到期被误报;size 未设上限,-s 70000 经 as u16 截断(main.rs:146);IPv6 直接退出(main.rs:37-40)。
- Windows 合理性:IcmpSendEcho 是正确选择(优于原始套接字);但 -t 在 Windows ping 语义是"持续 ping",此处是"超时秒数"(cli.rs:17),用户会踩坑;字节序处理正确(main.rs:36)。
- 结论:P1 修丢包退出码;P2 错误文案、size 上限、-t 语义说明。

#### nslookup
- 功能:仅 A/AAAA 查询走系统解析器(getaddrinfo);CNAME/MX/TXT 为纯输出桩。
- 移植完整性:仅 -t(且只有 5 种类型)、SERVER 位置参数;缺 -port、-retry、-timeout、交互模式(about 写着 "interactively" 但无交互循环,cli.rs:7)。
- 技术评估:P0:MX/TXT/CNAME 结果是伪造的:lookup_mx 输出 `{name} mail exchanger = 10 {name}`、lookup_txt 恒输出 "(no TXT records)"、lookup_cname 输出 `name canonical name = name`(main.rs:74-97),对用户输出假数据,比报错更糟;P1:SERVER 位置参数在 cli.rs:20-21 声明但 main.rs 从未引用,`nslookup example.com 8.8.8.8` 静默忽略指定服务器;P2:"Server: localhost" 硬编码且与实际解析器不符(main.rs:35-36);类型名大小写敏感,-t a 被拒(main.rs:19)。
- Windows 合理性:无 DNS 协议报文实现,完全依赖 getaddrinfo,无法查 MX/NS/SOA 属结构性局限。
- 结论:P0 伪造输出改为真实查询或显式报错;P1 支持 SERVER 或删掉;P2 大小写。

#### netstat
- 功能:GetTcpTable2 + GetUdpTable 枚举 IPv4 TCP(状态名映射正确)与 UDP,免管理员。
- 移植完整性:缺 -r(路由表,完全没有)、-o/-b(PID/进程,需换 GetExtendedTcpTable)、-e/-s 统计。
- 技术评估:P1:`-a` 与 `-n` 均在 cli.rs:14-18 声明并解析,但 main.rs 全文未引用:监听与 UDP 条目无条件输出、地址无条件数字显示,两个声明的选项是空操作;P2:端口换算 ((port >> 8) | (port << 8))(main.rs:82)仅在高位为 0 时正确;UDP 外部地址打 `*` 而非 `*:*`(main.rs:71);出错仅 eprintln 且退出码 0(main.rs:58-60);无 IPv6。
- Windows 合理性:GetTcpTable2/GetUdpTable 免管理员,合理;要 PID 应升级 GetExtendedTcpTable。
- 结论:P1 让 -a/-n 生效(或移除)并补 -r;P2 GetExtendedTcpTable、IPv6。

#### find
- 功能:递归遍历目录,支持 -name(glob)、-regex、-type f/d、-mindepth/-maxdepth、-basename、-i;用 symlink_metadata 起步,不跟随符号链接/联接,无循环风险(与 GNU 默认 -P 一致,处理正确)。
- 移植完整性:缺 -exec、-mtime、-print0、-size、-delete、-empty。
- 技术评估:P1:maxdepth 整体偏移一档且 0 值行为错误——本实现 -maxdepth 1 只输出起点(等价 GNU -maxdepth 0),-maxdepth 0 什么都不输出,因 main.rs:31 与 main.rs:84 的递归门槛用同一比较;P2:-type l/b/c/p/s 落入 _ => true 全部匹配(main.rs:50);glob_match 把模式直接转正则,非法时 main.rs:101 unwrap panic(如 -name "a[");-regex 只匹配文件名而非完整路径(main.rs:73);无效正则 expect panic(main.rs:20);遍历错误后退出码仍 0。
- Windows 合理性:与系统 C:\Windows\System32\find.exe 同名:本项目是目录遍历而 find.exe 是文本搜索,PATH 前置会静默破坏依赖后者的脚本,代码与文档均未提示(描述自认 "Windows find clone",Cargo.toml:7)。
- 结论:P1 修 maxdepth 偏移,评估补 -exec/-mtime;P2 -type 未知值报错、glob 转义、退出码、find.exe 冲突说明。

#### xargs
- 功能:按行+空白切分 stdin,分批用 Command::new 直接 spawn(不走 shell、参数逐个传递,无注入问题,做法正确);默认 echo,支持 -n/-p/-t/-i/--replace。
- 移植完整性:缺 -0/--null、-P 并行、-I str 短格式带值、引号处理完全缺失——`'a b'` 被切成两个参数(main.rs:20)。
- 技术评估:P1:`-p` 整体失效——stdin 已被 for line in stdin.lock().lines() 读到 EOF(main.rs:12-21),交互确认时 read_line 立即返回 EOF,恒判为拒绝,所有批次被跳过且最终退出 0(main.rs:106-112),应改从控制台读;P1:无参数长度上限管理——默认把全部输入塞进单批(main.rs:42-44),超过 Windows 约 32767 字符命令行上限时 spawn 直接失败,find | xargs cmd 大目录必炸;P2:空输入不运行命令(main.rs:23-25),与 GNU 默认"运行一次"相反(等于内置 -r);子命令失败即终止(main.rs:124-126),无 GNU 的 123"部分失败"语义;命令里含 {} 即隐式替换模式(main.rs:29-32)。
- Windows 合理性:子进程调用方式安全正确;缺 -0 使 find -print0 | xargs -0(含空格路径的唯一稳妥方案)不可用。
- 结论:P1 修 -p 从终端读、实现长度分批(顺带补 -0);P2 对齐 GNU 空输入/退出码、补 -P。

#### chmod
- 功能:八进制模式设置文件属性;Windows 上映射为 readonly 位:(mode & 0o222) == 0 才置只读(main.rs:96-103)——策略简明,compatibility.md:18 已声明语义不等价;支持 -R/-v。
- 移植完整性:仅八进制;符号模式(u+x、go-w、a=r)完全不支持,parse_mode 对非八进制串直接报错(main.rs:32-34)——这是 GNU chmod 最常用的一半语法。缺 -c、--reference、-f。
- 技术评估:P1:递归用 entry_path.is_dir()(跟随链接,main.rs:75-77)判定是否深入:遇到指向祖先目录的 junction/目录符号链接即无限递归直至栈溢出(与 find 用 symlink_metadata 防环形成对比);P2:仓库中提交了 test_chmod.txt 与 test_dir/ 应清理;0o644 与 0o777 在 Windows 上不可区分,属平台固有限制。
- Windows 合理性:readonly 映射是合理最小实现;无 ACL 编辑,execute 位无对应物,均为已声明差异;注意 Windows 目录 readonly 位基本被系统忽略。
- 结论:P1 补符号模式解析、修递归联接环;P2 清理杂物。

#### chown
- 功能:LookupAccountNameW 查 SID → GetFileSecurityW/SetSecurityDescriptorOwner/SetFileSecurityW(OWNER_SECURITY_INFORMATION)改文件 owner,支持 -R/-v;API 链路方向正确。
- 移植完整性:不遵循 GNU `chown [OWNER][:GROUP] FILE` 位置语法,改为 --user/--group 长选项;缺 --reference、-H/-L/-P、-h。
- 技术评估:P0:`--group` 实际改的是 owner 而非 group——main.rs:22 把 group 名当账号名查 SID,后续仅用 OWNER_SECURITY_INFORMATION(main.rs:129, 159, 168-171),全文无 SetSecurityDescriptorGroup/GROUP_SECURITY_INFORMATION,chown --group X f 会把文件 owner 改成组 X 的 SID;P1:任何文件失败仅 eprintln,进程退出码恒 0(main.rs:31-42);P2:--user 与 --group 同给时 group 被静默忽略(main.rs:22);递归用 is_dir()(main.rs:210)同样存在联接环风险;输出文案为中文,与全部兄弟命令不一致。
- Windows 合理性:API 链路可行;改 owner 为他人通常需 SeRestorePrivilege,失败会以错误暴露;SetFileSecurityW 属较老 API(SetNamedSecurityInfoW 更佳)。
- 结论:P0 修 --group(GROUP_SECURITY_INFORMATION 或明确拒绝);P1 失败返回非零;P2 联接环、输出语言统一。

#### test
- 功能:文件判断 -e/-f/-d/-s + 三操作数 `=`/`!=` 字符串比较,退出码 0/1,用法错误 2——退出码语义正确(main.rs:42, 33-39),无 panic(len==3 守卫,main.rs:27-28)。
- 移植完整性:缺数值判断 -eq/-ne/-lt/-le/-gt/-ge(test 1 -eq 1 现在报 "invalid operator" 退出 2)、单参数非空串测试(test foo 应为真)、-z/-n/!、-r/-w/-x/-L、-nt/-ot/-ef。
- 技术评估:未发现明显缺陷;主要问题是能力面过窄使大量合法用法落入 exit 2,而非实现错误。fs::metadata 跟随链接,与 GNU 一致。
- Windows 合理性:NTFS 下无 execute/设备文件概念,缺失对应判断反而避免了错误映射;clap conflicts_with_all 接口自洽。
- 结论:P1 补数值比较与单/双参数形式;P2 其余。其余无需调整。

### 第 13 组:环境与终端(env / tty / clear / reset / patch)

#### env
- 功能:仅打印当前进程环境变量,-u 打印时过滤,-i 直接返回(打印空)。
- 移植完整性:缺失 NAME=VAL 赋值与命令执行(env FOO=bar cmd / env FOO=bar 都会被 clap 拒绝)、-S/--split-string、-0/--null——不是 GNU env 的"运行命令"语义。
- 技术评估:未发现 panic 级缺陷;但 `-v` 诊断输出写进 stdout(main.rs:9-11、24-26),管道场景污染数据流;main.rs:30-34 的 if/else 两分支完全相同;std::env::vars()(main.rs:22)遇非 Unicode 环境变量会 panic。
- Windows 合理性:env::vars() 走宽字符 API 转换,可接受。
- 结论:P1 补 NAME=VAL 与命令执行、-S;P2 -v 改 stderr、删冗余分支。

#### tty
- 功能:名义上打印终端名并按是否为 tty 返回退出码;-a 逐个检查 stdin/stdout/stderr。
- 移植完整性:-s 已覆盖;-a 是自创扩展(已声明);缺 GNU 的 --quiet 长别名;GNU tty 语义是"打印连接到标准输入的终端名",此处未实现。
- 技术评估:P0——判断 tty 依据是环境变量 CON(main.rs:6-8、37-47),而 Windows 从不默认设置名为 CON 的环境变量(CON 是 DOS 设备名),真实控制台/Windows Terminal 中 tty 恒输出 "not a tty" 并 exit 1,tty -s 恒返回 1,脚本判断全部失效;且与 -a 分支正确使用的 is_terminal()(main.rs:15-29)自相矛盾,tty 与 tty -a 结论不同;`-a` 全部非 tty 时仍 exit 0(main.rs:34)。
- Windows 合理性:不合理,应改用 io::stdin().is_terminal()(已导入却只在 -a 用)。
- 结论:P0 主路径改用 is_terminal(main.rs:37-47);P2 补 --quiet、修 -a 退出码。

#### clear
- 功能:向 stdout 输出 \x1B[2J\x1B[H 清屏并光标归位。
- 移植完整性:核心行为已覆盖(等价 clear 默认,不清滚动缓冲);缺 -x、-T type。
- 技术评估:未发现明显缺陷(print! 后 main 正常返回,flush 正常)。
- Windows 合理性:纯 ANSI 转义,Windows Terminal/已启用 VT 的 conhost 下正确;未调用 SetConsoleMode(ENABLE_VIRTUAL_TERMINAL_PROCESSING)(对比 reset 用了 winapi),旧式 conhost 中会打印乱码。
- 结论:无需调整(P2:启用 VT 或改控制台 API,main.rs:8)。

#### reset
- 功能:将 stdin 控制台模式重置为 ECHO|LINE|PROCESSED,再输出 \x1b[H\x1b[2J\x1b[3J 清屏并清滚动缓冲。
- 移植完整性:抓住核心(恢复终端行编辑/回显 + 清屏);缺 RIS、输出模式重置、tset 系选项(可接受)。
- 技术评估:io::stdout().flush().unwrap()(main.rs:30)在 stdout 关闭/管道断裂时 panic;GetStdHandle 结果只判 NULL 未判 INVALID_HANDLE_VALUE(main.rs:22)。
- Windows 合理性:合理——控制台 API 重置输入模式 + ANSI 清屏是 Windows 上的正确组合。
- 结论:无需调整(P2:去 unwrap、补句柄判断)。

#### patch
- 功能:读 unified diff 补丁,逐 hunk 精确匹配上下文应用到单个目标文件,失败 hunk 生成 .rej(内容实为错误消息)。
- 移植完整性:已覆盖 -R(main.rs:292-311 逻辑正确)、-f、--version;缺 -i(补丁只能走位置参数,无 stdin)、--dry-run、-o、-b/--backup、-l、偏移/模糊匹配(fuzz)。**-p/--strip 被解析但全程序零引用(仅 cli.rs:16-19 出现)**,extract_target_file(main.rs:208-218)原样取 +++ 行路径——git 风格 b/file.txt 前缀不被剥离也找不到文件。
- 技术评估:多项已验证缺陷:① P0:hunk.old_start - 1(main.rs:223)对 `@@ -0,0 +1,N @@` 新建文件补丁 usize 下溢,debug panic/release 恒拒绝,新建文件类补丁完全不可用;② P1:解析 hunk 不按声明行数计数,遇空行提前截断(main.rs:180),遇 `---` 也截断(main.rs:182)——删除以 -- 开头内容的补丁被破坏;③ P1:多文件补丁的 target 只取第一个 +++(main.rs:42-45),全部 hunk 都被应用到这一个文件(main.rs:64-82);④ P1:非 force 模式遇 reject 即 break 且不写盘(main.rs:64-92),此前已成功应用的 hunk 全部丢弃,GNU 语义是写回成功部分;⑤ P1:.rej 写入的是错误消息字符串而非被拒 hunk(main.rs:76 + 251-260,crate 内遗留 test.txt.rej 即证据);⑥ P1:--force 时 rejects 不落盘直接丢弃(main.rs:84),与帮助文案不符;⑦ P2:&line[1..](main.rs:192)在多字节字符开头时按字节切片 panic。
- Windows 合理性:不佳。read_to_string + lines() + join("\n")(main.rs:221、289)把 CRLF 目标文件静默改写为 LF;UTF-8 BOM 残留导致首行上下文永不匹配(test.txt.rej 中即真实发生的案例);不处理 "\ No newline at end of file"(main.rs:277 的 _ => {} 被静默忽略)。
- 结论:P0:-p 未实现却接受、old_start-1 下溢;P1:hunk 解析、多文件补丁、reject 处理、缺 -i/--dry-run;P2:模糊匹配、CRLF/BOM 保持、清理根目录遗留 test.*。

## 修复路线图建议

按"先止漏、再补全、后打磨"排序:

1. **第一批·安全与数据损坏(P0 中的高危)**:tar/unzip 的 zip slip、zip 的 PowerShell 注入、sync 的 PATH 自递归、chown --group 错改 owner、nslookup/id/groups/mpstat/iostat/sar/vmstat/time/ps 的假数据输出、cat 的数据损坏、mv 的删树路径、cp/touch 的核心功能失效。
2. **第二批·panic 与退出码契约**:seq/cal/tr/join/od/hexdump/sed/awk 的 panic;wc/rm/diff/file/df/du/ping/kill/pkill/pgrep/chown/sort/xargs/shred/hostname 的失败 exit 0;yes 的 BrokenPipe;timeout 的 124。
3. **第三批·声明与实现对齐**:实现或移除全部 no-op 选项(sort -k/-t、date -d/-s、sar -d、netstat -a/-n、iostat -d/-x、patch -p、shred --force、timeout -k、ps --headers-repeat、cp --preserve);同步 compatibility.md 约 10 处过期条目;README(ps)移除未注册列。
4. **第四批·统一基础设施**:字节安全行读取层(解决 CRLF/无尾换行/非 UTF-8 三类系统性问题)、BrokenPipe 统一处理、verbatim 路径(\\?\)剥离工具(readlink/which/ln/pwd 复用)、OEM 代码页解码(w/groups/uname 复用)、ps-sys 能力下沉复用(pkill/pgrep/nice 去外壳化)。
5. **第五批·真实数据源**:性能类命令接 PDH/GetSystemTimes/GetTickCount64(GetProcessTimes);w/who 改 WTSEnumerateSessions;id/groups 改 Token API;touch 改 SetFileTime。
6. **持续项**:crate 根目录手工测试杂物清理;每个命令修复前先补 1-2 个最常用路径的行为测试,把 "Smoke only" 逐步升级。

## 与 compatibility.md 的文档同步项

以下条目声明与代码不符,修代码时同步:grep 行漏 --recursive/--quiet;cmp 行漏 --verbose/--silent;diff 行漏 --unified/--ignore-case/--ignore-whitespace/--ignore-blank-lines;join 行漏 --field1/--field2;od 行漏 --skip-bytes/--read-bytes/--width;hexdump 行漏 --skip;numfmt 行漏 --to/--to-unit/--padding/--round/--suffix;uname 行漏 --operating-system;basename 行漏 --suffix/--multiple;ls 行漏 --unbuffered 之外的若干;chown 行漏 --user/--group/--recursive/--verbose;zip 行应披露 PowerShell 外壳实现;sort 的 --key/--field-separator 与 fmt 的 --crown-margin 为"声明支持实为无效"。


