@echo off
rem Settings shared by play.bat and warm.bat (the shader cache is keyed by them: keep them the same).
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
