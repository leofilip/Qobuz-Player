@echo off
setlocal enabledelayedexpansion

:menu
cls
echo ===============================
echo Qobuz Player Build Menu
echo ===============================
echo.
echo 1 - Check for missing dependencies
echo 2 - Run the app in development mode (cargo tauri dev)
echo 3 - Build the app in release mode (cargo tauri build)
echo 4 - Open installer folder (src-tauri/target/release/bundle/msi)
echo 5 - Set version (update Cargo.toml and tauri.conf.json)
echo 0 - Exit
echo.
set /p choice="Select an option: "
if "%choice%"=="1" goto checkdeps
if "%choice%"=="2" goto devmode
if "%choice%"=="3" goto buildrelease
if "%choice%"=="4" goto openinstaller
if "%choice%"=="5" goto setversion
if "%choice%"=="0" exit /b

echo Invalid option.
pause
goto menu
:setversion
set /p newver="Enter new version (e.g. 0.1.2): "
@echo off
setlocal enabledelayedexpansion

:menu
cls
echo ===============================
echo Qobuz Player Build Menu
echo ===============================
echo.
echo 1 - Check for missing dependencies
echo 2 - Run the app in development mode (cargo tauri dev)
echo 3 - Build the app in release mode (cargo tauri build)
echo 4 - Open installer folder (src-tauri/target/release/bundle/msi)
echo 5 - Set version (update Cargo.toml and tauri.conf.json)
echo 0 - Exit
echo.
set /p choice="Select an option: "
if "%choice%"=="1" goto checkdeps
if "%choice%"=="2" goto devmode
if "%choice%"=="3" goto buildrelease
if "%choice%"=="4" goto openinstaller
if "%choice%"=="5" goto setversion
if "%choice%"=="0" exit /b

goto :eof

:check_wix
:setversion

set /p newver="Enter new version (e.g. 0.1.2): "
REM Validate format: must be #.#.#
set "verrormsg="
echo %newver% | findstr /r "^[0-9]\+\.[0-9]\+\.[0-9]\+$" >nul
if not %errorlevel%==0 (
    set "verrormsg=Invalid format! Version must be in the form #.#.# (e.g. 1.2.3)"
    goto versionerror
)

REM Get current version from Cargo.toml
for /f "tokens=3 delims= " %%v in ('findstr /b "version = " src-tauri\Cargo.toml') do set currver=%%v
set currver=%currver:~1,-1%

REM Compare versions
for /f "tokens=1-3 delims=." %%a in ("%currver%") do (
    set currmaj=%%a
    set currmin=%%b
    set currpat=%%c
)
for /f "tokens=1-3 delims=." %%a in ("%newver%") do (
    set newmaj=%%a
    set newmin=%%b
    set newpat=%%c
)

set /a cmpmaj=newmaj-currmaj
set /a cmpmin=newmin-currmin
set /a cmppat=newpat-currpat

if %cmpmaj% lss 0 (
    set "verrormsg=New version must be higher than current version (%currver%)!"
    goto versionerror
)
if %cmpmaj%==0 if %cmpmin% lss 0 (
    set "verrormsg=New version must be higher than current version (%currver%)!"
    goto versionerror
)
if %cmpmaj%==0 if %cmpmin%==0 if %cmppat% lss 1 (
    set "verrormsg=New version must be higher than current version (%currver%)!"
    goto versionerror
)

REM Update Cargo.toml
set "found=0"
break > src-tauri\Cargo.tmp
for /f "delims=" %%l in ('type src-tauri\Cargo.toml') do (
    echo %%l | findstr /b "version = " >nul
    if !errorlevel! == 0 (
        echo version = "%newver%" >> src-tauri\Cargo.tmp
        set "found=1"
    ) else (
        echo %%l >> src-tauri\Cargo.tmp
    )
)
if !found! == 0 (
    echo version = "%newver%" >> src-tauri\Cargo.tmp
)
move /Y src-tauri\Cargo.tmp src-tauri\Cargo.toml >nul

REM Update tauri.conf.json
set "foundjson=0"
break > src-tauri\tauri.tmp
for /f "delims=" %%l in ('type src-tauri\tauri.conf.json') do (
    echo %%l | findstr "\"version\"" >nul
    if !errorlevel! == 0 (
        echo     "version": "%newver%", >> src-tauri\tauri.tmp
        set "foundjson=1"
    ) else (
        echo %%l >> src-tauri\tauri.tmp
    )
)
if !foundjson! == 0 (
    echo     "version": "%newver%", >> src-tauri\tauri.tmp
)
move /Y src-tauri\tauri.tmp src-tauri\tauri.conf.json >nul
echo Version updated to %newver% in Cargo.toml and tauri.conf.json
pause
goto menu
where candle >nul 2>nul
if %errorlevel%==0 (
    echo [OK] WiX Toolset found: candle.exe
) else (
    echo [MISSING] WiX Toolset NOT found: candle.exe
    echo Download WiX Toolset: https://github.com/wixtoolset/wix/releases/
)
goto :eof

:devmode
echo Running: cargo tauri dev
cargo tauri dev
pause
goto menu

:buildrelease
echo Building: cargo tauri build
cargo tauri build
goto openinstaller

:openinstaller
:openinstaller
if exist src-tauri\target\release\bundle\msi (
    start "" src-tauri\target\release\bundle\msi
    echo Opened installer folder.
) else (
    echo Installer folder not found.
)
pause
goto menu

:versionerror
echo %verrormsg%
echo Press any key to return to the menu...
pause
goto menu