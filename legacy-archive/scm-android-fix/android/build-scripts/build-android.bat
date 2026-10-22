@echo off
setlocal
set "JAVA_HOME=E:\Android\android-studio\jbr"
set "PATH=E:\Android\android-studio\jbr\bin;E:\Sdk\platform-tools;%PATH%"
set "ANDROID_HOME=E:\Sdk"
set "TEMP=E:\build-tools\temp"
set "CARGO_HOME=E:\build-tools\.cargo"
set "GRADLE_USER_HOME=E:\build-tools\.gradle"
cd /d E:\SCMessenger-Github-Repo\SCMessenger\android
echo === Java version ===
"%JAVA_HOME%\bin\java.exe" -version
echo === Gradle wrapper version ===
call gradlew.bat --version
echo === Starting build ===
call gradlew.bat :app:assembleDebug -x lint --no-daemon
echo === EXIT CODE: %errorlevel% ===
