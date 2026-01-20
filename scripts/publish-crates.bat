@echo off
REM =============================================================================
REM Publish Hope OS to crates.io
REM =============================================================================

echo.
echo  Hope OS - Publishing to crates.io
echo  ()=^>[]
echo.

REM Run tests first
echo Running tests...
cargo test --release
if errorlevel 1 goto :error

REM Check clippy
echo Running clippy...
cargo clippy --all-targets -- -D warnings
if errorlevel 1 goto :error

REM Format check
echo Checking format...
cargo fmt -- --check
if errorlevel 1 goto :error

REM Dry run
echo Dry run...
cargo publish --dry-run
if errorlevel 1 goto :error

echo.
set /p CONFIRM="Ready to publish to crates.io? (y/n): "
if /i "%CONFIRM%"=="y" (
    cargo publish
    echo.
    echo Published to crates.io!
    echo https://crates.io/crates/hope-os
) else (
    echo Cancelled.
)

goto :end

:error
echo.
echo Error occurred! Publishing cancelled.
exit /b 1

:end
