@echo off
rem mangrove launcher: runs the game with the NVIDIA 8 GB settings and opens the easy view
rem (a plain-English status window) instead of the raw log. The game window goes to the
rem second monitor. Full raw log: %TEMP%\mangrove-game.log
rem
rem Set GAME to your own copy of the game's executable (no game files come with this repo).
if not defined GAME set GAME=C:\path\to\your\game\game_scarlett_master.exe
if not exist "%GAME%" (
  echo Edit pc\play.bat and set GAME to your own game executable first.
  pause
  exit /b 1
)
title mangrove - easy view
set VICTORY_LOG=info,victory_gpu::descriptor::host=error
rem NVIDIA: no repeated frames (fork #269), no black mosaic blocks (#310), card buffers sized for 8 GB
set VICTORY_TEXTURE_GUARD=0
set VICTORY_SHADER_OPT=0
set VICTORY_CARD_BUFFERS_MB=512
rem the title stops on an int3 when a fixed pool is full (~6-8 min into the opening cinematic, VIctory #698):
rem make those allocations fail with null instead (see README, "Getting past the 8-minute crash")
set VICTORY_POOL_NO_ASSERT=1
if not defined VICTORY_CACHE_DIR set VICTORY_CACHE_DIR=%LOCALAPPDATA%\mangrove\cache
if not defined VICTORY_DATA_DIR set VICTORY_DATA_DIR=%LOCALAPPDATA%\mangrove\data
set GLOG=%TEMP%\mangrove-game.log
del "%GLOG%" 2>nul
cd /d "%~dp0.."
start "" /b cmd /c "target\release\victory.exe run "%GAME%" --execute > "%GLOG%" 2>&1"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0easy-view.ps1" -Log "%GLOG%"
