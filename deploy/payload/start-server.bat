@echo off
setlocal
if not exist "%~dp0synesthesia-server.exe" (
    echo Missing synesthesia-server.exe. Build and package the server application first. 1>&2
    exit /b 2
)
pushd "%~dp0"
if errorlevel 1 exit /b 1
"%~dp0synesthesia-server.exe" %*
set "LAUNCH_EXIT=%ERRORLEVEL%"
popd
exit /b %LAUNCH_EXIT%
