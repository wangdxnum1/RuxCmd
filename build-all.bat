@echo off
setlocal enabledelayedexpansion

set "OUTPUT_DIR=D:\develop\rust-tools"
set "PROJECT_DIR=D:\Work\rust"

echo Creating output directory...
mkdir "%OUTPUT_DIR%" 2>nul

echo.
echo ========================================
echo Building all Rust tools...
echo ========================================
echo.

set "PROJECTS=cat ls open rm touch mkdir cp mv head tail wc grep date which sort cut find du df kill echo ln whoami chmod chown uname uptime env diff sed uniq tee xargs basename dirname tr cmp tar rev split paste nl file md5sum sha256sum stat readlink realpath seq yes sleep id free fmt fold comm join who w hostname groups gunzip unzip zcat patch od strings tty mkfifo true false test printf cal clear reset"
set "BUILD_SUCCESS=true"

for %%p in (%PROJECTS%) do (
    set "PROJECT_PATH=%PROJECT_DIR%\%%p"
    echo Building %%p...
    
    pushd "!PROJECT_PATH!"
    cargo build --release --offline 2>&1
    if !errorlevel! neq 0 (
        echo ERROR: Failed to build %%p
        set "BUILD_SUCCESS=false"
    ) else (
        echo Copying %%p.exe to %OUTPUT_DIR%...
        copy "target\release\%%p.exe" "%OUTPUT_DIR%\" >nul
        if !errorlevel! neq 0 (
            echo ERROR: Failed to copy %%p.exe
            set "BUILD_SUCCESS=false"
        ) else (
            echo Successfully built and copied %%p.exe
        )
    )
    popd
    echo.
)

echo ========================================
if "%BUILD_SUCCESS%"=="true" (
    echo All builds completed successfully!
    echo.
    echo Output directory: %OUTPUT_DIR%
    echo.
    dir "%OUTPUT_DIR%"
) else (
    echo Some builds failed!
    exit /b 1
)
