param(
    [string[]]$TargetDirs = @('C:\tmp\glory-target'),
    [int]$MaxTotalMB = 4096,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'

function Get-DirectorySizeMB {
    param([string]$Path)

    if (-not (Test-Path $Path)) {
        return 0
    }

    $totalBytes = 0
    Get-ChildItem -LiteralPath $Path -Recurse -Force -File -ErrorAction SilentlyContinue | ForEach-Object {
        $totalBytes += $_.Length
    }
    return [math]::Round($totalBytes / 1MB, 2)
}

function Remove-MatchingDirectories {
    param(
        [string]$Path,
        [string[]]$Names
    )

    foreach ($name in $Names) {
        Get-ChildItem -LiteralPath $Path -Recurse -Force -Directory -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -eq $name } |
            Sort-Object FullName -Descending |
            ForEach-Object {
                Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
            }
    }
}

function Test-RustcActive {
    return [bool](Get-Process -Name 'rustc' -ErrorAction SilentlyContinue)
}

foreach ($targetDir in $TargetDirs) {
    if (-not (Test-Path $targetDir)) {
        continue
    }

    $initialSize = Get-DirectorySizeMB -Path $targetDir
    if ($initialSize -le $MaxTotalMB) {
        Write-Host "[cargo-clean] $targetDir OK ($initialSize MB / $MaxTotalMB MB)"
        continue
    }

    if ((Test-RustcActive) -and (-not $Force)) {
        Write-Host "[cargo-clean] rustc activo; se pospone limpieza de $targetDir ($initialSize MB)"
        continue
    }

    Write-Host "[cargo-clean] limpiando $targetDir ($initialSize MB / $MaxTotalMB MB)"
    Remove-MatchingDirectories -Path $targetDir -Names @('incremental')

    $afterIncremental = Get-DirectorySizeMB -Path $targetDir
    if ($afterIncremental -le $MaxTotalMB) {
        Write-Host "[cargo-clean] $targetDir reducido a $afterIncremental MB"
        continue
    }

    Remove-MatchingDirectories -Path $targetDir -Names @('.fingerprint', 'build')

    $afterMetadata = Get-DirectorySizeMB -Path $targetDir
    if ($afterMetadata -le $MaxTotalMB) {
        Write-Host "[cargo-clean] $targetDir reducido a $afterMetadata MB"
        continue
    }

    Remove-MatchingDirectories -Path $targetDir -Names @('deps')
    $finalSize = Get-DirectorySizeMB -Path $targetDir
    Write-Host "[cargo-clean] $targetDir reducido a $finalSize MB"
}