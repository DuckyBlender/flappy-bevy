@REM BUILD SCRIPT FOR RUST BEVY PROGRAM
@REM THE POINT OF THIS SCRIPT IS TO MAKE IT EASY TO COMPILE AND COPY THE ASSETS FOLDER AND ZIP IT AUTOMATICALLY
@REM THIS SCRIPT IS NOT REQUIRED TO BUILD THE PROGRAM, IT IS JUST A CONVENIENCE SCRIPT

@echo off

setlocal

set PROJECT_NAME=flappy-bevy
set BUILD_DIR=build
set ASSETS_DIR=assets
set ZIP_FILE=%BUILD_DIR%\%PROJECT_NAME%.zip

if not exist %BUILD_DIR% mkdir %BUILD_DIR%

cargo build --release

xcopy /E /I /Y %ASSETS_DIR% %BUILD_DIR%\%ASSETS_DIR%

copy /Y target\release\%PROJECT_NAME%.exe %BUILD_DIR%

@REM Compress the build directory into a zip file in the main directory
powershell Compress-Archive -Path %BUILD_DIR%\* -DestinationPath %PROJECT_NAME%.zip
popd

rmdir /S /Q %BUILD_DIR%

endlocal
