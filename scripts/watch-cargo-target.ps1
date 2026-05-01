param(
    [string[]]$TargetDirs = @('C:\tmp\glory-target'),
    [int]$MaxTotalMB = 4096,
    [int]$IntervalSeconds = 120
)

$ErrorActionPreference = 'Stop'
$cleanScript = Join-Path $PSScriptRoot 'clean-cargo-target.ps1'

while ($true) {
    & powershell -ExecutionPolicy Bypass -File $cleanScript -TargetDirs $TargetDirs -MaxTotalMB $MaxTotalMB
    Start-Sleep -Seconds $IntervalSeconds
}