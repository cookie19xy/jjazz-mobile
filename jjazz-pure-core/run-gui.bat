@echo off
cd /d "%~dp0"

:: Maven (VS Code built-in)
set MVN=%USERPROFILE%\.vscode\extensions\oracle.oracle-java-25.1.0\nbcode\java\maven\bin\mvn.cmd
if not exist "%MVN%" set MVN=mvn

:: Guava jars
set GUAVA=%USERPROFILE%\.m2\repository\com\google\guava\guava\33.3.1-jre\guava-33.3.1-jre.jar
set FA=%USERPROFILE%\.m2\repository\com\google\guava\failureaccess\1.0.2\failureaccess-1.0.2.jar

echo === Compiling ===
call "%MVN%" compile test-compile -q
if %ERRORLEVEL% NEQ 0 ( echo BUILD FAILED & pause & exit /b 1 )

echo === Launching GUI ===
start javaw -cp "target\classes;target\test-classes;%GUAVA%;%FA%" org.jjazz.purecore.gui.TestGui
echo Done! GUI should be open.
