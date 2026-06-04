# Plan: SSH Guard — Prohibir comandos SSH directos

> **Contexto:** En el postmortem de la sesión 03/Jun/2026, el error #2 fue "Comandos SSH directos para deploy". A pesar de la regla `-2` del protocolo, el agente ejecutó `ssh root@66.94.100.241 "docker compose up -d ..."` directamente. Esto elude bind mounts, health checks y el flujo seguro de coolify-manager-rs.
>
> **Objetivo:** Hacer técnicamente imposible (o muy difícil) que el agente ejecute comandos SSH directos, obligándolo a usar coolify-manager-rs para toda operación de gestión.

---

## ¿Qué es SSH Guard?

Un sistema de **doble capa** que intercepta y filtra comandos SSH en el servidor:

1. **Capa 1 — Servidor (`authorized_keys`):** Usa `command=` en `~/.ssh/authorized_keys` para interceptar TODOS los comandos SSH y pasarlos por un script guardián.
2. **Capa 2 — Script guardián (`/opt/coolify-guard/ssh-guard.sh`):** Solo permite comandos que tengan un **marcador secreto** (`CM_GUARD_v1`), y **rechaza** cualquier otro comando o sesión interactiva.
3. **Capa 3 — Cliente modificado (`SshClient` en coolify-manager-rs):** Antepone el marcador a cada comando SSH que ejecuta.

El flujo completo:

```
Usuario → coolify-manager-rs → SshClient.execute("CM_GUARD_v1 docker ps")
  → ssh root@server "CM_GUARD_v1 docker ps"
    → authorized_keys: command="/opt/coolify-guard/ssh-guard.sh CM_GUARD_v1 docker ps"
      → ssh-guard.sh: detecta marcador, lo quita, ejecuta "docker ps"
```

```
Agente (por error) → ssh root@server "docker compose up -d"
  → authorized_keys: command="/opt/coolify-guard/ssh-guard.sh docker compose up -d"
    → ssh-guard.sh: NO hay marcador → RECHAZADO con mensaje claro
```

---

## Implementación

### Fase 1: Script guardián (`scripts/ssh-guard.sh`)

Archivo que se instala en `/opt/coolify-guard/ssh-guard.sh` en el servidor.

**Comportamiento:**
- Si el comando empieza con `CM_GUARD_v1 ` → lo quita y ejecuta el resto normalmente
- Si es sesión interactiva (sin comando) → muestra mensaje de bloqueo y sale con código 1
- Si es cualquier otro comando → muestra mensaje de bloqueo y sale con código 1
- Si existe `/tmp/ssh-emergency` → bypass total (permite cualquier comando, para emergencias reales)
- Logging a `/var/log/ssh-guard.log`

**Contenido ya creado en `scripts/ssh-guard.sh`.**

### Fase 2: Modificar `authorized_keys` en el servidor

Añadir `command="/opt/coolify-guard/ssh-guard.sh %s"` al principio de la línea de la clave pública en `~/.ssh/authorized_keys`.

La línea actual (típicamente) es:
```
ssh-ed25519 AAAAC3... root@server
```

Se modifica a:
```
command="/opt/coolify-guard/ssh-guard.sh %s" ssh-ed25519 AAAAC3... root@server
```

Esto hace que OpenSSH pase **cualquier** comando SSH a través del script guardián. `%s` se reemplaza por el comando original.

⚠️ **Riesgo:** Si hacemos esto directamente por SSH y el guardián tiene un bug, nos quedamos fuera. Por eso:
1. Primero instalar el script
2. Probar el marcador vía coolify-manager-rs
3. **Luego** modificar authorized_keys
4. Tener plan de contingencia (Coolify Terminal → `rm /tmp/ssh-emergency` para desbloquear)

### Fase 3: Modificar `SshClient` en coolify-manager-rs

En `src/infra/ssh_client.rs` (o donde esté `SshClient`):

```rust
// Constante con el marcador
const CM_GUARD_MARKER: &str = "CM_GUARD_v1";

impl SshClient {
    pub async fn execute(&self, cmd: &str) -> Result<SshResult> {
        // Anteponer marcador a TODO comando
        let guarded_cmd = format!("{} {}", CM_GUARD_MARKER, cmd);
        // ... ejecutar guarded_cmd en vez de cmd ...
    }
}
```

Esto asegura que **toda** operación de coolify-manager-rs tenga el marcador, sin cambiar ninguna llamada.

### Fase 4: Probar en un servicio no-crítico

1. Instalar script en servidor (via coolify-manager-rs `exec`)
2. Probar `$cm exec --name cap -- "echo test"` → debe funcionar (el SshClient pone el marcador)
3. Probar SSH directo `ssh root@server "echo test"` → debe ser RECHAZADO
4. Si funciona, modificar authorized_keys
5. Si falla, el emergency bypass `/tmp/ssh-emergency` permite desbloquear

---

## Contingencia (emergencia)

Si el guard bloquea incluso a coolify-manager-rs:

1. **Coolify Panel → Terminal** → entrar al contenedor o al host
2. Ejecutar: `touch /tmp/ssh-emergency`
3. Esto desactiva el guard temporalmente
4. Corregir el bug
5. `rm /tmp/ssh-emergency` para reactivar

También se puede desinstalar completamente:
```bash
# Editar authorized_keys manualmente (vía Coolify Terminal)
sed -i 's/^command="\/opt\/coolify-guard\/ssh-guard.sh %s" //' ~/.ssh/authorized_keys
rm -rf /opt/coolify-guard/
```

---

## ¿Qué comandos QUEDAN PROHIBIDOS?

SSH directo **no pasa por coolify-manager-rs**, luego es rechazado:

| Comando | Resultado |
|---------|-----------|
| `ssh root@server "docker compose up -d"` | ❌ RECHAZADO |
| `ssh root@server "docker ps"` | ❌ RECHAZADO |
| `ssh root@server -L 8080:localhost:80` | ❌ RECHAZADO (sesión interactiva) |
| `scp file root@server:/path` | ❌ RECHAZADO (usa SSH) |
| `rsync -avz file root@server:/path` | ❌ RECHAZADO (usa SSH) |

**¿Qué comandos SIGUEN FUNCIONANDO?**

| Comando | Resultado |
|---------|-----------|
| `$cm health --name cap` | ✅ Pasa por SshClient → marcador añadido |
| `$cm redeploy --name studio` | ✅ Pasa por SshClient |
| `$cm exec --name cap -- "df -h"` | ✅ Pasa por SshClient |
| `ssh root@server "cmd"` CON `/tmp/ssh-emergency` presente | ✅ Bypass de emergencia |

---

## Limitaciones

- **`scp`/`rsync`:** No son comandos SSH ejecutables, SCP usa subsistema SFTP/SCP. Para SCP necesitamos `command=` en authorized_keys que invoque `scp` wrapper. Por ahora, si se necesita transferir archivos, usar coolify-manager-rs o Coolify Terminal.
- **Subsistema SFTP:** También interceptado por `command=`. Si se necesita SFTP, hay que modificar el guard para permitir `$SFTP_SUBSYSTEM`.
- **Coolify no se ve afectado:** Coolify usa su propia conexión interna, no pasa por `authorized_keys`.

---

## Próximos pasos

1. ✅ script creado en `scripts/ssh-guard.sh`
2. ❌ Instalar script en servidor (coolify-manager-rs exec)
3. ❌ Modificar SshClient para añadir marcador automático
4. ❌ Probar con servicio no-crítico
5. ❌ Modificar authorized_keys
6. ❌ Probar SSH directo → debe fallar
