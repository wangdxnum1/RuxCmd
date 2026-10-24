# 命令与 Windows 兼容性

[English](compatibility.md) | **简体中文**

当前发行范围为 111 个命令。下表参数来自各 cli.rs 中显式声明的 long 名称，不是完整语法；以该命令 --help 为准。源码声明不等于功能已经通过测试。

所有命令已通过有界发布冒烟检查（帮助入口，true/false 为退出码）。只有标记“行为测试”的命令运行了代表性业务断言；其余业务行为尚未自动验证。未声明完整 GNU/POSIX 兼容性。

| 命令 | 显式长参数示例（源码） | 当前验证 | Windows 差异 / 限制 |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | 冒烟 + 行为测试 | 条件判断、else/next 与多参数 print 可用；NR 跨文件累计、BEGIN/END 仅执行一次；for 循环解析报错。匹配使用子串（非正则）；无数组、内建函数与重定向。 |
| base64 | --decode, --version, --wrap | 冒烟 + 行为测试 | 解码与 GNU 一致地严格：无填充且长度非 4 倍数的输入会输出已解码字节，但报 invalid input 并退出 1。 |
| basename | --suffix, --multiple, --version | 冒烟 + 行为测试 | 支持双操作数 `basename NAME SUFFIX` 形式。尾部 `.`/`..` 组件输出 `.`/`..`；`//` 输出 `/`；盘符根输出 `C:`。 |
| bc | --version | 冒烟 + 行为测试 | i128 整数运算，/ 与 % 向零截断（GNU scale=0 默认语义）；支持 + - * / % ^ ( )、变量赋值、; 分隔、# 注释与 quit。无任意精度、scale、ibase/obase 与函数——遇到不支持结构时打印提示并跳过该行。banner 为 bc 0.1.0 (RuxCmd)。 |
| cal | --version | 冒烟 + 行为测试 | 单参数 1-12 视为月份，100-9999 视为年份；13-99 被拒绝。完整 GNU 行为未验证。 |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs, --unbuffered | 冒烟 + 行为测试 | 默认字节透明（CRLF、二进制、非 ASCII 数据原样透传）；-v 使用字节级 ^/M- 记号；-E 不给未结尾的末行补 $。行号与空行压缩状态按文件重置。 |
| chmod | --recursive, --verbose, --version | 冒烟 + 行为测试 | 符号模式（u/g/o/a 与 +-=rwx）映射到只读位；-R 不再跟随 junction。Windows 权限语义与 Unix mode 不等价。 |
| chown | --user, --group, --recursive, --verbose, --version | 冒烟 + 行为测试 | --user 设 owner、--group 设组（GROUP_SECURITY_INFORMATION），可同时给；任一文件失败退出 1。改 owner 为他人通常需要特权。Unix UID/GID 语义不等价。 |
| clear | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cmp | --silent, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | 冒烟 + 行为测试 | -t 模式未给 -s 时连续空白折叠为单一分隔符；显式 -s 按字符切分不折叠。默认 fill 模式未实现。 |
| comm | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | 冒烟 + 行为测试 | -f 覆盖前清除目标的只读属性。以 `.`/`..` 结尾的源将内容并入已存在的目标目录（目标不存在时报错）；多源时目标必须是已存在目录。 |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet | 冒烟 + 行为测试 | 行号须 >0、升序且不超过文件末尾，否则退出 1。模式仅支持子串与行号；/re/+N、{N} 等 GNU 语法报错退出 1；模式不匹配退出 1（无 -k 时删除输出文件）。--suffix-format 已移除。 |
| curl | --data, --header, --output, --request, --silent, --version, --include, --head, --fail, --location | 冒烟 + 行为测试 | 默认仅输出 body（-i 加响应头）；-d 隐含 POST；重定向默认关闭（-L 手动跟随，最多 10 次）；-f 对 HTTP >= 400 退出 22。HTTPS/TLS 与联网行为尚未验证。 |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter | 冒烟 + 行为测试 | 字节安全：非 UTF-8 行透传，-b 选择原始字节（不支持 -n）；仅剥行尾 \r。完整 GNU 行为未验证。 |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | 冒烟 + 行为测试 | -d/--date 与位置参数接受 @EPOCH、YYYY-MM-DD、YYYY-MM-DDTHH:MM[:SS]（空串=now，其他格式报错退出 1）；-s/--set 经 SetLocalTime 设置本地时钟。--seconds 作用于解析出的时刻；%N 恒为 0。 |
| df | --human-readable, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| diff | --unified, --ignore-case, --ignore-whitespace, --ignore-blank-lines, --version | 冒烟 + 行为测试 | 退出码遵循 GNU（0/1/2）。普通格式输出未合并的 d/a 命令，行号锚点与 GNU 一致；c 合并与区间折叠未实现。 |
| dirname | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| du | --human-readable, --summarize, --total, --version | 冒烟 + 行为测试 | -s 与默认模式都递归求 apparent-size 合计、每个参数输出一行（逐子目录列出仍未实现）；不可读路径报错并退出 1。 |
| echo | --version | 冒烟 + 行为测试 | 默认不解释转义（GNU 默认 -E）；-e 开启，-E 强制关闭；两者同给时 -E 生效。 |
| env | --ignore-environment, --unset, --verbose, --version | 冒烟 + 行为测试 | NAME=VAL 赋值并执行命令（-i 清空、-u 移除、透传退出码、spawn 失败 127）；无命令时打印环境。 |
| expand | --initial, --tabs, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| factor | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| false | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| file | 见 --help | 冒烟 + 行为测试 | -b 省略名称前缀；gzip（1F 8B）与 zlib（78 xx）区分识别；可打印 ASCII 与合法 UTF-8 报 ASCII text / UTF-8 text；打不开的操作数报错跳过，结束时如有失败退出 1。 |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | 冒烟 + 行为测试 | maxdepth 遵循 GNU（0=仅起点，1 含子项）；接受 GNU 单横线长选项（-name、-type 等）；未知 -type 值与遍历错误退出 1；非法 glob 不再 panic。 |
| fmt | --crown-margin, --split-only, --version, --width | 冒烟 + 行为测试 | -c 为保留段落首行缩进的开关；-s 尽量在空白处断行。 |
| fold | --bytes, --spaces, --version, --width | 冒烟 + 行为测试 | 非 UTF-8 输入按 lossy 折行不再报错；-b 按原始字节切分。 |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching, --quiet, --recursive, --extended-regexp, --fixed-strings, --word-regexp, --files-with-matches, --with-filename, --no-filename, --after-context, --before-context, --context | 冒烟 + 行为测试 | 字节安全读行：非 UTF-8 输入按 lossy 匹配并原样输出字节；单文件输出无前缀；-F 按字面匹配。递归错误传播尚未验证。 |
| groups | --version | 冒烟 + 行为测试 | 输出当前用户的真实令牌组（空格分隔，组名与区域相关）；仅支持当前用户，其他名字报错退出 1。 |
| gunzip | --force, --keep, --stdout, --list, --verbose | 冒烟 + 行为测试 | 多成员流完整解压；-c 输出到 stdout 并保留源；-l 输出大小与比率。 |
| head | --bytes, --lines, --quiet, --verbose | 冒烟 + 行为测试 | 行模式字节透明（CRLF、无尾换行、二进制）；-n 与 -c 互斥；-v 对 stdin 打印头。 |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal, --skip | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| hostname | --fqdn, --ip-address, --version | 冒烟 + 行为测试 | 失败时打印错误并退出 1，不再静默。完整 GNU 行为未验证。 |
| iconv | --from-code, --list, --output, --to-code, --version | 冒烟 + 行为测试 | 解码严格遵循 -f（无 BOM 嗅探）；--list 覆盖完整编码表。 |
| id | --group, --groups, --name, --user, --version | 冒烟 + 行为测试 | uid/gid/groups 取自进程令牌 SID 的 RID；-u/-g/-G/-n 遵循 GNU；组名与区域相关，无法解析时回退为 RID。 |
| iostat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| join | --field1, --field2, --separator, --version | 冒烟 + 行为测试 | 输出为键 + 双方剩余字段；无 -t 时按连续空白切分（输出以空格连接）。小于 1 的字段号报错退出 1。 |
| kill | --list, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ln | --force, --symbolic, --verbose, --version | 冒烟 + 行为测试 | 链接目标按书写形式存储（无 \\?\ 前缀）；-v 打印创建结果。Windows 符号链接可能需要权限或开发者模式。 |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | 冒烟 + 行为测试 | 长格式附加列（-i/-s 与符号链接目标）按位置索引渲染，着色时不丢失。完整 GNU 行为未验证。 |
| md5sum | --check, --quiet, --status, --tag, --version | 冒烟 + 行为测试 | check 模式处理 * 标记与 \ 转义名，聚合失败并在任一失败时退出 1。 |
| mkdir | --mode, --parents, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mkfifo | --mode, --version | 仅冒烟；业务待验证 | Windows 管道语义与 POSIX FIFO 不等价。 |
| mpstat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mv | --force, --interactive, --no-clobber, --verbose | 冒烟 + 行为测试 | 拒绝以非目录覆盖目录（绝不删除目标树）；跨卷回退为 copy+delete；src/. 并入已存在目录。 |
| netstat | --all, --numeric, --protocol, --version, --own-pid | 冒烟 + 行为测试 | -a 控制是否显示 LISTENING/未绑定条目；-o 增加所属 PID 列（GetExtendedTcp/UdpTable）；表查询错误退出 1。 |
| nice | --adjustment, --version | 冒烟 + 行为测试 | 直接 spawn 命令（不依赖 wmic）并应用优先级：adjustment <=-15 High、-14..-5 Above normal、-4..4 Normal、5..14 Below normal、>=15 Idle；透传子进程退出码。 |
| nl | --body-numbering, --number-width, --separator, --version | 冒烟 + 行为测试 | 按 UTF-8 行读取不乱码；-b 接受 t（默认，仅非空行）、n（不编号）、a（全部行）。 |
| nslookup | --type, --version | 冒烟 + 行为测试 | A/AAAA/CNAME/MX/TXT 经真实查询解析（getaddrinfo / DnsQuery_W）；类型名大小写不敏感；不支持的类型报错退出 1。SERVER 位置参数已移除。 |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header, --padding, --round, --suffix, --to, --to-unit | 冒烟 + 行为测试 | 显式 -d 在输出中保留且不折叠连续分隔符；--from=auto 读取 K=1000、Ki=1024。 |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal, --read-bytes, --skip-bytes, --width | 冒烟 + 行为测试 | -t 支持 x1/d1/o1/u1/a/c/z 与 f4/f8（GNU 字段宽度、小端序）；仅格式含 c 时输出 ASCII 列。 |
| open | --application, --edit, --reveal, --version, --wait | 冒烟 + 行为测试 | 经 ShellExecuteW 打开（无 shell 引号问题）；-R 忽略 explorer 退出码；-W 在启动返回进程句柄时等待。 |
| paste | --delimiters, --serial, --version | 冒烟 + 行为测试 | 剥行尾 \r；-d 接受按输出列循环的字符列表。 |
| patch | --force, --reverse, --strip, --version | 冒烟 + 行为测试 | --strip/-p 剥离 N 个前导路径组件（默认 0）；hunk 严格按 @@ 声明计数消费（处理空上下文行与 \\ No newline）；多文件补丁分段应用；.rej 为被拒 hunk 原文；成功的 hunk 写回。 |
| pgrep | --full, --list-name, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ping | --count, --size, --version | 冒烟 + 行为测试 | 全部丢包退出 1；失败状态分类提示；-s 上限 65500；-t 为持续 ping（Windows 语义），-w 为单次应答超时。 |
| pkill | --full, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | 冒烟 + 行为测试 | 页眉显示真实本地日期时间；-n 遵循 GNU -n[SEP][DIGITS]（裸 -n 为 TAB + 宽 5，非法规格退出 1）；help 仅 --help（-h 是页眉）。 |
| printf | --version | 冒烟 + 行为测试 | 实现格式串反斜杠转义（\n \r \t \ \" \0NNN \xHH）、缺参输出空串/0、非法数字参数退出 1。宽度/精度/标志与格式复用未实现。 |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | 冒烟 + 行为测试 | 原生 Windows 进程信息；不可读取字段可能显示 ?。%MEM 使用真实总物理内存（GlobalMemoryStatusEx）；请求未注册列时 stderr 警告并跳过该列。 |
| pwd | --logical, --physical, --version | 冒烟 + 行为测试 | UNC 路径输出为 //server/share。完整 GNU 行为未验证。 |
| readlink | --canonicalize, --no-newline, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| realpath | --strip-components, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| reset | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| rev | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| rm | --dir, --force, --no-preserve-root, --one-file-system, --preserve-root, --verbose | 冒烟 + 行为测试 | -f 仅静默不存在的路径；其余失败均打印错误并退出 1，preserve-root 拒绝信息不被压制。 |
| sar | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sed | --expression, --quiet, --in-place, --separate, --version | 冒烟 + 行为测试 | 支持位置参数脚本、p/d/q 命令与行号、$、/子串/ 及范围地址；-i 原子写并支持备份后缀；未知命令报错。s 与地址为字面子串（无正则、无 &/\1）；行号按文件重置。 |
| seq | --equal-width, --format, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sha256sum | --check, --quiet, --status, --tag, --version | 冒烟 + 行为测试 | 同 md5sum：* 标记、BOM 剥离、聚合退出码，并支持 `-` 读 stdin。 |
| shred | --force, --iterations, --remove, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| shuf | --head-count, --input-range, --output, --random-source, --version | 冒烟 + 行为测试 | --random-source 以文件前 64 KiB 内容（FNV）播种：相同内容复现相同洗牌；源文件不可读时报错退出 1。 |
| sleep | --version | 冒烟 + 行为测试 | 支持小数时长（0.5）；拒绝负数与溢出区间。 |
| sort | --check, --field-separator, --human-numeric-sort, --ignore-case, --key, --numeric-sort, --reverse, --unique | 冒烟 + 行为测试 | 已实现 -k F1[,F2] 与 -t SEP（单字符）；无 -t 时前导空白归入后一字段；-u 排序后按键去重；-c -r 检查逆序；-n 取前导数字（无则为 0）；-h 为 1024 进制全后缀。字节安全排序并原样输出；`-` 读 stdin；打不开的操作数退出 2。不支持字符偏移与按键修饰符（-k2.3/-k2n）。help 仅 --help。 |
| split | --bytes, --lines, --number, --numeric-suffixes, --output, --suffix-length | 冒烟 + 行为测试 | 字母后缀遵循 GNU 顺序（aa..az、ba..）；后缀耗尽报错退出 1；-l 0 被拒绝；行模式字节透明。 |
| stat | --file-system, --format, --terse, --version | 冒烟 + 行为测试 | -c 指令遵循 GNU 语义；设备号/inode/uid/gid/链接数为 0/1 占位，%b 为 ceil(size/512) 近似，%Z 近似 mtime，时间为本地时区秒级，-f 仅输出块与空间行。未知指令报错退出 1。 |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sync | --version | 冒烟 + 行为测试 | 沿 PATH 解析外部 sync.exe（跳过与本程序相同的候选以防自递归）并传播其退出码；无可用外部命令时回退 rundll32 ProcessIdleTasks（stderr 英文提示）。 |
| tac | --before, --separator, --version | 冒烟 + 行为测试 | 分隔符跟随其记录；尾分隔符不产生空记录，未结尾的末段输出时保持无分隔符。 |
| tail | --bytes, --follow, --lines, --quiet, --verbose | 冒烟 + 行为测试 | 无操作数或 `-` 时读 stdin；-n/-c 支持 +N 偏移；-c -f 也可跟随；多文件 -f 轮询全部文件并带 GNU 风格头；非法数值退出 1。stdin 不可 follow。 |
| tar | --create, --extract, --file, --gzip, --list, --verbose, --directory, --to-stdout, --exclude | 冒烟 + 行为测试 | -C 前缀路径、-O 输出内容到 stdout、--exclude 按通配排除条目、成员名选择解包/列表目标。解包拒绝绝对路径、盘符/UNC 前缀或含 `..` 组件的条目（发生拒绝时退出码 2）。 |
| tee | --append, --ignore-interrupts, --version | 冒烟 + 行为测试 | 字节透明（二进制、CRLF、无尾换行输入原样透传）；-i 在 Windows 上无操作。 |
| test | --version | 冒烟 + 行为测试 | POSIX 子集：单操作数字符串、-z/-n、!、数值 -eq/-ne/-lt/-le/-gt/-ge、文件 -e/-f/-d/-s 与 -r/-w/-x（Windows 近似：-r/-x 表示存在，-w 表示非只读）。用法错误退出 2。 |
| time | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| timeout | --kill-after, --version | 冒烟 + 行为测试 | 轮询子进程：正常结束透传退出码，超时返回 124；-k/--kill-after 在宽限期后升级终止。时长支持小数与 s/m/h/d 后缀；子命令允许以 '-' 开头。仅终止直接子进程。 |
| top | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| touch | --no-create, --version | 冒烟 + 行为测试 | 经 SetFileTime 更新已存在文件、目录与只读文件的 atime/mtime；-a 与 -m 选择更新的时间。-d/--date、-t、-r/--reference 未实现。 |
| tr | --complement, --delete, --squeeze, --version | 冒烟 + 行为测试 | 支持转义（\n、\t、八进制）与 POSIX 字符类（[:alpha:] 等）；-c 作用于替换与挤压；-s 按 SET2 挤压；反序区间报错退出 1。按字符而非字节处理。 |
| true | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| tty | --all, --silent, --version | 冒烟 + 行为测试 | 经 IsTerminal 判断控制台，名称输出占位 CONIN$；-a 检查三个流（CONIN$/CONOUT$ 占位）；-s 静默。 |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename, --operating-system | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| unexpand | --all, --tabs, --version | 冒烟 + 行为测试 | 至少两个空白才转换为 Tab（GNU 规则）；-t 仅接受单值。 |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version | 冒烟 + 行为测试 | 比较键与输出行分离：-i 与 -f 只影响比较，输出保持原始行。 |
| unzip | --directory, --list, --overwrite, --quiet, --version | 冒烟 + 行为测试 | 条目流式解压不再整体缓存；-o 覆盖前清除只读位。不安全条目路径（`..`、绝对路径、盘符/UNC 前缀）被拒并警告、跳过；发生拒绝时退出码 2。不恢复条目 mtime。 |
| uptime | 见 --help | 冒烟 + 行为测试 | 经 GetTickCount64 报告开机以来的秒数；-p 输出中文时长。 |
| vmstat | 见 --help | 冒烟 + 行为测试 | swpd/free 逐帧采样；-h 短名为 help（长名 --header-once 保留）。完整 GNU 行为未验证。 |
| w | --header, --help, --ignore-username, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| wc | --bytes, --chars, --lines, --max-line-length, --version, --words | 冒烟 + 行为测试 | -m 按 UTF-8 字符计数（无效序列回退字节计数）；-L 不含换行符；打不开的文件报错并退出 1。stdin 输出附带 <stdin> 标签；不是 GNU 的无标签输出。 |
| wget | --directory-prefix, --output-document, --quiet, --version | 冒烟 + 行为测试 | -O - 输出到 stdout；URL 文件名解码（尾 / 取 index.html）；HTTP 错误退出 8；连接超时 10s。HTTPS/TLS 与联网行为尚未验证。 |
| whereis | --version | 仅冒烟；业务待验证 | 保留历史固定搜索路径，待专项修正。 |
| which | --all, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| who | --all, --boot, --dead, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| whoami | --hostname, --username, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| xargs | --interactive, --max-args, --replace, --verbose, --version, --null | 冒烟 + 行为测试 | -0 读 NUL 分隔输入；分批遵守 Windows 命令行长度上限；空输入仍运行一次命令；-p 从控制台读确认。 |
| yes | --version | 冒烟 + 行为测试 | 多操作数以空格拼接输出；管道关闭静默退出 0。 |
| zcat | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| zip | --recurse, --version | 冒烟 + 行为测试 | PowerShell 5.1 Compress-Archive 外壳；文件与目标参数经单引号转义（内部引号加倍，防注入）。其余 GNU zip 参数不支持。 |

更详细的 ps 参数文档见 [ps README](ps/README-zh.md)。本表由首次源码调查建立，新增或修改命令时需同步更新。
