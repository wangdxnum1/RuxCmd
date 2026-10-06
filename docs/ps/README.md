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
pwsh -File scripts/package.ps1
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
- `-F`, `--extra-full` — extra full (`…+rss,psr,comm,args`)
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
| `psr`   | Processor the process is currently assigned to           |
| `wchan` | Wait channel / reason                                    |
| `sz`    | Size in 4 KiB pages                                      |
| `vsz`   | Virtual size in KiB                                      |
| `cputime` / `time` | CPU time (HH:MM:SS / MM:SS)                  |

## Unsupported / best-effort columns

Windows does not expose a 1:1 concept for every Linux field. These are
best-effort mappings (see [Cross-platform mapping](#cross-platform-mapping)):

- `vsz`, `rss` — derive from `GetProcessMemoryInfo`
- `cputime` — derive from `GetProcessTimes`
- `pcpu` — derived from CPU time vs elapsed wall clock since the process started
- `stat` — synthesised from `GetExitCodeProcess` + main window state
- `ni`, `pri` — synthesised from `GetPriorityClass`
- `tty` — Windows has no TTYs; `?` when there is no console window
- `ruser` — same as `user` for now (Windows tokens are per-session)

Fields that have no Windows counterpart (e.g. `wchan` on most GUI apps) are
rendered as `?` rather than omitted.

## Architecture

```
ps/
├── ps-sys/        # raw Win32 FFI (no_std-friendly, thiserror)
│   ├── snapshot   # EnumProcesses + Toolhelp32
│   ├── procinfo   # PEB + NtQueryInformationProcess
│   ├── token      # OpenProcessToken + LookupAccountSidW
│   ├── times      # GetProcessTimes
│   ├── threads    # thread count
│   └── cmdline    # RTL_USER_PROCESS_PARAMETERS from PEB
│
├── ps-core/       # data model + business logic (no Win32)
│   ├── process    # Process struct, Windows → Linux field mapping
│   ├── columns/   # Column trait + 38+ implementations
│   ├── filter     # -p, -u, -C selectors
│   ├── sort       # --sort parser + applier
│   ├── tree       # Forest builder + renderer
│   ├── state      # stat-letter mapping
│   └── user       # user resolution
│
├── ps-bin/        # CLI entry point
│   ├── cli.rs     # clap parser
│   ├── render.rs  # table + forest renderers
│   └── main.rs    # orchestrates snapshot → filter → sort → render
│
└── scripts/
    └── release.ps1
```

## Cross-platform mapping

| Linux concept      | Windows source                                          |
| ------------------ | ------------------------------------------------------- |
| `uid` / `user`     | `OpenProcessToken` + `LookupAccountSidW`                |
| `ppid`             | `ProcessBasicInformation.InheritedFromUniqueProcessId`  |
| `rss` / `vsz`      | `GetProcessMemoryInfo` (WorkingSetSize / PagefileUsage) |
| `pcpu`             | (CPU time) ÷ (uptime since `CreateTime`)                |
| `cputime`          | `GetProcessTimes`                                       |
| `pri` / `ni`       | `GetPriorityClass` mapped to 0..40 range                |
| `stat`             | `GetExitCodeProcess` + main window presence             |
| `start`            | `GetProcessTimes` (create time)                         |
| `cmd` / `args`     | `RTL_USER_PROCESS_PARAMETERS` from PEB                  |
| `tty`              | `GetConsoleWindow` (`?` if null)                        |
| `thcount`          | `Thread32First/Next` count                              |
| `pgid` / `sid`     | `GetProcessId` for both (Windows uses session = sid)    |

## Testing

```powershell
cargo test -p ps-sys -p ps-core
```

Unit tests cover the Win32 layer (`ps-sys`) and the data-model/column layer
(`ps-core`). They do not require any external services and run offline.

## License

MIT — see [LICENSE](LICENSE).
