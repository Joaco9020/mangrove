@echo off
rem mangrove launcher for 8 GB NVIDIA cards: the game (pc\play.ps1) in a minimised window, and the easy view
rem (a plain-English status window) here. Set GAME to your own copy of the game's executable (no game files
rem come with this repo). Run pc\preload.cmd once first, or the first play has missing 3D while shaders build.
if not defined GAME set GAME=C:\path\to\your\game\game_scarlett_master.exe
if not exist "%GAME%" (
  echo Edit pc\play.cmd and set GAME to your own game executable first.
  pause
  exit /b 1
)
title mangrove - easy view
start "mangrove game" /min powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0play.ps1" -Exe "%GAME%"
powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Sleep 5; $l = Get-ChildItem \"$env:LOCALAPPDATA\victory\logs\victory-nvidia-*.log\" | Sort-Object LastWriteTime | Select-Object -Last 1; & '%~dp0easy-view.ps1' -Log $l.FullName"
