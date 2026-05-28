@echo off
cd /d "%~dp0"
setlocal enabledelayedexpansion

echo ========================================
echo   JJazz Engine - Demo Presets
echo ========================================
echo.

call cargo build --bin jjazz-demo -q 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Build failed! Run: cargo build --bin jjazz-demo
    pause
    exit /b 1
)

set PRESETS=^
"II-V-I Jazz: Dm7 G7 Cmaj7" ^
"Blues in C: C7 F7 C7 C7 F7 F7 C7 C7 G7 F7 C7 G7" ^
"Autumn Leaves: Am7 D7 Gmaj7 Cmaj7 F#m7b5 B7 Em" ^
"Pop Classic: C G Am F" ^
"Minor Turn: Am Dm E7 Am" ^
"Jazz Extended: Dm7 G7 Cmaj7 Am7 Dm7 G7 Cmaj7" ^
"Modal: Dm7 Ebm7 Dm7" ^
"Bossa: Dm7 G7 Cmaj7 Fmaj7 Bm7b5 E7 Am7"

echo Generating %PRESETS%  preset audio files...
echo.

set COUNT=0
for %%P in (%PRESETS%) do (
    set /a COUNT+=1
    set "LINE=%%~P"
    for /f "tokens=1,* delims=: " %%a in ("!LINE!") do (
        set "NAME=%%a"
        set "CHORDS=%%b"
    )
    echo [!COUNT!] !NAME!
    echo     Chords: !CHORDS!
    cargo run --bin jjazz-demo -q -- !CHORDS! 2>nul
    move /Y output.wav "output\!NAME!.wav" >nul 2>nul
    echo     -> output/!NAME!.wav
    echo.
)

echo ========================================
echo   Done! %COUNT% files in output/
echo ========================================
dir /b output\*.wav 2>nul
