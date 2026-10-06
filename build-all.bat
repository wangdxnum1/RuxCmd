@echo off
setlocal
pushd "%~dp0" || exit /b 1
cargo build --workspace --bins --release --target x86_64-pc-windows-msvc --locked
set "BUILD_EXIT=%errorlevel%"
popd
exit /b %BUILD_EXIT%
