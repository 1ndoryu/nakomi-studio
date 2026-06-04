<#
.SYNOPSIS
    Ejecuta un script bash en el servidor remoto via SSH, sin problemas de escaping.
    
.DESCRIPTION
    Codifica el script en base64, lo transmite via SSH, y lo ejecuta en el servidor.
    Evita TODOS los problemas de: CRLF, comillas de PowerShell, expansiones de fecha,
    caracteres especiales, heredocs rotos.

.PARAMETER ScriptPath
    Ruta al archivo .sh local que se ejecutará en el servidor.

.PARAMETER Host
    Host SSH destino. Default: root@66.94.100.241

.PARAMETER DryRun
    Solo muestra el comando SSH que se ejecutaría, sin ejecutarlo.

.EXAMPLE
    .\ssh-exec.ps1 -ScriptPath .\mi-script.sh
    .\ssh-exec.ps1 -ScriptPath .\fix-db.sh -Host root@173.249.50.44
    .\ssh-exec.ps1 -ScriptPath .\deploy.sh -DryRun
#>
param(
    [Parameter(Mandatory=$true)]
    [string]$ScriptPath,
    
    [string]$Host = "root@66.94.100.241",
    
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Validar que el archivo existe
if (-not (Test-Path $ScriptPath)) {
    Write-Error "Archivo no encontrado: $ScriptPath"
    exit 1
}

# Leer bytes crudos (preserva LF exacto, sin conversion CRLF)
$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $ScriptPath))

# Codificar en base64
$encoded = [Convert]::ToBase64String($bytes)

# Nombre temporal en el servidor
$tmpName = "ssh-exec-$(Get-Date -Format 'yyyyMMddHHmmss').sh"

# Construir comando remoto:
# 1. Decodificar base64 → archivo temporal
# 2. Forzar LF (por si acaso)
# 3. Verificar sintaxis bash
# 4. Ejecutar
# 5. Capturar exit code
# 6. Limpiar
$remoteCmd = @"
echo '$encoded' | base64 -d > /tmp/$tmpName && sed -i 's/\r$//' /tmp/$tmpName && bash -n /tmp/$tmpName && bash /tmp/$tmpName; EXITCODE=`$?; rm -f /tmp/$tmpName; exit `$EXITCODE
"@

if ($DryRun) {
    Write-Host "=== DRY RUN ===" -ForegroundColor Yellow
    Write-Host "Host: $Host"
    Write-Host "Script: $ScriptPath ($($bytes.Length) bytes)"
    Write-Host "Base64 length: $($encoded.Length) chars"
    Write-Host "Remote tmp: /tmp/$tmpName"
    Write-Host ""
    Write-Host "Comando SSH:" -ForegroundColor Cyan
    Write-Host "ssh $Host `"$remoteCmd`""
    exit 0
}

Write-Host "Subiendo $ScriptPath ($($bytes.Length) bytes) a $Host..." -ForegroundColor Cyan

# Ejecutar via SSH
ssh $Host $remoteCmd
$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    Write-Host "Script ejecutado OK (exit 0)" -ForegroundColor Green
} else {
    Write-Host "Script falló (exit $exitCode)" -ForegroundColor Red
    exit $exitCode
}
