@echo off
setlocal
if not exist "%~dp0app\debug-launch.ps1" (
    echo Debug launching is not implemented in this deployment scaffold. 1>&2
    echo See deploy\scripts\CONTRACTS.md in the source project. 1>&2
    exit /b 2
)
if "%~1"=="" (
    echo Usage: start-debug.bat full^|server^|client 1>&2
    exit /b 2
)
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0app\debug-launch.ps1" -Product "%~1"
exit /b %ERRORLEVEL%
