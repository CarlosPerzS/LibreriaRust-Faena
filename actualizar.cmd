@echo off
setlocal
set "RUST_PATH=C:\Users\Carlos\projects\rust\LibreriaRust-Faena"
set "ANDROID_PATH=C:\Users\Carlos\AndroidStudioProjects\Faena"
set "LIB=libLibreriaRust.so"

cd /d "%RUST_PATH%"

cargo ndk --target aarch64-linux-android --platform 28 build --release
cargo ndk --target i686-linux-android --platform 28 build --release

set "RUST_ARM64=%RUST_PATH%\target\aarch64-linux-android\release\%LIB%"
set "ANDROID_ARM64=%ANDROID_PATH%\app\src\main\jniLibs\arm64-v8a"
set "RUST_X86=%RUST_PATH%\target\i686-linux-android\release\%LIB%"
set "ANDROID_X86=%ANDROID_PATH%\app\src\main\jniLibs\x86"

copy /Y "%RUST_ARM64%" "%ANDROID_ARM64%"

copy /Y "%RUST_X86%" "%ANDROID_X86%"

:eof
endlocal
pause