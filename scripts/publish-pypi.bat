@echo off
REM =============================================================================
REM Publish Hope OS to PyPI
REM =============================================================================

echo.
echo  Hope OS - Publishing to PyPI
echo  ()=^>[]
echo.

REM Check maturin
where maturin >nul 2>&1
if errorlevel 1 (
    echo Installing maturin...
    pip install maturin
)

REM Build wheels
echo Building wheels...
maturin build --release --features python
if errorlevel 1 goto :error

REM Install locally for testing
echo Installing locally...
pip install -e . --features python
if errorlevel 1 goto :error

REM Run tests
echo Running Python tests...
pytest tests/
if errorlevel 1 goto :error

echo.
set /p CONFIRM="Ready to publish to PyPI? (y/n): "
if /i "%CONFIRM%"=="y" (
    maturin publish --features python
    echo.
    echo Published to PyPI!
    echo https://pypi.org/project/hope-os/
) else (
    echo Cancelled.
)

goto :end

:error
echo.
echo Error occurred! Publishing cancelled.
exit /b 1

:end
