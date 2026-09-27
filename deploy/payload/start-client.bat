@echo off
setlocal
if not exist "%~dp0synesthesia-client.exe" (
    echo Missing synesthesia-client.exe. Build and package the client application first. 1>&2
    exit /b 2
)
pushd "%~dp0"
if errorlevel 1 exit /b 1
start "" /wait "%~dp0synesthesia-client.exe" %*
set "LAUNCH_EXIT=%ERRORLEVEL%"
popd
exit /b %LAUNCH_EXIT%
