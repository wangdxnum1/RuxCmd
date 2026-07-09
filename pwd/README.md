# pwd — Linux-style working directory printer for Windows (Rust)

A native Windows port of the GNU coreutils `pwd` command. Flags, output
format and error messages follow the Linux original so muscle memory works
unmodified on Windows.

> Outputs the current working directory with forward-slash separators,
> strips the Windows `\\?\` extended-length prefix, and honours `$PWD`
> in logical mode.

## Quick start

```powershell
# from source
cargo build --release
.\target\release\pwd.exe
```

```bash
# default (logical mode)
pwd

# physical mode — resolve all symlinks / junctions
pwd -P

# logical mode — explicit
pwd -L
```

## Flags

| Short | Long          | Description                                                       |
|-------|---------------|-------------------------------------------------------------------|
| `-L`  | `--logical`   | Print `$PWD` if it matches the current directory (default)        |
| `-P`  | `--physical`  | Resolve all symlinks and print the canonical physical path        |
|       | `--help`      | Display help and exit                                             |
|       | `--version`   | Display version and exit                                          |

When both `-L` and `-P` are given, the **last one wins** (matching GNU behaviour).

## Exit codes

| Code | Meaning |
|------|---------|
| `0`  | Success |
| `1`  | Failure (e.g. current directory inaccessible) |

## Error format

```
pwd: error retrieving current directory: <reason>
```

Matches the GNU coreutils error prefix for script compatibility.

## Logical vs physical mode

**Logical mode (`-L`, default):**

1. Read the `PWD` environment variable.
2. If `PWD` is set and resolves to the same directory as `current_dir()`,
   print the `PWD` value (preserves user-facing symlinks / junctions).
3. Otherwise fall back to `std::env::current_dir()`.

**Physical mode (`-P`):**

1. Use `std::fs::canonicalize(".")` to resolve all symlinks, NTFS
   junctions and reparse points.
2. Strip the `\\?\` extended-length prefix.

## Windows-specific handling

- Path separators normalised: `\` → `/`
- `\\?\` extended-length prefix stripped from canonicalized paths
- Graceful error when the working directory has been deleted

## Architecture

```
pwd/
├── src/
│   ├── cli.rs       # clap derive CLI argument definitions
│   └── main.rs      # entry point + path resolution logic
├── Cargo.toml
└── .gitignore
```

## License

MIT — see [LICENSE](LICENSE).
