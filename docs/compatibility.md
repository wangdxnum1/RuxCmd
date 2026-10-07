# Command and Windows compatibility

**English** | [简体中文](compatibility-zh.md)

The current release covers 111 commands. The options in the table below are the explicit long names declared in each command's `cli.rs`, not the full syntax; the command's `--help` is authoritative. A source declaration does not mean the feature has passed tests.

All commands have passed bounded release smoke checks (help entry, exit codes for true/false). Only commands marked "Smoke + behavior tests" have run representative behavior assertions; the remaining business behavior is not automatically verified. Full GNU/POSIX compatibility is not claimed.

| Command | Explicit long options (from source) | Verification status | Windows differences / limitations |
| --- | --- | --- | --- |
| --- | --- | --- | --- |
| awk | --assign, --field-separator, --file, --version | Smoke only; behavior unverified | The implementation simplifies some matching with substrings; it is not a full awk interpreter. |
| base64 | --decode, --version, --wrap | Smoke only; behavior unverified | Full GNU behavior unverified. |
| basename | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| bc | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cal | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cat | --number, --number-nonblank, --show-all, --show-ends, --show-nonprinting, --show-tabs | Smoke + behavior tests | Full GNU behavior unverified. |
| chmod | --recursive, --verbose, --version | Smoke only; behavior unverified | Windows permission semantics are not equivalent to Unix modes. |
| chown | --version | Smoke only; behavior unverified | Uses the Windows user/security APIs; Unix UID/GID semantics are not equivalent. |
| clear | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cmp | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| column | --output-separator, --separator, --table, --table-header-repeat, --table-right, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| comm | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| cp | --force, --interactive, --preserve, --recursive, --symbolic-link, --verbose | Smoke + behavior tests | Full GNU behavior unverified. |
| csplit | --digits, --elide-empty-files, --keep-files, --prefix, --quiet, --suffix-format | Smoke only; behavior unverified | Full GNU behavior unverified. |
| curl | --data, --header, --output, --request, --silent, --version | Smoke only; behavior unverified | HTTPS/TLS and network behavior are unverified. |
| cut | --bytes, --characters, --delimiter, --fields, --only-delimited, --output-delimiter | Smoke only; behavior unverified | Full GNU behavior unverified. |
| date | --date, --nanoseconds, --seconds, --set, --utc, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| df | --human-readable, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| diff | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| dirname | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| du | --human-readable, --summarize, --total, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| echo | --version | Smoke + behavior tests | Full GNU behavior unverified. |
| env | --ignore-environment, --unset, --verbose, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| expand | --initial, --tabs, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| factor | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| false | --version | Smoke + behavior tests | Full GNU behavior unverified. |
| file | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| find | --basename, --ignore-case, --maxdepth, --mindepth, --name, --regex | Smoke only; behavior unverified | Full GNU behavior unverified. |
| fmt | --crown-margin, --split-only, --version, --width | Smoke only; behavior unverified | Full GNU behavior unverified. |
| fold | --bytes, --spaces, --version, --width | Smoke only; behavior unverified | Full GNU behavior unverified. |
| free | --bytes, --gigabytes, --human-readable, --kilobytes, --megabytes, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| grep | --count, --ignore-case, --invert-match, --line-number, --line-regexp, --only-matching | Smoke + behavior tests | Match/no-match exit codes verified; recursive error propagation unverified. |
| groups | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| gunzip | --force, --keep, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| head | --bytes, --lines, --quiet, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| hexdump | --ascii, --canonical, --decimal, --hexadecimal, --length, --octal | Smoke only; behavior unverified | Full GNU behavior unverified. |
| hostname | --fqdn, --ip-address, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| iconv | --from-code, --list, --output, --to-code, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| id | --group, --groups, --name, --user, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| iostat | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| join | --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| kill | --list, --signal, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| ln | --force, --symbolic, --version | Smoke only; behavior unverified | Windows symbolic links may require privileges or Developer Mode. |
| ls | --all, --almost-all, --author, --block-size, --classify, --color | Smoke only; behavior unverified | Full GNU behavior unverified. |
| md5sum | --check, --quiet, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mkdir | --mode, --parents, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mkfifo | --mode, --version | Smoke only; behavior unverified | Windows pipe semantics are not equivalent to POSIX FIFOs. |
| mpstat | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| mv | --force, --interactive, --no-clobber, --verbose | Smoke + behavior tests | Full GNU behavior unverified. |
| netstat | --all, --numeric, --protocol, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| nice | --adjustment, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| nl | --body-numbering, --number-width, --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| nslookup | --type, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| numfmt | --delimiter, --field, --from, --from-unit, --grouping, --header | Smoke only; behavior unverified | Full GNU behavior unverified. |
| od | --address-radix, --ascii, --decimal, --format, --hexadecimal, --octal | Smoke only; behavior unverified | Full GNU behavior unverified. |
| open | --application, --edit, --reveal, --version, --wait | Smoke only; behavior unverified | Full GNU behavior unverified. |
| paste | --delimiters, --serial, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| patch | --force, --reverse, --strip, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| pgrep | --full, --list-name, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| ping | --count, --size, --timeout, --version | Smoke only; behavior unverified | Network behavior and permission requirements are unverified. |
| pkill | --full, --signal, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| pr | --double-space, --form-feed, --header, --indent, --length, --number-lines | Smoke only; behavior unverified | Full GNU behavior unverified. |
| printf | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| ps | --ascii, --comm, --everyone, --extra-full, --forest, --format | Smoke + behavior tests | Native Windows process information; unreadable fields may show ?. |
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
| stat | --file-system, --format, --terse, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| strings | --all, --bytes, --encoding, --print-file-name, --radix, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| sync | --version | Smoke only; behavior unverified | Currently shells out to the external sync; standalone capability is unverified. |
| tac | --before, --separator, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tail | --bytes, --follow, --lines, --quiet, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tar | --create, --extract, --file, --gzip, --list, --verbose | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tee | --append, --ignore-interrupts, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| test | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| time | --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| timeout | --kill-after, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| top | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| touch | --no-create, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| tr | --complement, --delete, --squeeze, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| true | --version | Smoke + behavior tests | Full GNU behavior unverified. |
| tty | --all, --silent, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| uname | --all, --kernel-name, --kernel-release, --kernel-version, --machine, --nodename | Smoke only; behavior unverified | Full GNU behavior unverified. |
| unexpand | --all, --tabs, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| uniq | --count, --ignore-case, --repeated, --skip-fields, --unique, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| unzip | --directory, --list, --overwrite, --quiet, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |
| uptime | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
| vmstat | See --help | Smoke only; behavior unverified | Full GNU behavior unverified. |
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
| zip | --recurse, --version | Smoke only; behavior unverified | Full GNU behavior unverified. |

See the [ps README](ps/README.md) for detailed ps option documentation. This table was established by the first source survey and must be kept in sync when commands are added or changed.
