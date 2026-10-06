# 命令与 Windows 兼容性

当前发行范围为 111 个命令。下表参数来自各 cli.rs 中显式声明的 long 名称，不是完整语法；以该命令 --help 为准。源码声明不等于功能已经通过测试。

所有命令已通过有界发布冒烟检查（帮助入口，true/false 为退出码）。只有标记“行为测试”的命令运行了代表性业务断言；其余业务行为尚未自动验证。未声明完整 GNU/POSIX 兼容性。

| 命令 | 显式长参数示例（源码） | 当前验证 | Windows 差异 / 限制 |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | 仅冒烟；业务待验证 | 已有实现使用子串简化部分匹配；不是完整 awk 解释器。 |
| base64 | --decode, --version, --wrap | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| basename | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| bc | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cal | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| chmod | --recursive, --verbose, --version | 仅冒烟；业务待验证 | Windows 权限语义与 Unix mode 不等价。 |
| chown | --version | 仅冒烟；业务待验证 | 使用 Windows 用户/安全 API，Unix UID/GID 语义不等价。 |
| clear | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cmp | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| comm | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet, --suffix-format | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| curl | --data, --header, --output, --request, --silent, --version | 仅冒烟；业务待验证 | HTTPS/TLS 与联网行为尚未验证。 |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| df | --human-readable, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| diff | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| dirname | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| du | --human-readable, --summarize, --total, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| echo | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| env | --ignore-environment, --unset, --verbose, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| expand | --initial, --tabs, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| factor | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| false | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| file | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| fmt | --crown-margin, --split-only, --version, --width | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| fold | --bytes, --spaces, --version, --width | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching | 冒烟 + 行为测试 | 匹配/未匹配退出码已验证；递归错误传播尚未验证。 |
| groups | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| gunzip | --force, --keep, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| head | --bytes, --lines, --quiet, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| hostname | --fqdn, --ip-address, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| iconv | --from-code, --list, --output, --to-code, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| id | --group, --groups, --name, --user, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| iostat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| join | --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| kill | --list, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ln | --force, --symbolic, --version | 仅冒烟；业务待验证 | Windows 符号链接可能需要权限或开发者模式。 |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| md5sum | --check, --quiet, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mkdir | --mode, --parents, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mkfifo | --mode, --version | 仅冒烟；业务待验证 | Windows 管道语义与 POSIX FIFO 不等价。 |
| mpstat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| mv | --force, --interactive, --no-clobber, --verbose | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| netstat | --all, --numeric, --protocol, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| nice | --adjustment, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| nl | --body-numbering, --number-width, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| nslookup | --type, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| open | --application, --edit, --reveal, --version, --wait | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| paste | --delimiters, --serial, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| patch | --force, --reverse, --strip, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| pgrep | --full, --list-name, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ping | --count, --size, --timeout, --version | 仅冒烟；业务待验证 | 网络行为及权限要求尚未验证。 |
| pkill | --full, --signal, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| printf | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | 冒烟 + 行为测试 | 原生 Windows 进程信息；不可读取字段可能显示 ?。 |
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
| sort | --check, --field-separator, --human-numeric-sort, --ignore-case, --key, --numeric-sort | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| split | --bytes, --lines, --number, --numeric-suffixes, --output, --suffix-length | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| stat | --file-system, --format, --terse, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| sync | --version | 仅冒烟；业务待验证 | 当前业务调用外部 sync，独立使用能力尚未验证。 |
| tac | --before, --separator, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tail | --bytes, --follow, --lines, --quiet, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tar | --create, --extract, --file, --gzip, --list, --verbose | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tee | --append, --ignore-interrupts, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| test | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| time | --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| timeout | --kill-after, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| top | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| touch | --no-create, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| tr | --complement, --delete, --squeeze, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| true | --version | 冒烟 + 行为测试 | 完整 GNU 行为未验证。 |
| tty | --all, --silent, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| unexpand | --all, --tabs, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| unzip | --directory, --list, --overwrite, --quiet, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| uptime | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
| vmstat | 见 --help | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |
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
| zip | --recurse, --version | 仅冒烟；业务待验证 | 完整 GNU 行为未验证。 |

更详细的 ps 参数文档见 [ps README](ps/README-zh.md)。本表由首次源码调查建立，新增或修改命令时需同步更新。
