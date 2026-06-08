@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR:~0,-1%"
for %%I in ("%PROJECT_DIR%") do set "PROJECT_NAME=%%~nxI"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%..\tools\package_dxc_to_moomoo_targets.ps1" -Workspace "%SCRIPT_DIR%.." -Projects "%PROJECT_NAME%" %*
exit /b %ERRORLEVEL%
