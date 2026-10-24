# 命令与 Windows 兼容性

[English](compatibility.md) | **简体中文**

当前发行范围为 111 个命令。下表参数来自各 cli.rs 中显式声明的 long 名称，不是完整语法；以该命令 --help 为准。源码声明不等于功能已经通过测试。

所有命令已通过有界发布冒烟检查（帮助入口，true/false 为退出码）。只有标记“行为测试”的命令运行了代表性业务断言；其余业务行为尚未自动验证。未声明完整 GNU/POSIX 兼容性。

| 命令 | 显式长参数示例（源码） | 当前验证 | Windows 差异 / 限制 |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | 仅冒烟；业务待验证 | 已有实现使用子串简化部分匹配；不是完整 awk 解释器。 |
| base64 | --decode, --version, --wrap | 冒烟 + 行为测试 | 解码与 GNU 一致地严格：无填充且长度非 4 倍数的输入会输出已解码字节，但报 invalid input 并退出 1。 |
| basename | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| bc | --version | 冒烟 + 行为测试 | i128 整数运算，/ 与 % 向零截断（GNU scale=0 默认语义）；支持 + - * / % ^ ( )、变量赋值、; 分隔、# 注释与 quit。无任意精度、scale、ibase/obase 与函数——遇到不支持结构时打印提示并跳过该行。banner 为 bc 0.1.0 (RuxCmd)。 |
| cal | --version | 冒烟 + 行为测试 | 单参数 1-12 视为月份，100-9999 视为年份；13-99 被拒绝。完整 GNU 行为未验证。 |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs, --unbuffered | 冒烟 + 行为测试 | 默认字节透明（CRLF、二进制、非 ASCII 数据原样透传）；-v 使用字节级 ^/M- 记号；-E 不给未结尾的末行补 $。行号与空行压缩状态按文件重置。 |
| chmod | --recursive, --verbose, --version | 仅冒烟；业务待验证 | Windows 权限语义与 Unix mode 不等价。 |
| chown | --user, --group, --recursive, --verbose, --version | 冒烟 + 行为测试 | --user 设 owner、--group 设组（GROUP_SECURITY_INFORMATION），可同时给；任一文件失败退出 1。改 owner 为他人通常需要特权。Unix UID/GID 语义不等价。 |
| clear | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cmp | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| comm | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | 冒烟 + 行为测试 | 以 `.`/`..` 结尾的源将内容并入已存在的目标目录（目标不存在时报错）；多源时目标必须是已存在目录。完整 GNU 行为未验证。 |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet, --suffix-format | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| curl | --data, --header, --output, --request, --silent, --version | 仅冒烟；业务待验证 | HTTPS/TLS 与联网行为尚未验证。 |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | 冒烟 + 行为测试 | -d/--date 与位置参数接受 @EPOCH、YYYY-MM-DD、YYYY-MM-DDTHH:MM[:SS]（空串=now，其他格式报错退出 1）；-s/--set 经 SetLocalTime 设置本地时钟。--seconds 作用于解析出的时刻；%N 恒为 0。 |
| df | --human-readable, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| diff | --unified, --ignore-case, --ignore-whitespace, --ignore-blank-lines, --version | 冒烟 + 行为测试 | 退出码遵循 GNU（0/1/2）。普通格式输出未合并的 d/a 命令，行号锚点与 GNU 一致；c 合并与区间折叠未实现。 |
| dirname | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| du | --human-readable, --summarize, --total, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| echo | --version | 冒烟 + 行为测试 | 默认不解释转义（GNU 默认 -E）；-e 开启，-E 强制关闭；两者同给时 -E 生效。 |
| env | --ignore-environment, --unset, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| expand | --initial, --tabs, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| factor | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| false | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| file | 见 --help | 冒烟 + 行为测试 | -b 省略名称前缀；gzip（1F 8B）与 zlib（78 xx）区分识别；可打印 ASCII 与合法 UTF-8 报 ASCII text / UTF-8 text；打不开的操作数报错跳过，结束时如有失败退出 1。 |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| fmt | --crown-margin, --split-only, --version, --width | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| fold | --bytes, --spaces, --version, --width | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching | 冒烟 + 行为测试 | 匹配/未匹配退出码已验证，含单文件 -c 计数；递归错误传播尚未验证。 |
| groups | --version | 冒烟 + 行为测试 | 输出当前用户的真实令牌组（空格分隔，组名与区域相关）；仅支持当前用户，其他名字报错退出 1。 |
| gunzip | --force, --keep, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| head | --bytes, --lines, --quiet, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| hostname | --fqdn, --ip-address, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| iconv | --from-code, --list, --output, --to-code, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| id | --group, --groups, --name, --user, --version | 冒烟 + 行为测试 | uid/gid/groups 取自进程令牌 SID 的 RID；-u/-g/-G/-n 遵循 GNU；组名与区域相关，无法解析时回退为 RID。 |
| iostat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| join | --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| kill | --list, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ln | --force, --symbolic, --version | 仅冒烟；业务待验证 | Windows 符号链接可能需要权限或开发者模式。 |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | 冒烟 + 行为测试 | 长格式附加列（-i/-s 与符号链接目标）按位置索引渲染，着色时不丢失。完整 GNU 行为未验证。 |
| md5sum | --check, --quiet, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mkdir | --mode, --parents, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mkfifo | --mode, --version | 仅冒烟；业务待验证 | Windows 管道语义与 POSIX FIFO 不等价。 |
| mpstat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mv | --force, --interactive, --no-clobber, --verbose | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| netstat | --all, --numeric, --protocol, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| nice | --adjustment, --version | 冒烟 + 行为测试 | 直接 spawn 命令（不依赖 wmic）并应用优先级：adjustment <=-15 High、-14..-5 Above normal、-4..4 Normal、5..14 Below normal、>=15 Idle；透传子进程退出码。 |
| nl | --body-numbering, --number-width, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| nslookup | --type, --version | 冒烟 + 行为测试 | A/AAAA/CNAME/MX/TXT 经真实查询解析（getaddrinfo / DnsQuery_W）；类型名大小写不敏感；不支持的类型报错退出 1。SERVER 位置参数已移除。 |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| open | --application, --edit, --reveal, --version, --wait | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| paste | --delimiters, --serial, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| patch | --force, --reverse, --strip, --version | 冒烟 + 行为测试 | --strip/-p 剥离 N 个前导路径组件（默认 0，GNU 语义）；/dev/null 与全 `@@ -0,0` 补丁会创建目标文件。hunk 匹配不按行计数，每次运行只处理一个目标文件，.rej 内容为错误消息。 |
| pgrep | --full, --list-name, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ping | --count, --size, --timeout, --version | 仅冒烟；业务待验证 | 网络行为及权限要求尚未验证。 |
| pkill | --full, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| printf | --version | 冒烟 + 行为测试 | 实现格式串反斜杠转义（\n \r \t \ \" \0NNN \xHH）、缺参输出空串/0、非法数字参数退出 1。宽度/精度/标志与格式复用未实现。 |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | 冒烟 + 行为测试 | 原生 Windows 进程信息；不可读取字段可能显示 ?。%MEM 使用真实总物理内存（GlobalMemoryStatusEx）；请求未注册列时 stderr 警告并跳过该列。 |
| pwd | --logical, --physical, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| readlink | --canonicalize, --no-newline, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| realpath | --strip-components, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| reset | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| rev | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| rm | --dir, --force, --no-preserve-root, --one-file-system, --preserve-root, --verbose | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| sar | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sed | --expression, --in-place, --separate, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| seq | --equal-width, --format, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sha256sum | --check, --quiet, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| shred | --force, --iterations, --remove, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| shuf | --head-count, --input-range, --output, --random-source, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sleep | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sort | --check, --field-separator, --human-numeric-sort, --ignore-case, --key, --numeric-sort, --reverse, --unique | 冒烟 + 行为测试 | 已实现 -k F1[,F2] 与 -t SEP（单字符）；无 -t 时前导空白归入后一字段；-u 排序后按键去重；-c -r 检查逆序；-n 取前导数字（无则为 0）；-h 为 1024 进制全后缀。不支持字符偏移与按键修饰符（-k2.3/-k2n）。help 仅 --help。 |
| split | --bytes, --lines, --number, --numeric-suffixes, --output, --suffix-length | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| stat | --file-system, --format, --terse, --version | 冒烟 + 行为测试 | -c 指令遵循 GNU 语义；设备号/inode/uid/gid/链接数为 0/1 占位，%b 为 ceil(size/512) 近似，%Z 近似 mtime，时间为本地时区秒级，-f 仅输出块与空间行。未知指令报错退出 1。 |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sync | --version | 冒烟 + 行为测试 | 沿 PATH 解析外部 sync.exe（跳过与本程序相同的候选以防自递归）并传播其退出码；无可用外部命令时回退 rundll32 ProcessIdleTasks（stderr 英文提示）。 |
| tac | --before, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tail | --bytes, --follow, --lines, --quiet, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tar | --create, --extract, --file, --gzip, --list, --verbose | 冒烟 + 行为测试 | 解包拒绝绝对路径、盘符/UNC 前缀或含 `..` 组件的条目（跳过并报错；发生拒绝时退出码 2）。完整 GNU 行为未验证。 |
| tee | --append, --ignore-interrupts, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| test | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| time | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| timeout | --kill-after, --version | 冒烟 + 行为测试 | 轮询子进程：正常结束透传退出码，超时返回 124；-k/--kill-after 在宽限期后升级终止。时长支持小数与 s/m/h/d 后缀；子命令允许以 '-' 开头。仅终止直接子进程。 |
| top | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| touch | --no-create, --version | 冒烟 + 行为测试 | 经 SetFileTime 更新已存在文件、目录与只读文件的 atime/mtime；-a 与 -m 选择更新的时间。-d/--date、-t、-r/--reference 未实现。 |
| tr | --complement, --delete, --squeeze, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| true | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| tty | --all, --silent, --version | 冒烟 + 行为测试 | 经 IsTerminal 判断控制台，名称输出占位 CONIN$；-a 检查三个流（CONIN$/CONOUT$ 占位）；-s 静默。 |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| unexpand | --all, --tabs, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| unzip | --directory, --list, --overwrite, --quiet, --version | 冒烟 + 行为测试 | 不安全条目路径（`..`、绝对路径、盘符/UNC 前缀）被拒并警告、跳过；发生拒绝时退出码 2。不恢复条目 mtime。完整 GNU 行为未验证。 |
| uptime | 见 --help | 冒烟 + 行为测试 | 经 GetTickCount64 报告开机以来的秒数；-p 输出中文时长。 |
| vmstat | 见 --help | 冒烟 + 行为测试 | swpd/free 逐帧采样；-h 短名为 help（长名 --header-once 保留）。完整 GNU 行为未验证。 |
| w | --header, --help, --ignore-username, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| wc | --bytes, --chars, --lines, --max-line-length, --version, --words | 冒烟 + 行为测试 | stdin 输出附带 <stdin> 标签；不是 GNU 的无标签输出。 |
| wget | --directory-prefix, --output-document, --quiet, --version | 仅冒烟；业务待验证 | HTTPS/TLS 与联网行为尚未验证。 |
| whereis | --version | 仅冒烟；业务待验证 | 保留历史固定搜索路径，待专项修正。 |
| which | --all, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| who | --all, --boot, --dead, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| whoami | --hostname, --username, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| xargs | --interactive, --max-args, --replace, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| yes | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| zcat | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| zip | --recurse, --version | 冒烟 + 行为测试 | PowerShell 5.1 Compress-Archive 外壳；文件与目标参数经单引号转义（内部引号加倍，防注入）。其余 GNU zip 参数不支持。 |

更详细的 ps 参数文档见 [ps README](ps/README-zh.md)。本表由首次源码调查建立，新增或修改命令时需同步更新。
