#!/usr/bin/env bash
# Compatibility entry for Git Bash on Windows; delegates to suite packaging.
set -euo pipefail
repository=$(cd "$(dirname "$0")/../../.." && pwd)
exec pwsh -File "$repository/scripts/package.ps1" "$@"
