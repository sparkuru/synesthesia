@echo off
setlocal
if not exist "%~dp0scripts\install.ps1" (
    echo Installation is not implemented in this deployment scaffold. 1>&2
    echo See scripts\CONTRACTS.md. 1>&2
    exit /b 2
)
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\install.ps1" %*
exit /b %ERRORLEVEL%
