# 命令与 Windows 兼容性

[English](compatibility.md) | **简体中文**

当前 workspace 范围为 137 个命令。下表参数来自各 cli.rs 中显式声明的 long 名称，不是完整语法；以该命令 --help 为准。源码声明不等于功能已经通过测试。

所有命令已通过有界发布冒烟检查（帮助入口，true/false 为退出码）。只有标记“行为测试”的命令运行了代表性业务断言；其余业务行为尚未自动验证。未声明完整 GNU/POSIX 兼容性。

| 命令 | 显式长参数示例（源码） | 当前验证 | Windows 差异 / 限制 |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | 冒烟 + 行为测试 | 条件、else/next 与多参数 print 可用；NR 跨文件累计，BEGIN/END 各运行一次；for/while 循环与三段式 for（含 i++/i--）可执行。模式与 `~` 使用 ERE（regex crate）。内建：length（含裸形式）、substr（gawk 裁剪语义）、index、sprintf（共享 GNU 格式化器）、toupper、tolower、sub/gsub（& 写回变量或 $0，字段重切分）、split 入数组。数组：元素读写、delete arr[k] / delete arr、`k in arr`、for-in（键排序）。printf 支持完整指示符集。仍不支持：用户函数、重定向、相邻值字符串拼接。 |
| base64 | --decode, --version, --wrap | 冒烟 + 行为测试 | 解码与 GNU 一致地严格：无填充且长度非 4 倍数的输入会输出已解码字节，但报 invalid input 并退出 1。 |
| base32 | -d, --decode, --ignore-garbage, --wrap | 冒烟 + 行为测试 | 按 GNU 换行语义编解码 RFC 4648 base32（默认 76，0 为不换行）；解码始终容忍换行，-i 跳过其他非法字节；填充结束每组。 |
| | b2sum | --check, --length, --quiet, --status, --tag | 冒烟 + 行为测试 | BLAKE2b，沿用 sha1sum 家族的输出、--tag 与校验规则；--length 接受 8 的任意倍数且不超过 512。无 -b/-t/-z 标记；GNU (MSYS) 对 stdin 输出二进制标记，本实现从不输出。 |
| basename | --multiple, --suffix, --version, --zero | 冒烟 + 行为测试 | -z/--zero 仅改变输出终止符，不改变输入解析。支持 NAME SUFFIX 和 -a/-s，保留操作数内 LF。Windows 路径解析规则保留：末尾 ./.. 输出 ./..，// 输出 /，盘符根输出 C:。 |
| bc | --version | 冒烟 + 行为测试 | i128 整数运算，/ 与 % 向零截断（GNU scale=0 默认语义）；支持 + - * / % ^ ( )、变量赋值、; 分隔、# 注释与 quit。无任意精度、scale、ibase/obase 与函数——遇到不支持结构时打印提示并跳过该行。banner 为 bc 0.1.0 (RuxCmd)。 |
| cal | --version | 冒烟 + 行为测试 | 单参数 1-12 视为月份，100-9999 视为年份；13-99 被拒绝。完整 GNU 行为未验证。 |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs, --unbuffered | 冒烟 + 行为测试 | 默认字节透明（CRLF、二进制、非 ASCII 数据原样透传）；-v 使用字节级 ^/M- 记号；-E 不给未结尾的末行补 $。行号与空行压缩状态按文件重置。 |
| chmod | --recursive, --verbose, --version | 冒烟 + 行为测试 | 符号模式（u/g/o/a 与 +-=rwx）映射到只读位；-R 不再跟随 junction。Windows 权限语义与 Unix mode 不等价。 |
| | chgrp | --recursive, --silent, --verbose | 冒烟 + 行为测试 | GROUP 通过 LookupAccountNameW 解析（本机账户；数字 GID 不做特殊处理），经 SetNamedSecurityInfoW 应用。改组通常要求目标组在当前令牌中；-R 会跟随 junction（与 chown 相同的限制）。 |
| chown | --user, --group, --recursive, --verbose, --version | 冒烟 + 行为测试 | --user 设 owner、--group 设组（GROUP_SECURITY_INFORMATION），可同时给；任一文件失败退出 1。改 owner 为他人通常需要特权。Unix UID/GID 语义不等价。 |
| cksum | --algorithm, --version, --zero | 冒烟 + 行为测试 | 仅 POSIX CRC32（-a crc）。-z/--zero 以 NUL 终止输出，隐式 stdin 不带文件名，显式 - 打印 -；读取错误报错。本机 GNU cksum 8.32 无 --zero，仅该差分用例跳过；NUL CRC 有已知向量行为测试。不支持其他算法和校验清单。 |
| clear | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| cmp | --silent, --verbose, --version | 冒烟 + 行为测试 | GNU 退出码（0/1/2），differ 消息走 stdout（char N、line M），带引号的 EOF 消息含字节/行上下文，-l 八进制清单，单操作数与 stdin 比较，-l/-s 互斥拒绝。 |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | 冒烟 + 行为测试 | -t 模式未给 -s 时连续空白折叠为单一分隔符；显式 -s 按字符切分不折叠。默认 fill 模式未实现。 |
| comm | --version, --zero-terminated | 冒烟 + 行为测试 | 支持 -z NUL 记录及原始字节比较；-1/-2/-3 隐藏列后压缩剩余列的 TAB 前缀。要求两个文件操作数，输入已按相同字节顺序排序；不支持 stdin 操作数、GNU 排序校验诊断和 --total。 |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | 冒烟 + 行为测试 | -f 覆盖前清除目标的只读属性。以 `.`/`..` 结尾的源将内容并入已存在的目标目录（目标不存在时报错）；多源时目标必须是已存在目录。 |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet | 冒烟 + 行为测试 | 行号须 >0、升序且不超过文件末尾，否则退出 1。模式仅支持子串与行号；/re/+N、{N} 等 GNU 语法报错退出 1；模式不匹配退出 1（无 -k 时删除输出文件）。--suffix-format 已移除。 |
| curl | --data, --header, --output, --request, --silent, --version, --include, --head, --fail, --location | 冒烟 + 行为测试 | 默认仅输出 body（-i 加响应头）；-d 隐含 POST；重定向默认关闭（-L 手动跟随，最多 10 次）；-f 对 HTTP >= 400 退出 22。HTTPS/TLS 与联网行为尚未验证。 |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter, --zero-terminated | 冒烟 + 行为测试 | 支持 -z NUL 记录；字节和字段选择保持原始数据，-c 对合法 UTF-8 记录按字符选择，非法输入回退为字节。NUL 模式保留 CR/LF，默认 LF 模式仍剥 CRLF。无字段分隔符的记录默认透传，-s 时抑制；本端既有 -s 也作用于 -b/-c（与 GNU 不同）。不支持 -n/--complement。 |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | 冒烟 + 行为测试 | -d/--date 与位置参数接受 @EPOCH、YYYY-MM-DD、YYYY-MM-DDTHH:MM[:SS]（空串=now，其他格式报错退出 1）；-s/--set 经 SetLocalTime 设置本地时钟。--seconds 作用于解析出的时刻；%N 恒为 0。 |
| dd | conv=notrunc, status=none/noxfer | 冒烟 + 行为测试 | 核心子集：if/of/bs/ibs/obs/count/skip/seek，支持 GNU 尺寸后缀（c/w/b/K/KB/M/G 及 1000 进制形式）；conv 支持 notrunc 并拒绝其他值；stdout 不可 seek。GNU 的 records/copy 统计输出到 stderr。conv/flag 完整面（sync、fullblock、iflag）不支持。 |
| df | --human-readable, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| diff | --unified, --ignore-case, --ignore-whitespace, --ignore-blank-lines, --version | 冒烟 + 行为测试 | 退出码遵循 GNU（0/1/2）。普通格式输出未合并的 d/a 命令，行号锚点与 GNU 一致；c 合并与区间折叠未实现。 |
| dirname | --version, --zero | 冒烟 + 行为测试 | -z/--zero 仅改变输出终止符，路径仍作为命令行操作数传入。路径输出正斜杠；仅根 / 或 // 输出 /，裸盘符前缀输出 .。Windows 路径与 UNC 近似保持不变。 |
| du | --human-readable, --summarize, --total, --version | 冒烟 + 行为测试 | -s 与默认模式都递归求 apparent-size 合计、每个参数输出一行（逐子目录列出仍未实现）；不可读路径报错并退出 1。 |
| echo | --version | 冒烟 + 行为测试 | 未知的前导选项簇按操作数原样输出（含 --）；-e 覆盖 coreutils 全部转义（\c 截断全部输出）；--help/--version 与 coreutils 一致。 |
| env | --ignore-environment, --unset, --verbose, --version | 冒烟 + 行为测试 | NAME=VAL 赋值并执行命令（-i 清空、-u 移除、透传退出码、spawn 失败 127）；无命令时打印环境。 |
| expr | See --help | 冒烟 + 行为测试 | 整数四则（i64）、| / &（GNU 左操作数语义）、比较（双方均为整数按数值否则按字符串）、: 与 match 的锚定 BRE 匹配（返回分组 1 或匹配长度）、substr/index/length。BRE 反向引用不支持（报语法错误）；退出码与 GNU 一致（0/1/2）。 |
| expand | --initial, --tabs, --version | 冒烟 + 行为测试 | 字节安全：CR 字节、非法 UTF-8 与未终止末行原样透传；tab 诊断与 GNU 一致（0 尺寸、升序列表）。 |
| factor | --version | 冒烟 + 行为测试 | 对 u64 试除（无 Pollard rho，大半素数较慢）；0 和 1 与 GNU 一致输出裸标签。 |
| false | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| file | 见 --help | 冒烟 + 行为测试 | -b 省略名称前缀；gzip（1F 8B）与 zlib（78 xx）区分识别；可打印 ASCII 与合法 UTF-8 报 ASCII text / UTF-8 text；打不开的操作数报错跳过，结束时如有失败退出 1。 |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | 冒烟 + 行为测试 | maxdepth 遵循 GNU（0=仅起点，1 含子项）；接受 GNU 单横线长选项（-name、-type 等）；未知 -type 值与遍历错误退出 1；非法 glob 不再 panic。 |
| fmt | --crown-margin, --split-only, --version, --width | 冒烟 + 行为测试 | -c 为保留段落首行缩进的开关；-s 尽量在空白处断行。 |
| fold | --bytes, --spaces, --version, --width | 冒烟 + 行为测试 | 非 UTF-8 输入按 lossy 折行不再报错；-b 按原始字节切分。 |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching, --quiet, --recursive, --extended-regexp, --fixed-strings, --word-regexp, --files-with-matches, --with-filename, --no-filename, --after-context, --before-context, --context | 冒烟 + 行为测试 | 字节安全读行：非 UTF-8 输入按 lossy 匹配并原样输出字节；单文件输出无前缀；-F 按字面匹配。递归错误传播尚未验证。 |
| groups | --version | 冒烟 + 行为测试 | 输出当前用户的真实令牌组（空格分隔，组名与区域相关）；仅支持当前用户，其他名字报错退出 1。 |
| gunzip | --force, --keep, --stdout, --list, --verbose | 冒烟 + 行为测试 | 多成员流完整解压；-c 输出到 stdout 并保留源；-l 输出大小与比率。 |
| head | --bytes, --lines, --quiet, --verbose, --zero-terminated | 冒烟 + 行为测试 | 支持 -z 记录计数，-c 仍按字节计数；选中数据、CR/LF 和未终止末记录原样输出，不补 NUL。多文件及 -v 标题仍以 LF 终止；-n 与 -c 互斥。GNU 负数行数语义仍不支持。 |
| hostid | --version | 冒烟 + 行为测试 | 输出 COMPUTERNAME 的 32 位 FNV-1a 散列（八位小写十六进制）；与 Linux hostid 语义不等价。 |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal, --skip | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| hostname | --fqdn, --ip-address, --version | 冒烟 + 行为测试 | 失败时打印错误并退出 1，不再静默。完整 GNU 行为未验证。 |
| iconv | --from-code, --list, --output, --to-code, --version | 冒烟 + 行为测试 | 解码严格遵循 -f（无 BOM 嗅探）；--list 覆盖完整编码表。 |
| id | --group, --groups, --name, --user, --version | 冒烟 + 行为测试 | uid/gid/groups 取自进程令牌 SID 的 RID；-u/-g/-G/-n 遵循 GNU；组名与区域相关，无法解析时回退为 RID。 |
| install | --directory, --mode, --verbose | 冒烟 + 行为测试 | 复制文件；`-d` 建目录，`-D` 建目标父目录。八进制权限的写位映射 Windows 只读属性，不等价于 Unix 所有者、组和执行位。替换已有文件条目，不改写其硬链接别名；源与目标同一文件时报错。不支持符号权限及其他 GNU install 选项。 |
| iostat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| join | --field1, --field2, --separator, --version | 冒烟 + 行为测试 | 输出为键 + 双方剩余字段；无 -t 时按连续空白切分（输出以空格连接）。小于 1 的字段号报错退出 1。 |
| kill | --list, --signal, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| link | （仅自动 help/version） | 冒烟 + 行为测试 | 创建一个文件硬链接，两个路径必须位于支持硬链接的同一卷；不创建符号链接或目录链接。 |
| ln | --force, --symbolic, --verbose, --version | 冒烟 + 行为测试 | 链接目标按书写形式存储（无 \\?\ 前缀）；-v 打印创建结果。Windows 符号链接可能需要权限或开发者模式。 |
| logname | --version | 冒烟 + 行为测试 | GetUserNameW 获取当前进程账户，失败回退 USERNAME；切换凭据后可能与 Unix 登录身份不同。 |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | 冒烟 + 行为测试 | 长格式附加列（-i/-s 与符号链接目标）按位置索引渲染，着色时不丢失。完整 GNU 行为未验证。 |
| md5sum | --check, --quiet, --status, --tag, --version, --zero | 冒烟 + 行为测试 | 支持 -z/--zero 和 --tag -z，NUL 模式不转义文件名；-z -c 退出 1。普通校验从 stdin 或 - 读取清单，保留精确文件名，支持 GNU 转义/BOM/二进制标记/tag，并传播读取错误；FAILED 写 stdout，--status 静默。不支持 -b/-t、strict 或 NUL 清单解析。 |
| mkdir | --mode, --parents, --verbose | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| mkfifo | --mode, --version | 冒烟 + 行为测试 | 创建瞬时的 Windows 命名管道服务端句柄并立即关闭，不留下持久对象；-m 声明不支持。 |
| mktemp | --directory, --quiet, --suffix, --tmpdir | 冒烟 + 行为测试 | 使用系统安全随机源替换至少三个连续 X，独占创建避免覆盖。支持 `-d`、`-p DIR`、`-t` 和后缀。默认目录遵循 Windows 临时目录规则；继承 Windows ACL，不提供 Unix 0600/0700 权限语义。不支持无值 `--tmpdir` 和 dry-run。 |
| mpstat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mv | --force, --interactive, --no-clobber, --verbose | 冒烟 + 行为测试 | 拒绝以非目录覆盖目录（绝不删除目标树）；跨卷回退为 copy+delete；src/. 并入已存在目录。 |
| netstat | --all, --numeric, --protocol, --version, --own-pid | 冒烟 + 行为测试 | -a 控制是否显示 LISTENING/未绑定条目；-o 增加所属 PID 列（GetExtendedTcp/UdpTable）；表查询错误退出 1。 |
| nice | --adjustment, --version | 冒烟 + 行为测试 | 直接 spawn 命令（不依赖 wmic）并应用优先级：adjustment <=-15 High、-14..-5 Above normal、-4..4 Normal、5..14 Below normal、>=15 Idle；透传子进程退出码。 |
| nl | --body-numbering, --number-width, --separator, --version | 冒烟 + 行为测试 | 按 UTF-8 行读取不乱码；-b 接受 t（默认，仅非空行）、n（不编号）、a（全部行）；每行输出均以换行结束，与 GNU 一致。 |
| nohup | （仅自动 help/version） | 冒烟 + 行为测试 | DETACHED_PROCESS 脱离控制台，并尝试脱离 Job；Job 禁止脱离时明确警告子进程仍受其生命周期约束。等待并透传退出码；终端 stdin 改为空设备，终端 stdout 追加 nohup.out（TEMP 回退），重定向流保持独立。已测试脱离执行与重定向流，未自动模拟物理关闭终端。 |
| nslookup | --type, --version | 冒烟 + 行为测试 | A/AAAA/CNAME/MX/TXT 经真实查询解析（getaddrinfo / DnsQuery_W）；类型名大小写不敏感；不支持的类型报错退出 1。SERVER 位置参数已移除。 |
| nproc | --all, --ignore | 冒烟 + 行为测试 | 默认为可用并行度；--all 输出处理器总数；--ignore 做减法且下限为 1。 |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header, --padding, --round, --suffix, --to, --to-unit | 冒烟 + 行为测试 | 显式 -d 在输出中保留且不折叠连续分隔符；--from=auto 读取 K=1000、Ki=1024。 |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal, --read-bytes, --skip-bytes, --width | 冒烟 + 行为测试 | -t 支持 x1/d1/o1/u1/a/c/z 与 f4/f8（GNU 字段宽度、小端序）；仅格式含 c 时输出 ASCII 列。 |
| open | --application, --edit, --reveal, --version, --wait | 冒烟 + 行为测试 | 经 ShellExecuteW 打开（无 shell 引号问题）；-R 忽略 explorer 退出码；-W 在启动返回进程句柄时等待。 |
| pathchk | --all, --portability | 冒烟 + 行为测试 | 检查空组件、255 字节组件与 4096 字节路径上限；-p 限制为便携文件名字符集；盘符前缀属结构部分。不检查文件系统访问权限。 |
| paste | --delimiters, --serial, --version, --zero-terminated | 冒烟 + 行为测试 | 支持 -z NUL 记录、并行/串行和循环 -d 分隔符列表。支持 \n、\t、\\、\0 转义（\0 表示不插入字段分隔符）；记录分隔符与字段分隔符独立。重复 - 操作数共享 stdin；串行空输入仍输出一个记录终止符。默认 LF 模式剥 CRLF，NUL 模式保留 CR/LF 和非法字节。 |
| patch | --force, --reverse, --strip, --version | 冒烟 + 行为测试 | --strip/-p 剥离 N 个前导路径组件（默认 0）；hunk 严格按 @@ 声明计数消费（处理空上下文行与 \\ No newline）；多文件补丁分段应用；.rej 为被拒 hunk 原文；成功的 hunk 写回。 |
| pgrep | --full, --list-name, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ping | --count, --size, --version | 冒烟 + 行为测试 | 全部丢包退出 1；失败状态分类提示；-s 上限 65500；-t 为持续 ping（Windows 语义），-w 为单次应答超时。 |
| pkill | --full, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | 冒烟 + 行为测试 | 页眉显示真实本地日期时间；-n 遵循 GNU -n[SEP][DIGITS]（裸 -n 为 TAB + 宽 5，非法规格退出 1）；help 仅 --help（-h 是页眉）。 |
| printenv | --null | 冒烟 + 行为测试 | 无名字时输出整个环境；给名字时输出命中值，任一缺失退出 1。 |
| printf | --version | 冒烟 + 行为测试 | 完整 GNU 指示符支持：宽度/精度/标志（- 0 + 空格 #）、d/i/u/o/x/X（按 base-0 解析，%u 回绕负数）、f/e/E/g/G、c/s、%b（含 \c 截断）、%q、跨参数的格式复用，以及无转换指令时对多余参数的警告。未知指令报错退出 1；非法数字仍输出 0 并退出 1。 |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | 冒烟 + 行为测试 | 原生 Windows 进程信息；不可读取字段可能显示 ?。%MEM 使用真实总物理内存（GlobalMemoryStatusEx）；请求未注册列时 stderr 警告并跳过该列。 |
| pwd | --logical, --physical, --version | 冒烟 + 行为测试 | UNC 路径输出为 //server/share。完整 GNU 行为未验证。 |
| readlink | --canonicalize, --no-newline, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| realpath | --strip-components, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| reset | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| rev | --version | 冒烟 + 行为测试 | 有效 UTF-8 按字符反转（util-linux 语义），非法序列按字节反转；未终止末行保持未终止。 |
| rm | --dir, --force, --no-preserve-root, --one-file-system, --preserve-root, --verbose | 冒烟 + 行为测试 | -f 仅静默不存在的路径；其余失败均打印错误并退出 1，preserve-root 拒绝信息不被压制。 |
| rmdir | --ignore-fail-on-non-empty, --parents, --verbose | 冒烟 + 行为测试 | 仅删除空目录，`-p` 向上逐级删除；忽略标志仅抑制目录非空错误。受 Windows 文件系统和 ACL 限制。 |
| sar | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sed | --expression, --regexp-extended, --quiet, --in-place, --separate, --version | 冒烟 + 行为测试 | 默认 POSIX BRE（-E/-r/--regexp-extended 切换 ERE，两种方言均带 GNU \+ \? 扩展）；地址与 s 命令共用方言；替换端支持 & 与 \1-\9、\n/\t/\r 以及 g/N/p/i 标志。模式中的反向引用不支持（启动报错）。非 UTF-8 数据按原始字节匹配并原样透传；行号逐文件重置；缺失的末行换行保留。 |
| seq | --equal-width, --format, --separator, --version | 冒烟 + 行为测试 | 选项之后可接负数操作数；-w 补位含符号（经 GNU 实测核对）。 |
| sha1sum | --check, --quiet, --status, --tag, --version, --zero | 冒烟 + 行为测试 | 支持 -z/--zero 和 --tag -z，NUL 模式不转义文件名；-z -c 退出 1。普通校验支持 stdin、精确文件名、GNU 转义、BOM、二进制标记和 BSD tag；读取错误不产生部分数据的成功摘要。不支持 -b/-t、strict 或 NUL 清单解析。 |
| sha256sum | --check, --quiet, --status, --tag, --version, --zero | 冒烟 + 行为测试 | SHA-256，NUL 输出与普通清单校验规则同 sha1sum：-z 禁止文件名转义，-z -c 报错；支持 stdin 清单、精确文件名和读取错误传播。不支持 -b/-t、strict 或 NUL 清单解析。 |
| sha512sum | --check, --quiet, --status, --tag, --version, --zero | 冒烟 + 行为测试 | SHA-512，NUL 输出与普通清单校验规则同 sha1sum；不支持 -b/-t、strict 或 NUL 清单解析。 |
| shred | --force, --iterations, --remove, --verbose, --version | 冒烟 + 行为测试 | 原地覆写（伪随机轮次后末轮清零），仅 -u 才删除；失败传播退出码 1。 |
| shuf | --head-count, --input-range, --output, --random-source, --version, --zero-terminated | 冒烟 + 行为测试 | 支持 -z NUL 记录、-n/-i/-o 和原始字节。--random-source 读取前 64 KiB 后以 FNV 播种，相同字节可复现，但排列不保证与 GNU 相同；随机源不可读时报错。随机行为以记录多重集合校验，不按 GNU 排列逐字节比较。 |
| sleep | --version | 冒烟 + 行为测试 | 接受多个操作数求和，后缀为小写 s/m/h/d；大写后缀与负数退出 1。支持 --help/--version。 |
| sort | --check, --field-separator, --human-numeric-sort, --ignore-case, --key, --numeric-sort, --reverse, --unique, --zero-terminated | 冒烟 + 行为测试 | 支持 -z/--zero-terminated 的原始 NUL 记录；默认 LF 模式保留 CRLF 处理。键、-u/-r/-c/-n/-h 可用于 NUL 记录，默认字段空白包含 space/TAB/LF，CR 仍为内容。比较区分不同非法字节；有效 UTF-8 的大小写折叠保留既有 Unicode 小写规则。不提供完整 locale 排序、外排、字符级键偏移或每键修饰符。 |
| split | --bytes, --lines, --number, --numeric-suffixes, --output, --suffix-length | 冒烟 + 行为测试 | 字母后缀遵循 GNU 顺序（aa..az、ba..）；后缀耗尽报错退出 1；-l 0 被拒绝；行模式字节透明。 |
| stat | --file-system, --format, --terse, --version | 冒烟 + 行为测试 | -c 指令遵循 GNU 语义；设备号/inode/uid/gid/链接数为 0/1 占位，%b 为 ceil(size/512) 近似，%Z 近似 mtime，时间为本地时区秒级，-f 仅输出块与空间行。未知指令报错退出 1。 |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | 冒烟 + 行为测试 | TAB 计入可打印（binutils 规则）；-t 偏移为 7 宽空格填充；8 位模式原样输出高位字节；-f 按书写形式打印路径；编码名大小写敏感。 |
| sum | --bsd, --sysv | 冒烟 + 行为测试 | BSD 模式（默认）输出 5 宽校验和与 1K 块计数、不带文件名；SysV 模式输出校验和、512 块计数与书写形式的操作数。 |
| sync | --version | 冒烟 + 行为测试 | 沿 PATH 解析外部 sync.exe（跳过与本程序相同的候选以防自递归）并传播其退出码；无可用外部命令时回退 rundll32 ProcessIdleTasks（stderr 英文提示）。 |
| tac | --before, --separator, --version | 冒烟 + 行为测试 | 分隔符跟随其记录；尾分隔符不产生空记录，未结尾的末段输出时保持无分隔符。 |
| tail | --bytes, --follow, --lines, --quiet, --verbose, --zero-terminated | 冒烟 + 行为测试 | 支持 -z 的末 N 条/+N 记录选择，不补终止符；-c 仍按字节。-f 会刷新初始 NUL 数据，并在收到 NUL 前即时输出部分追加内容；多文件标题间使用 LF，-v 也作用于 stdin。stdin 不能 follow，其余文件缓存/follow 限制保留。 |
| tar | --create, --extract, --file, --gzip, --list, --verbose, --directory, --to-stdout, --exclude | 冒烟 + 行为测试 | -C 前缀路径、-O 输出内容到 stdout、--exclude 按通配排除条目、成员名选择解包/列表目标。解包拒绝绝对路径、盘符/UNC 前缀或含 `..` 组件的条目（发生拒绝时退出码 2）。 |
| tee | --append, --ignore-interrupts, --version | 冒烟 + 行为测试 | 字节透明（二进制、CRLF、无尾换行输入原样透传）；-i 在 Windows 上无操作。 |
| test | --version | 冒烟 + 行为测试 | POSIX 子集：单操作数字符串、-z/-n、!、数值 -eq/-ne/-lt/-le/-gt/-ge、文件 -e/-f/-d/-s 与 -r/-w/-x（Windows 近似：-r/-x 表示存在，-w 表示非只读）。用法错误退出 2。 |
| time | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| timeout | --kill-after, --version | 冒烟 + 行为测试 | 轮询子进程：正常结束透传退出码，超时返回 124；-k/--kill-after 在宽限期后升级终止。时长支持小数与 s/m/h/d 后缀；子命令允许以 '-' 开头。仅终止直接子进程。 |
| top | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| touch | --no-create, --version | 冒烟 + 行为测试 | 经 SetFileTime 更新已存在文件、目录与只读文件的 atime/mtime；-a 与 -m 选择更新的时间。-d/--date、-t、-r/--reference 未实现。 |
| truncate | --no-create, --reference, --size | 冒烟 + 行为测试 | 尺寸支持 +/- 调整与 K/M/G/T/P/E（1024 进制）或 KB/MB/GB（1000 进制）后缀；+ 表示不小于。--io-blocks 不支持。 |
| tr | --complement, --delete, --squeeze, --version | 冒烟 + 行为测试 | 支持转义（\n、\t、八进制）与 POSIX 字符类（[:alpha:] 等）；-c 作用于替换与挤压；-s 按 SET2 挤压；反序区间报错退出 1。按字符而非字节处理。 |
| traceroute | --max-hops, --nqueries, --timeout, --version | 冒烟 + 行为测试 | 通过 IcmpSendEcho 发送 IPv4 ICMP 探测，超时单位为毫秒；到达目标、收到不可达状态或达到跳数上限时结束。不支持 IPv6、UDP/TCP 探测和反向 DNS。已测环回和有界 TEST-NET 场景，多路由器网络行为未验证。没有 `tracert` 可执行别名。 |
| true | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| tsort | See --help | 冒烟 + 行为测试 | 读取空白分隔的边对；重复边折叠；成环与奇数 token 数按 GNU 措辞（含输入名）报错退出 1。 |
| tty | --all, --silent, --version | 冒烟 + 行为测试 | 经 IsTerminal 判断控制台，名称输出占位 CONIN$；-a 检查三个流（CONIN$/CONOUT$ 占位）；-s 静默。 |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename, --operating-system | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| unexpand | --all, --tabs, --version | 冒烟 + 行为测试 | 至少两个空白才转换为 Tab（GNU 规则）；-t 仅接受单值。 |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version, --zero-terminated | 冒烟 + 行为测试 | 支持 -z NUL 记录，-c/-d/-u/-i/-f 保留分组首条原始记录。跳过字段时以 space/TAB/LF 分界，剩余字节不归一化，CR 保留为内容；非法 UTF-8 记录不混同。有效 UTF-8 的 -i 保留 Unicode 小写规则；不支持输出文件操作数及其他分组扩展。 |
| unlink | （仅自动 help/version） | 冒烟 + 行为测试 | 删除一个文件或文件符号链接条目；拒绝目录，保留符号链接目标。受 Windows ACL 和共享访问规则限制。 |
| unzip | --directory, --list, --overwrite, --quiet, --version | 冒烟 + 行为测试 | 条目流式解压不再整体缓存；-o 覆盖前清除只读位。不安全条目路径（`..`、绝对路径、盘符/UNC 前缀）被拒并警告、跳过；发生拒绝时退出码 2。不恢复条目 mtime。 |
| uptime | 见 --help | 冒烟 + 行为测试 | 经 GetTickCount64 报告开机以来的秒数；-p 输出中文时长。 |
| users | --count, --version | 冒烟 + 行为测试 | 枚举 WTS 会话用户名，按首次出现顺序去重；失败回退 USERNAME（或 unknown）。`-q/--count` 为扩展，与 GNU utmp 排序和重复会话输出不同；不支持 utmp FILE 操作数。 |
| vdir | 同 ls | 冒烟 + 行为测试 | 注入 `ls -l` 的薄别名，支持 `-h`；继承 ls 格式、权限和所有者近似，GNU 长格式的逐字节对比豁免。 |
| vmstat | 见 --help | 冒烟 + 行为测试 | swpd/free 逐帧采样；-h 短名为 help（长名 --header-once 保留）。完整 GNU 行为未验证。 |
| w | --header, --help, --ignore-username, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| watch | --chgexit, --differences, --errexit, --exec, --interval, --no-title, --version | 冒烟 + 行为测试 | ANSI 刷新，支持小数间隔（限制在 0.1..2678400 秒）、变化行高亮、变化或错误时退出。直接执行程序，内建命令/管道需显式启动 shell。标题按 80 列，stderr 拼在 stdout 后，UTF-8 有损解码，`-d` 整行高亮；Ctrl+C 遵循 Windows 控制台行为。 |
| wc | --bytes, --chars, --lines, --max-line-length, --version, --words | 冒烟 + 行为测试 | -m 按 UTF-8 字符计数（无效序列回退字节计数）；-L 不含换行符；打不开的文件报错并退出 1。stdin 输出裸计数，与 GNU 一致。 |
| wget | --directory-prefix, --output-document, --quiet, --version | 冒烟 + 行为测试 | -O - 输出到 stdout；URL 文件名解码（尾 / 取 index.html）；HTTP 错误退出 8；连接超时 10s。HTTPS/TLS 与联网行为尚未验证。 |
| whereis | --version | 冒烟 + 行为测试 | 只搜索 PATH 条目（已移除硬编码个人路径）；输出 `cmd: 路径...` 或裸 `cmd:`；任一命令命中退出 0，否则 1。-m/-s 在 Windows 上无结果。 |
| which | --all, --version | 冒烟 + 行为测试 | 先探测精确名，再依次尝试标准可执行扩展名与 PATHEXT 附加项（大小写不敏感去重）；输出 PATH 书写形式的路径，不解析符号链接。 |
| who | --all, --boot, --dead, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| whoami | --hostname, --username, --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| xargs | --interactive, --max-args, --replace, --verbose, --version, --null | 冒烟 + 行为测试 | -0 读 NUL 分隔输入；分批遵守 Windows 命令行长度上限；空输入仍运行一次命令；-p 从控制台读确认。 |
| yes | --version | 冒烟 + 行为测试 | 多操作数以空格拼接输出；管道关闭静默退出 0。 |
| zcat | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| zip | --recurse, --version | 冒烟 + 行为测试 | PowerShell 5.1 Compress-Archive 外壳；文件与目标参数经单引号转义（内部引号加倍，防注入）。其余 GNU zip 参数不支持。 |


Round 15 同时新增 `ls -h`，等价于 `--human-readable`；帮助入口保留 `--help`。Round 16 为上表 15 个已有命令增加 NUL 模式，命令总数仍为 125。 Round 18 新增 9 个命令（base32/expr/hostid/nproc/pathchk/printenv/sum/truncate/tsort），Round 20 新增 b2sum 与 chgrp，Round 22 新增 dd，命令总数 137；iostat、mpstat、sar 与 w 仍为仅冒烟。

更详细的 ps 参数文档见 [ps README](ps/README-zh.md)。本表由首次源码调查建立，新增或修改命令时需同步更新。