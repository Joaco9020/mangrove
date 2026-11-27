@echo off
rem Builds every shader and pipeline ahead at idle priority on half the CPU (it can take hours on a 6-core CPU;
rem stopped, it goes on where it was next time). Run once, and again after an update or a new NVIDIA driver.
if not defined GAME set GAME=C:\path\to\your\game\game_scarlett_master.exe
title mangrove - preloading shaders
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0play.ps1" -Exe "%GAME%" -Preload
pause
