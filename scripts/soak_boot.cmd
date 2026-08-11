@echo off
setlocal enabledelayedexpansion
rem Launcher for the always-on soak, invoked by the "SCMessengerSoak" scheduled
rem task at logon. Kept as a wrapper rather than pointing the task straight at
rem python so that stdout is captured to a file -- a scheduled task discards it
rem otherwise, and the supervisor's [OK]/[WARNING] lines are the first thing you
rem want when it did not come up.
rem
rem Repo root is derived from this script's own location, so moving or cloning
rem the checkout elsewhere does not silently break the task.

set "REPO=%~dp0.."
pushd "%REPO%" || exit /b 1

set "SOAKDIR=%LOCALAPPDATA%\scmessenger\soak\runlogs"
if not exist "%SOAKDIR%" mkdir "%SOAKDIR%" 2>nul

rem Honour an explicit interpreter if the operator sets one, else take the
rem first python on PATH, else fall back to the known install.
set "PY=%SCM_PYTHON%"
if not defined PY for /f "delims=" %%P in ('where python 2^>nul') do (
    if not defined PY set "PY=%%P"
)
if not defined PY set "PY=C:\Python314\python.exe"

if not exist "%PY%" (
    echo [FAIL] no python interpreter found ^(tried SCM_PYTHON, PATH, C:\Python314^) >> "%SOAKDIR%\supervisor_boot.log"
    popd
    exit /b 1
)

echo. >> "%SOAKDIR%\supervisor_boot.log"
echo ==== soak_boot %DATE% %TIME% using %PY% ==== >> "%SOAKDIR%\supervisor_boot.log"

"%PY%" scripts\soak_supervisor.py run --with-bridge >> "%SOAKDIR%\supervisor_boot.log" 2>&1
set "RC=%ERRORLEVEL%"

echo ==== soak_boot exited rc=%RC% %DATE% %TIME% ==== >> "%SOAKDIR%\supervisor_boot.log"
popd
exit /b %RC%
