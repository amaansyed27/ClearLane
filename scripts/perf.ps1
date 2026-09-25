param(
    [int[]]$TabCounts = @(1, 10, 30),
    [int]$SettleSeconds = 8
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\bundle\clearlane.exe"
if (-not (Test-Path $exe)) { throw "Run .\scripts\setup.ps1 first." }

$resultDir = Join-Path $root "perf-results"
New-Item -ItemType Directory -Force -Path $resultDir | Out-Null
$rows = @()

function Get-ProcessTree([int]$RootPid) {
    $all = Get-CimInstance Win32_Process
    $ids = New-Object System.Collections.Generic.HashSet[int]
    [void]$ids.Add($RootPid)
    $changed = $true
    while ($changed) {
        $changed = $false
        foreach ($p in $all) {
            if ($ids.Contains([int]$p.ParentProcessId) -and -not $ids.Contains([int]$p.ProcessId)) {
                [void]$ids.Add([int]$p.ProcessId); $changed = $true
            }
        }
    }
    @(Get-Process -Id $ids.ToArray() -ErrorAction SilentlyContinue)
}

function Measure-Run([string]$Label, [int]$Tabs) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $exe -ArgumentList "--perf-tabs", "$Tabs" -PassThru
    $startupMs = $null
    for ($i = 0; $i -lt 240; $i++) {
        Start-Sleep -Milliseconds 50
        $p.Refresh()
        if ($p.MainWindowHandle -ne 0) { $startupMs = $sw.ElapsedMilliseconds; break }
        if ($p.HasExited) { throw "ClearLane exited during startup." }
    }
    if ($null -eq $startupMs) { throw "ClearLane did not expose a usable window within 12 seconds." }
    Start-Sleep -Seconds $SettleSeconds
    $tree1 = Get-ProcessTree $p.Id
    $cpu1 = ($tree1 | Measure-Object CPU -Sum).Sum
    Start-Sleep -Seconds 2
    $tree2 = Get-ProcessTree $p.Id
    $cpu2 = ($tree2 | Measure-Object CPU -Sum).Sum
    $memoryMb = [math]::Round((($tree2 | Measure-Object WorkingSet64 -Sum).Sum / 1MB), 1)
    $idleCpuPct = [math]::Round((($cpu2 - $cpu1) / 2.0 / [Environment]::ProcessorCount) * 100.0, 2)
    $row = [pscustomobject]@{ label=$Label; tabs=$Tabs; startup_ms=$startupMs; working_set_mb=$memoryMb; idle_cpu_pct=$idleCpuPct }
    $script:rows += $row
    $p.CloseMainWindow() | Out-Null
    if (-not $p.WaitForExit(5000)) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Milliseconds 750
}

Measure-Run "cold-ish" 1
Measure-Run "warm" 1
foreach ($tabs in $TabCounts | Where-Object { $_ -ne 1 }) { Measure-Run "tabs-$tabs" $tabs }

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$out = Join-Path $resultDir "baseline-$stamp.json"
$rows | ConvertTo-Json | Set-Content -Encoding UTF8 $out
$rows | Format-Table -AutoSize
Write-Host "Saved: $out"
Write-Host "Note: 'cold-ish' is a fresh process, not a guaranteed OS disk-cache purge. Compare browsers on the same machine and procedure."
