@echo off
setlocal
if not exist "%~dp0synesthesia-full.exe" (
    echo Missing synesthesia-full.exe. Build and package the full application first. 1>&2
    exit /b 2
)
pushd "%~dp0"
if errorlevel 1 exit /b 1
start "" /wait "%~dp0synesthesia-full.exe" %*
set "LAUNCH_EXIT=%ERRORLEVEL%"
popd
exit /b %LAUNCH_EXIT%
