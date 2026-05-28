@echo off
cd /d "%~dp0"

set MVN=%USERPROFILE%\.vscode\extensions\oracle.oracle-java-25.1.0\nbcode\java\maven\bin\mvn.cmd
if not exist "%MVN%" set MVN=mvn

echo === Compiling ===
call "%MVN%" compile test-compile -q
if %ERRORLEVEL% NEQ 0 ( echo BUILD FAILED & pause & exit /b 1 )

echo === Launching GUI ===
start "JJazz Pure Core" "%MVN%" exec:java -q
