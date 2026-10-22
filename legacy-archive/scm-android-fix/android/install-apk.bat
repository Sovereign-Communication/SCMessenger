@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo SCMessenger Direct APK Installer for Windows (Josh)
echo ===================================================
echo.

:: 1. Check ADB
where adb >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] ADB not found in PATH.
    echo Please make sure adb.exe is in the same directory or in your system PATH.
    echo.
    pause
    exit /b 1
)

:: 2. Check for connected Android device
echo Checking for connected Android phone...
for /f "tokens=1,2 delims=	 " %%a in ('adb devices ^| findstr /v "List"') do (
    if "%%b"=="device" (
        set DEVICE_ID=%%a
    )
)

if "!DEVICE_ID!"=="" (
    echo [ERROR] No connected Android device detected!
    echo Please connect your phone via USB, unlock the screen, and allow USB Debugging.
    echo.
    pause
    exit /b 1
)

echo Found device: !DEVICE_ID!
echo.

:: 3. Find APK
set APK_PATH=app-debug.apk
if not exist "%APK_PATH%" (
    set APK_PATH=app\build\outputs\apk\debug\app-debug.apk
)

if not exist "%APK_PATH%" (
    echo [ERROR] Could not find app-debug.apk!
    echo Place app-debug.apk in this directory and try again.
    echo.
    pause
    exit /b 1
)

echo Installing %APK_PATH% onto your phone...
adb -s !DEVICE_ID! install -r "%APK_PATH%"

if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Installation failed.
    pause
    exit /b 1
)

echo.
echo [OK] Installation successful! Granting initial permissions...
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.ACCESS_FINE_LOCATION 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.ACCESS_COARSE_LOCATION 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.BLUETOOTH_SCAN 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.BLUETOOTH_ADVERTISE 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.BLUETOOTH_CONNECT 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.NEARBY_WIFI_DEVICES 2>nul
adb -s !DEVICE_ID! shell pm grant com.scmessenger.android android.permission.POST_NOTIFICATIONS 2>nul

echo.
echo Launching SCMessenger on your phone...
adb -s !DEVICE_ID! shell am start -n "com.scmessenger.android/.ui.MainActivity"

echo.
echo [DONE] SCMessenger is now ready on your phone!
pause
