# Command and Windows compatibility

**English** | [简体中文](compatibility-zh.md)

The current release covers 111 commands. The options in the table below are the explicit long names declared in each command's `cli.rs`, not the full syntax; the command's `--help` is authoritative. A source declaration does not mean the feature has passed tests.

All commands have passed bounded release smoke checks (help entry, exit codes for true/false). Only commands marked "Smoke + behavior tests" have run representative behavior assertions; the remaining business behavior is not automatically verified. Full GNU/POSIX compatibility is not claimed.

| Command | Explicit long options (from source) | Verification status | Windows differences / limitations |
| --- | --- | --- | --- |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | Smoke only; behavior unverified | The implementation simplifies some matching with substrings; it is not a full awk interpreter. |
| base64 | --decode, --version, --wrap | Smoke + behavior tests | Decoding is strict like GNU: input without padding that is not a multiple of 4 produces decoded bytes but reports invalid input and exits 1. |
| basename | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| bc | --version | Smoke + behavior tests | Integer arithmetic on i128 with truncating / and % (GNU scale=0 default); supports + - * / % ^ ( ), variable assignment, ; separators, # comments, and quit. No arbitrary precision, scale, ibase/obase, or functions - unsupported constructs print a notice and the line is skipped. Banner is bc 0.1.0 (RuxCmd). |
| cal | --version | Smoke + behavior tests | A single argument 1-12 is a month, 100-9999 a year; 13-99 is rejected. Full GNU behavior unverified. |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs, --unbuffered | Smoke + behavior tests | Byte-transparent by default (CRLF, binary, and non-ASCII data pass through unchanged); -v uses byte-level ^/M- notation; -E does not mark an unterminated last line. Line numbering and empty-line compression state reset per file. |
| chmod | --recursive, --verbose, --version | Smoke only; behavior unverified | Windows permission semantics are not equivalent to Unix modes. |
| chown | --user, --group, --recursive, --verbose, --version | Smoke + behavior tests | --user sets the owner and --group the group (GROUP_SECURITY_INFORMATION), both may be given; any file failure exits 1. Changing the owner to another account usually requires privileges. Unix UID/GID semantics are not equivalent. |
| clear | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cmp | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| comm | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | Smoke + behavior tests | Sources ending in `.`/`..` merge their contents into an existing destination directory (error if it does not exist); with multiple sources the destination must be an existing directory. Full GNU behavior unverified. |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet, --suffix-format | Smoke only; behavior unverified | Full GNU behavior unverified. |
| curl | --data, --header, --output, --request, --silent, --version | Smoke only; behavior unverified | HTTPS/TLS and network behavior are unverified. |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter | Smoke only; behavior unverified | Full GNU behavior unverified. |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | Smoke + behavior tests | -d/--date and the positional operand accept @EPOCH, YYYY-MM-DD, and YYYY-MM-DDTHH:MM[:SS] (empty = now, other formats error with exit 1); -s/--set sets the local clock via SetLocalTime. --seconds applies to the parsed instant; %N is always 0. |
| df | --human-readable, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| diff | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| dirname | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| du | --human-readable, --summarize, --total, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| echo | --version | Smoke + behavior tests | Escapes are off by default (GNU -E default); -e enables them, -E forces off. When both are given, -E wins. |
| env | --ignore-environment, --unset, --verbose, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| expand | --initial, --tabs, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| factor | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| false | --version | Smoke + behavior tests | Full GNU behavior unverified. |
| file | See --help | Smoke + behavior tests | -b omits the name prefix; gzip (1F 8B) and zlib (78 xx) are distinguished; printable ASCII and UTF-8 report ASCII text / UTF-8 text; operands that cannot be opened are skipped with an error and exit 1 at the end. |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | Smoke only; behavior unverified | Full GNU behavior unverified. |
| fmt | --crown-margin, --split-only, --version, --width | Smoke only; behavior unverified | Full GNU behavior unverified. |
| fold | --bytes, --spaces, --version, --width | Smoke only; behavior unverified | Full GNU behavior unverified. |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching | Smoke + behavior tests | Match/no-match exit codes verified, including single-file -c counts; recursive error propagation unverified. |
| groups | --version | Smoke + behavior tests | Prints the current user's real token groups (space-separated, locale-dependent names); only the current user is supported and other names error with exit 1. |
| gunzip | --force, --keep, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| head | --bytes, --lines, --quiet, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal | Smoke only; behavior unverified | Full GNU behavior unverified. |
| hostname | --fqdn, --ip-address, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| iconv | --from-code, --list, --output, --to-code, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| id | --group, --groups, --name, --user, --version | Smoke + behavior tests | uid/gid/groups derive from the process token SID RIDs; -u/-g/-G/-n follow GNU; group names are locale-dependent and fall back to the RID. |
| iostat | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| join | --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| kill | --list, --signal, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| ln | --force, --symbolic, --version | Smoke only; behavior unverified | Windows symbolic links may require privileges or Developer Mode. |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | Smoke + behavior tests | Long-format extra columns (-i/-s and symlink targets) are rendered by position index and are not lost when coloring is active. Full GNU behavior unverified. |
| md5sum | --check, --quiet, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mkdir | --mode, --parents, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mkfifo | --mode, --version | Smoke only; behavior unverified | Windows pipe semantics are not equivalent to POSIX FIFOs. |
| mpstat | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mv | --force, --interactive, --no-clobber, --verbose | Smoke + behavior tests | Full GNU behavior unverified. |
| netstat | --all, --numeric, --protocol, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| nice | --adjustment, --version | Smoke + behavior tests | Spawns the command directly (no wmic) and applies a priority class: adjustment <=-15 High, -14..-5 Above normal, -4..4 Normal, 5..14 Below normal, >=15 Idle; the child exit code is passed through. |
| nl | --body-numbering, --number-width, --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| nslookup | --type, --version | Smoke + behavior tests | Types A/AAAA/CNAME/MX/TXT resolve for real (getaddrinfo / DnsQuery_W); type names are case-insensitive; unsupported types error with exit 1. The SERVER positional argument was removed. |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header | Smoke only; behavior unverified | Full GNU behavior unverified. |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal | Smoke only; behavior unverified | Full GNU behavior unverified. |
| open | --application, --edit, --reveal, --version, --wait | Smoke only; behavior unverified | Full GNU behavior unverified. |
| paste | --delimiters, --serial, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| patch | --force, --reverse, --strip, --version | Smoke + behavior tests | --strip/-p strips N leading path components (default 0, GNU semantics); /dev/null and all-@@ -0,0 patches create the target file. Hunks are matched without line counts, one target file is used per run, and .rej files contain the error message. |
| pgrep | --full, --list-name, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| ping | --count, --size, --timeout, --version | Smoke only; behavior unverified | Network behavior and permission requirements are unverified. |
| pkill | --full, --signal, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | Smoke only; behavior unverified | Full GNU behavior unverified. |
| printf | --version | Smoke + behavior tests | Implements format-string backslash escapes (\n \r \t \ \" \0NNN \xHH), empty string/0 for missing arguments, and exit code 1 for invalid numeric arguments. Width/precision/flags and format reuse are not implemented. |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | Smoke + behavior tests | Native Windows process information; unreadable fields may show ?. %MEM uses the real total physical memory (GlobalMemoryStatusEx); requesting an unregistered column warns on stderr and skips it. |
| pwd | --logical, --physical, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| readlink | --canonicalize, --no-newline, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| realpath | --strip-components, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| reset | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| rev | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| rm | --dir, --force, --no-preserve-root, --one-file-system, --preserve-root, --verbose | Smoke + behavior tests | Full GNU behavior unverified. |
| sar | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sed | --expression, --in-place, --separate, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| seq | --equal-width, --format, --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sha256sum | --check, --quiet, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| shred | --force, --iterations, --remove, --verbose, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| shuf | --head-count, --input-range, --output, --random-source, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sleep | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sort | --check, --field-separator, --human-numeric-sort, --ignore-case, --key, --numeric-sort | Smoke only; behavior unverified | Full GNU behavior unverified. |
| split | --bytes, --lines, --number, --numeric-suffixes, --output, --suffix-length | Smoke only; behavior unverified | Full GNU behavior unverified. |
| stat | --file-system, --format, --terse, --version | Smoke + behavior tests | -c directives follow GNU semantics; device/inode/uid/gid/link fields are 0/1 placeholders, %b is ceil(size/512), %Z approximates mtime, times are local with second precision, and -f prints block/space lines only. Unknown directives error with exit 1. |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sync | --version | Smoke + behavior tests | Resolves an external sync.exe along PATH (candidates identical to this executable are skipped to prevent self-recursion) and propagates its exit code; falls back to rundll32 ProcessIdleTasks with a stderr notice when no external command is available. |
| tac | --before, --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tail | --bytes, --follow, --lines, --quiet, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tar | --create, --extract, --file, --gzip, --list, --verbose | Smoke + behavior tests | Extraction rejects entries with absolute paths, drive/UNC prefixes, or '..' components (skipped with an error; exit code 2 when any entry was rejected). Full GNU behavior unverified. |
| tee | --append, --ignore-interrupts, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| test | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| time | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| timeout | --kill-after, --version | Smoke + behavior tests | Polls the child and returns its exit code, or 124 on expiry; -k/--kill-after escalates after the grace period. Durations accept decimals with s/m/h/d suffixes; the command may start with '-'. Only the direct child is killed. |
| top | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| touch | --no-create, --version | Smoke + behavior tests | Updates atime/mtime via SetFileTime on existing files, directories, and read-only files; -a and -m select which time to update. -d/--date, -t, and -r/--reference are not implemented. |
| tr | --complement, --delete, --squeeze, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| true | --version | Smoke + behavior tests | Full GNU behavior unverified. |
| tty | --all, --silent, --version | Smoke + behavior tests | Detects a console via IsTerminal and prints CONIN$ as the name placeholder; -a checks all three streams with CONIN$/CONOUT$ placeholders; -s is quiet. |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename | Smoke only; behavior unverified | Full GNU behavior unverified. |
| unexpand | --all, --tabs, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| unzip | --directory, --list, --overwrite, --quiet, --version | Smoke + behavior tests | Unsafe entry paths (`..`, absolute, drive/UNC prefixes) are rejected with a warning and skipped; exit code 2 when any entry was rejected. Entry mtimes are not restored. Full GNU behavior unverified. |
| uptime | See --help | Smoke + behavior tests | Reports seconds since boot via GetTickCount64; -p prints a Chinese-language duration. |
| vmstat | See --help | Smoke + behavior tests | swpd/free are sampled every frame; the -h short option is help (--header-once remains the long name). Full GNU behavior unverified. |
| w | --header, --help, --ignore-username, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| wc | --bytes, --chars, --lines, --max-line-length, --version, --words | Smoke + behavior tests | stdin output carries a <stdin> label, unlike GNU's unlabeled output. |
| wget | --directory-prefix, --output-document, --quiet, --version | Smoke only; behavior unverified | HTTPS/TLS and network behavior are unverified. |
| whereis | --version | Smoke only; behavior unverified | Keeps historical fixed search paths; a dedicated fix is pending. |
| which | --all, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| who | --all, --boot, --dead, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| whoami | --hostname, --username, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| xargs | --interactive, --max-args, --replace, --verbose, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| yes | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| zcat | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| zip | --recurse, --version | Smoke + behavior tests | Wrapper around PowerShell 5.1 Compress-Archive; file and destination arguments are single-quoted with embedded quotes doubled (injection-safe). Other GNU zip flags unsupported. |

See the [ps README](ps/README.md) for detailed ps option documentation. This table was established by the first source survey and must be kept in sync when commands are added or changed.
