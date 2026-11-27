# mangrove launcher for 8 GB NVIDIA cards (tested on an RTX 3070 8 GB, Ryzen 5 2600, 32 GB): Series S profile +
# DLSS Quality (much less card memory than Series X at 1440p, which ran out of card memory here after 80 s),
# 30 fps cap, the 8-minute pool stop disarmed (VICTORY_POOL_NO_ASSERT), and the game at below-normal priority on
# all CPU threads but the first two, so the desktop stays smooth. The window goes to the second monitor.
#   pc\play.ps1 -Exe 'D:\MyDump\game_scarlett_master.exe'            play
#   pc\play.ps1 -Exe ... -Preload                                    build every shader first (idle priority), then stop
# F1 in the game = settings menu (what it saves overrides these), F3 = stats.
param([Parameter(Mandatory)][string]$Exe, [switch]$Preload)
$repo = Split-Path $PSScriptRoot -Parent
$me = Get-Process -Id $PID
$all = [Environment]::ProcessorCount
$mask = [long]([math]::Pow(2, $all) - 1)
if ($Preload) { $me.PriorityClass = 'Idle'; $me.ProcessorAffinity = [IntPtr]($mask -band -bnot ([long]([math]::Pow(2, [int]($all / 2)) - 1))) }
else { $me.PriorityClass = 'BelowNormal'; if ($all -gt 4) { $me.ProcessorAffinity = [IntPtr]($mask -band -bnot 3L) } }
$opts = @{ Exe = $Exe; SeriesS = $true; Dlss = 'quality'; Fps = 30; More = @{ VICTORY_POOL_NO_ASSERT = '1' } }
if ($Preload) {
    # play-foxhound.ps1 preloads with these same switches and then starts the game: stop it there.
    $logs = Join-Path $env:LOCALAPPDATA 'victory\logs'
    $job = Start-Job { while ($true) { Start-Sleep 5
        $l = Get-ChildItem "$using:logs\victory-nvidia-*.log" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime | Select-Object -Last 1
        if ($l -and (Select-String -LiteralPath $l.FullName -Pattern 'Starting the game' -Quiet)) { Stop-Process -Name victory -Force; break } } }
    & "$repo\scripts\play-foxhound.ps1" @opts -Preload
    Stop-Job $job -ErrorAction SilentlyContinue
} else {
    & "$repo\scripts\play-foxhound.ps1" @opts
}
