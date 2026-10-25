# ps — Linux-style process viewer for Windows (Rust)

A native Windows port of the Linux `ps` command. Column names, flags and
shortcuts follow the procps-ng `ps` so muscle memory from Linux works
unmodified on Windows.

> Backed by Win32 (EnumProcesses + Toolhelp32 + PEB).
> Values that cannot be retrieved (denied PEB, missing token, …) are rendered
> as `?` rather than aborting the listing.

## Quick start

```powershell
# from the repository root
cargo build -p ps-bin --release --target x86_64-pc-windows-msvc --locked
.\target\x86_64-pc-windows-msvc\release\ps.exe

# or build the complete verified suite ZIP
pwsh -File scripts/release.ps1
```

```bash
# default columns: PID TTY TIME COMM
ps

# full listing
ps -ef

# custom columns
ps -o pid,user,pcpu,etime,comm

# sort by CPU descending, then PID ascending
ps --sort=-pcpu,pid

# process tree
ps --forest

# filter by PID / user / command
ps -p 1234,5678
ps -u Tim
ps -C chrome

# skip PEB lookup (faster on systems with many privileged processes)
ps -ef --no-cmdline
```

## Flags

### Process selection
- `-e`, `--everyone` — select all processes
- `-A` — alias for `-e`
- `-a` — select all with a tty
- `-x` — include processes with no tty

### Output format
- `-f`, `--full` — full listing (`uid,pid,ppid,c,stime,tty,time,cmd`)
- `-F`, `--extra-full` — extra full (`…+vsz,rss,comm,args`)
- `-l`, `--long` — long format
- `-j`, `--jobs` — jobs format
- `-o`, `--format COLS` — custom columns (comma-separated)
- `--no-headers` — omit the header row
- `--list-columns` — print all available column names and exit
- `--write FILE` — write to file instead of stdout
- `--headers-repeat N` — repeat the header every N rows

### Filters
- `-p`, `--pid LIST` — filter by PID (comma-separated, repeatable)
- `-u`, `--user LIST` — filter by effective user (name or SID)
- `-U`, `--User LIST` — filter by real user (name or SID)
- `-C`, `--comm LIST` — substring match on command name
- `--sort KEY[,KEY…]` — sort by column, prefix with `-` to reverse

### Tree
- `--forest` — show the parent / child hierarchy
- `--ascii` — use ASCII branch characters (default uses UTF-8 box drawing)

### Performance
- `--no-cmdline` — skip the PEB-based command line lookup. Useful when
  many system processes are inaccessible.

## Columns

Run `ps --list-columns` to print the canonical list. Highlights:

| Name    | Description                                              |
| ------- | -------------------------------------------------------- |
| `pid`   | Process ID                                               |
| `ppid`  | Parent process ID                                        |
| `user`  | Effective user (resolved name)                           |
| `ruser` | Real user (resolved name)                                |
| `comm`  | Executable file name (e.g. `powershell.exe`)             |
| `args`  | Full command line, with quoting                          |
| `cmd`   | Truncated command line (`-f` style)                      |
| `pcpu`  | `%CPU` (CPU time / elapsed)                              |
| `pmem`  | `%MEM` (working set / total RAM)                         |
| `vsz`   | Virtual memory size (KiB)                                |
| `rss`   | Resident set size (KiB)                                  |
| `stat`  | Single-letter state, mapped from Windows status          |
| `ni`    | Nice value, mapped from Windows priority class           |
| `pri`   | Priority                                                 |
| `start` / `stime` / `etime` | Start time / start time-of-day / elapsed |
| `tty`   | Controlling terminal (`?` if none)                       |
| `sid`   | Session ID                                               |
| `pgid`  | Process group ID                                         |
| `thcount` / `nlwp` | Number of threads                            |
| `f`     | Flags (process flags)                                    |
| `c`     | Processor utilization (CPU * 100 truncated)              |
| `time` / `times` | CPU time (HH:MM:SS / seconds)                   |

Future columns (P1, not registered yet — requesting one prints
`ps: warning: unknown column '<name>', skipped`): `sz`, `psr`, `wchan`,
`cputime`.

## Unsupported / best-effort columns

Windows does not expose a 1:1 concept for every Linux field. These are
best-effort mappings (see [Cross-platform mapping](#cross-platform-mapping)):

- `vsz`, `rss` — derive from `GetProcessMemoryInfo`
- `pmem` — working set divided by total physical memory from
  `GlobalMemoryStatusEx`
- `time` / `times` — derive from `GetProcessTimes`
- `pcpu` — derived from CPU time vs elapsed wall clock since the process started
- `stat` — synthesised from `GetExitCodeProcess` + main window state
- `ni`, `pri` — synthesised from `GetPriorityClass`
- `tty` — Windows has no TTYs; `?` when there is no console window
- `ruser` — same as `user` for now (Windows tokens are per-session)

Columns whose Linux fields have no Windows counterpart yet (`sz`, `psr`,
`wchan`) are not registered at all — see the future-columns note above —
rather than rendered as `?`.

## Architecture

```text
crates/
├── ps-sys/          # Win32 collector shared by ps, pgrep and pkill
│   └── src/         # snapshots, process data, tokens, times and PEB command lines
└── ps-bin/          # one command package, producing ps.exe
    └── src/
        ├── main.rs / cli.rs
        ├── process.rs                 # internal process and owner model
        ├── assemble.rs / meminfo.rs    # map native data and cache total memory
        ├── columns/                   # static metadata + four formatter groups
        │   ├── mod.rs
        │   ├── identity.rs / resources.rs
        │   └── time.rs / command.rs
        └── filter.rs / sort.rs / tree.rs / render.rs
```

The former single-consumer `ps-core` package is internal to `ps-bin`. Columns
use immutable descriptors and plain functions instead of one trait object and
source file per column. All 42 registered names, including aliases, retain
their headers, widths, alignment and formatting. Sort keys are resolved once
against the selected columns. The archived original design plans describe the
former three-package implementation.

## Cross-platform mapping

| Linux concept      | Windows source                                          |
| ------------------ | ------------------------------------------------------- |
| `uid` / `user`     | `OpenProcessToken` + `LookupAccountSidW`                |
| `ppid`             | Toolhelp32 parent PID  |
| `rss` / `vsz`      | `GetProcessMemoryInfo` (WorkingSetSize / PagefileUsage) |
| `pcpu`             | (CPU time) ÷ (uptime since `CreateTime`)                |
| `time` / `times`   | `GetProcessTimes`                                       |
| `pri` / `ni`       | `GetPriorityClass` mapped to 0..40 range                |
| `stat`             | `S` fallback (per-process runnable state is unavailable)             |
| `start`            | `GetProcessTimes` (create time)                         |
| `cmd` / `args`     | `RTL_USER_PROCESS_PARAMETERS` from PEB                  |
| `tty`              | `?` (controlling TTY is unavailable)                        |
| `thcount`          | `Thread32First/Next` count                              |
| `pgid` / `sid`     | PID for pgid; Windows session ID for sid    |

## Testing

```powershell
cargo test -p ps-bin -p ps-sys -p pgrep -p pkill --all-targets --target x86_64-pc-windows-msvc --locked
```

Tests cover the shared Win32 layer, the internal ps model/columns, and the pgrep/pkill consumers. The 42-column golden fixture and alias/sort/fallback tests preserve pre-refactor behavior. They require normal Windows API access and no external services.

## License

MIT — see [LICENSE](LICENSE).
