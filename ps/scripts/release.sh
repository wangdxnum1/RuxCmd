#!/usr/bin/env bash
# scripts/release.sh — POSIX fallback. Windows users should use release.ps1.
set -euo pipefail
cd "$(dirname "$0")/.."

profile="release"
target="${TARGET:-x86_64-pc-windows-msvc}"
bin="ps.exe"
clean="${CLEAN:-0}"
strip_bin="${STRIP:-0}"

if [ "$clean" = "1" ]; then
    echo "[release] cargo clean"
    cargo clean
fi

echo "[release] cargo build --release --target $target"
cargo build --release --target "$target" --locked

src="target/$target/$profile/$bin"
if [ ! -f "$src" ]; then
    echo "Built binary not found at $src" >&2
    exit 1
fi

mkdir -p dist
cp -f "$src" "dist/$bin"

if [ "$strip_bin" = "1" ] && command -v strip >/dev/null 2>&1; then
    echo "[release] strip dist/$bin"
    strip "dist/$bin"
fi

size=$(du -h "dist/$bin" | cut -f1)
echo "[release] done -> dist/$bin ($size)"
