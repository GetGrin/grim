@echo off
setlocal enabledelayedexpansion

:: Change directory to the script's location
cd /d "%~dp0"

:: Skip if Go not found.
where go >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Go could not be found
    exit /b 0
)

set "go_os=%~1"
set "go_arch=%~2"
set "output_path=%~3"

echo Go build for os: %go_os%, arch: %go_arch%

:: Setup vars for Android.
if "%go_os%"=="android" (

    :: Setup NDK root path env.
    if "%ANDROID_NDK_HOME%"=="" (
        :: Extract ndkVersion from build.gradle
        :: Equivalent to: cat ../android/app/build.gradle | grep 'ndkVersion' | cut -d ' -f 2
        for /f "tokens=2 delims='" %%a in ('findstr "ndkVersion" ..\android\app\build.gradle') do (
            set "NDK_VERSION=%%a"
        )
        set "ANDROID_NDK_HOME=%ANDROID_HOME%\ndk\!NDK_VERSION!"
    )

    :: Setup NDK host path.
    :: Since this is a Batch script, the host is Windows.
    set "arch_host=windows-x86_64"

    :: Setup NDK target arch.
    if "%go_arch%"=="arm64" (
        set "arch_bin_prefix=aarch64-linux-android"
    ) else if "%go_arch%"=="arm" (
        set "arch_bin_prefix=armv7a-linux-androideabi"
    ) else (
        set "arch_bin_prefix=x86_64-linux-android"
    )

    :: Build for current target.
    set "CGO_ENABLED=1"
    set "GOOS=%go_os%"
    set "GOARCH=%go_arch%"

    :: Define CC and CXX paths
    set "CC=!ANDROID_NDK_HOME!\toolchains\llvm\prebuilt\!arch_host!\bin\!arch_bin_prefix!35-clang"
    set "CXX=!ANDROID_NDK_HOME!\toolchains\llvm\prebuilt\!arch_host!\bin\!arch_bin_prefix!35-clang++"

    go build -C "../tor/webtunnel" -ldflags="-s -w" -o "%output_path%" code.gri.mw/WEB/webtunnel/main/client

) else (
    set "extra_flag="
    if "%go_os%"=="windows" (
        set "extra_flag=-H=windowsgui"
    )

    set "GOOS=%go_os%"
    set "GOARCH=%go_arch%"

    :: Build for non-android targets
    go build -C "../tor/webtunnel" -ldflags="-s -w !extra_flag!" -o "%output_path%" code.gri.mw/WEB/webtunnel/main/client
)

endlocal
