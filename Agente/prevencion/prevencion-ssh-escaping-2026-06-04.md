# Prevención: SSH Escaping desde PowerShell

## Problema
PowerShell + SSH + bash = CRLF, comillas rotas, expansiones interceptadas (`$(date)` → `Get-Date`), heredocs fallidos. Causa lockouts del servidor.

## Regla
**NUNCA** ejecutar comandos bash multilinea o con comillas directamente via `ssh root@server "..."` desde PowerShell.

**SIEMPRE** usar uno de estos patrones:

### Patrón 1: Script local + `ssh-exec.ps1` (preferido)
```powershell
# Crear script local, luego ejecutarlo remotamente
.\scripts\ssh-exec.ps1 -ScriptPath .\mi-script.sh
```

### Patrón 2: Base64 inline (para comandos cortos)
```powershell
$script = "echo hello"  # Sin comillas bash, sin CRLF
$bytes = [System.Text.Encoding]::UTF8.GetBytes($script)
$encoded = [Convert]::ToBase64String($bytes)
ssh root@66.94.100.241 "echo $encoded | base64 -d | bash"
```

### Patrón 3: Archivo temporal en servidor (para scripts complejos)
```powershell
$bytes = [System.IO.File]::ReadAllBytes(".\script.sh")
$encoded = [Convert]::ToBase64String($bytes)
ssh root@server "echo $encoded | base64 -d > /tmp/run.sh && bash /tmp/run.sh && rm /tmp/run.sh"
```

## Checklist antes de cualquier SSH con script
- [ ] ¿El script usa comillas bash? → Usar base64
- [ ] ¿El script tiene variables `$(...)`? → Usar base64 (PowerShell intercepta)
- [ ] ¿El script modifica `authorized_keys`? → Backup primero con nombre fijo (sin `$(date)`)
- [ ] ¿El script tiene más de una línea? → Usar base64 o `ssh-exec.ps1`
- [ ] ¿Verifiqué `bash -n script.sh` antes de activarlo con `command=`?

## Herramienta
`scripts/ssh-exec.ps1` — script PowerShell reutilizable que maneja todo el flujo (base64, CRLF fix, syntax check, ejecución, cleanup).
