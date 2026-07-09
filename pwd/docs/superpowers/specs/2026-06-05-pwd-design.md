# pwd — Linux pwd Command in Rust

## Overview

A faithful Rust port of GNU coreutils `pwd` for Windows, replicating all flags, output format, error messages, and exit codes of the Linux original.

## Project Structure

```
pwd/
├── src/
│   ├── cli.rs       # clap derive CLI argument definitions
│   └── main.rs      # entry point + core logic
├── Cargo.toml
└── .gitignore
```

Single crate, edition 2024. Binary name: `pwd`.

## Feature Parity with GNU coreutils pwd

### Flags

| Short | Long        | Behavior                                                       |
|-------|-------------|----------------------------------------------------------------|
| `-L`  | `--logical` | Print `PWD` env var if valid (preserves symlinks). **Default** |
| `-P`  | `--physical`| Resolve all symlinks; print the physical path                  |
|       | `--help`    | Display help and exit                                          |
|       | `--version` | Display version and exit                                       |

### Exit Codes

- `0` — success
- `1` — failure (e.g., current directory inaccessible)

### Error Format

```
pwd: error retrieving current directory: <reason>
```

Matches GNU coreutils error prefix for script compatibility.

## Core Logic

### Logical path (`-L`, default)

1. Read `PWD` environment variable.
2. If `PWD` is set, canonicalize both `PWD` and `std::env::current_dir()` and compare.
3. If they resolve to the same directory → print `PWD` value (preserves user-facing symlinks/junctions).
4. Otherwise → fall back to `std::env::current_dir()`.

### Physical path (`-P`)

1. Use `std::fs::canonicalize(".")` to resolve all symlinks / NTFS junctions / reparse points.
2. Strip the `\\?\` extended-length prefix that Windows `canonicalize` returns.

### Windows-specific Handling

- Normalize path separators: `\` → `/` for Linux-style output.
- Strip `\\?\` prefix from canonicalized paths.
- Handle case where `current_dir()` fails (e.g., deleted working directory).

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

## CLI Definition (cli.rs)

```rust
use clap::Parser;

/// Print the full filename of the current working directory.
#[derive(Parser, Debug)]
#[command(name = "pwd", version, about)]
pub struct Args {
    /// Print the value of $PWD if it matches the current working directory
    #[arg(short = 'L', long = "logical")]
    pub logical: bool,

    /// Print the physical working directory (resolve all symlinks)
    #[arg(short = 'P', long = "physical")]
    pub physical: bool,
}
```

When both `-L` and `-P` are given, the last one wins (matching GNU behavior via clap `overrides_with`).

## Testing

- Manual: verify output matches `cmd /c cd` and `powershell (Get-Location).Path`.
- Edge cases: deleted CWD, junction points, subst drives, paths with spaces.
