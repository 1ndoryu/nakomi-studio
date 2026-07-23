# Plan: Restauración Guillermo + Fix Errores 500

> **Fecha:** 2026-07-22
> **Estado:** Pendiente de aprobación
> **Prioridad:** Alta — producción con errores 500 y datos de cliente perdidos

---

## Contexto

La base de datos de nakomi.studio fue recreada, causando:

1. **Pérdida de datos del cliente `guillermo@nakomi.com`** — 4 hostings y 5 billing_items eliminados
2. **Errores 500** en endpoints `/api/hosting/deployments` y `/api/hosting/vps` — probablemente por migraciones no aplicadas tras la recreación de la BD
3. **Suscripción Stripe activa de `cap.wandori.us`** (`sub_1TdRDgCdHJpmDkrr69Vn4grz`) sin vincular a ningún registro en BD

---

## Diagnóstico Errores 500

Los endpoints fallan porque consultan tablas/columnas que pueden no existir tras la recreación de la BD:

| Tabla/columna faltante                         | Endpoint afectado          | Migración requerida                                  |
| ---------------------------------------------- | -------------------------- | ---------------------------------------------------- |
| `infrastructure_servers`                       | `/api/hosting/vps`         | `20260522010000_infrastructure_resource_enforcement` |
| `server_capacity`                              | `/api/hosting/vps`         | `20260522010000_infrastructure_resource_enforcement` |
| `infrastructure_resource_samples`              | `/api/hosting/deployments` | `20260522010000_infrastructure_resource_enforcement` |
| `cpu_scaling_policy` en `hosting_plan_configs` | `/api/hosting/deployments` | `20260526000000_hosting_cpu_scaling_policy`          |
| `runtime_kind` en `hosting_subscriptions`      | `/api/hosting/deployments` | `20260524083000_hosting_runtime_identity`            |

**Verificación rápida:** ejecutar query para listar tablas existentes vía `cm exec --target postgres`.

---

## Fase 1: Fix Errores 500

### Acción

Reiniciar el servicio nakomi.studio:

```powershell
$cm = "C:\Users\Owner\OneDrive\Documentos\WP\app\public\wp-content\themes\glorytemplate\.agent\coolify-manager-rs\target\release\coolify-manager.exe"
& $cm restart --name studio
```

Esto debería re-ejecutar las migraciones de SQLx al arrancar y crear las tablas/columnas faltantes.

### Plan B (si restart no resuelve)

Si las migraciones no se auto-ejecutan, se necesitará:

1. Verificar qué tablas/columnas faltan con queries de diagnóstico
2. Ejecutar las migraciones pendientes manualmente vía `cm exec --target postgres`
3. Documentar el hallazgo como mejora al proceso de deploy

---

## Fase 2: Restaurar Datos de Guillermo

### Approach: API + Frontend (sin SSH directo)

#### Backend — Nuevo endpoint `POST /api/admin/restore-client-data`

**Archivo:** `src/handlers/admin_client_bootstrap.rs` (extender el módulo existente)

**Body:**

```json
{
    "temporary_password": "Guillermo2026!",
    "cap_wandori_stripe_subscription_id": "sub_1TdRDgCdHJpmDkrr69Vn4grz"
}
```

**Lógica:**

1. Reutiliza la lógica existente del bootstrap (upsert usuario + 4 hostings + 5 billing_items)
2. Si se proporciona `cap_wandori_stripe_subscription_id`:
    - Ejecuta `UPDATE hosting_subscriptions SET stripe_subscription_id = $1, updated_at = NOW() WHERE domain = 'cap.wandori.us'`
    - Marca el billing_item `d1000001-0001-4000-8000-000000000002` como `paid`
3. Retorna resumen de lo creado/actualizado

**Es idempotente:** ON CONFLICT en todos los INSERTs, re-ejecutable sin duplicar.

#### Backend — Endpoint de status `GET /api/admin/restore-client-data/status`

Verifica si `guillermo@nakomi.com` tiene hostings registrados. Retorna:

```json
{
    "needs_restore": false,
    "user_exists": true,
    "hostings_count": 4,
    "billing_items_count": 5
}
```

#### Frontend — Banner de restauración one-time

En la vista admin de hosting (o sección admin dedicada):

1. **Check de estado:** llama a `GET /api/admin/restore-client-data/status`
2. **Renderizado condicional:** si `needs_restore: true`, muestra un banner con:
    - Mensaje: "Datos del cliente Guillermo no encontrados. Se restaurarán los 4 hostings y 5 cobros pendientes."
    - Información del Stripe subscription a vincular
    - Botón: "Restaurar datos de Guillermo"
    - Confirmación: modal con resumen de lo que se va a crear
3. **Después de ejecutar exitosamente:** el banner desaparece automáticamente
4. **Persistencia:** el endpoint de status verifica datos reales en BD (no flag artificial)

---

## Datos a Restaurar

### Usuario

| Campo        | Valor                  |
| ------------ | ---------------------- |
| email        | `guillermo@nakomi.com` |
| role         | `client`               |
| display_name | `Guillermo`            |

### 4 Hosting Subscriptions

| Domain                   | Coolify Site | Server UUID                | Stripe Sub ID                  | Status   |
| ------------------------ | ------------ | -------------------------- | ------------------------------ | -------- |
| `materialdepadel.es`     | `padel`      | `zkcc040cc0scock4kcooowkc` | `sub_legacy_paid_padel`        | `active` |
| `guillechatbots.es`      | `guillermo`  | `owck8sww4ogk8gskgwcsk4w0` | `null`                         | `active` |
| `cap.wandori.us`         | `cap`        | `qgskgw8wwc08o444o08wko8o` | `sub_1TdRDgCdHJpmDkrr69Vn4grz` | `active` |
| `restaurante.wandori.us` | `glory-rest` | `b8s0cks444o0sogo8kg8wcgw` | `sub_legacy_paid_glory_rest`   | `active` |

### 5 Billing Items

| ID                                     | Recurso                              | Monto  | Período | Status    |
| -------------------------------------- | ------------------------------------ | ------ | ------- | --------- |
| `d1000001-0001-4000-8000-000000000001` | Hosting guillechatbots.es            | $2.48  | mensual | `paid`    |
| `d1000001-0001-4000-8000-000000000002` | Hosting cap.wandori.us               | $2.48  | mensual | `paid`    |
| `d1000001-0001-4000-8000-000000000003` | Dominio materialdepadel.es           | $15.00 | anual   | `pending` |
| `d1000001-0001-4000-8000-000000000004` | Dominio guillechatbots.es            | $15.00 | anual   | `pending` |
| `d1000001-0001-4000-8000-000000000005` | Hosting restaurante.wandori.us (Pro) | $4.13  | mensual | `pending` |

---

## Fase 3: Verificación

1. Confirmar que los 4 hostings están creados vía API
2. Confirmar que los 5 billing_items están creados vía API
3. Confirmar que `cap.wandori.us` tiene `stripe_subscription_id = 'sub_1TdRDgCdHJpmDkrr69Vn4grz'`
4. Confirmar que los errores 500 desaparecen en el frontend
5. Confirmar que el banner de restauración ya no aparece

---

## Archivos a Modificar

| Archivo                                  | Cambio                                                                                                   |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `src/handlers/admin_client_bootstrap.rs` | Agregar parámetro `cap_wandori_stripe_subscription_id`, lógica de vinculación Stripe, endpoint de status |
| `src/router.rs`                          | Registrar nuevas rutas                                                                                   |
| `frontend/src/`                          | Componente de banner de restauración en vista admin de hosting                                           |

---

## Resumen de Acciones

| Paso                | Método                     | Requiere SSH |
| ------------------- | -------------------------- | ------------ |
| Fix 500 errors      | `cm restart --name studio` | ❌ No        |
| Restaurar Guillermo | Botón en frontend → API    | ❌ No        |
| Vincular Stripe     | Parámetro en el endpoint   | ❌ No        |
| Verificar           | Frontend + API             | ❌ No        |

---

## Riesgos y Mitigaciones

| Riesgo                            | Mitigación                                                                            |
| --------------------------------- | ------------------------------------------------------------------------------------- |
| Restart no resuelve los 500       | Plan B: ejecutar migraciones vía `cm exec --target postgres`                          |
| Stripe subscription ID incorrecto | Verificar en Stripe Dashboard antes de ejecutar                                       |
| Datos duplicados por re-ejecución | ON CONFLICT en todos los INSERTs (idempotente)                                        |
| Webhook Stripe futuros fallen     | El `stripe_subscription_id` se vincula correctamente, webhooks buscarán por ese campo |

---

---

## Fase 4: Mejoras a coolify-manager-rs

### Problema actual

Cuando la BD se recrea o hay problemas de migración, no hay herramienting en coolify-manager-rs para:
1. Diagnosticar qué tablas/columnas faltan
2. Ejecutar migraciones pendientes remotamente
3. Ejecutar SQL arbitrario contra la BD del contenedor
4. Restaurar datos de clientes vía la API del bootstrap

El comando `run-script` ya existe pero no soporta `--target postgres` de forma nativa para SQL. El comando `exec` permite ejecutar comandos pero no tiene lógica específica para diagnóstico de BD.

### Nuevos comandos propuestos

#### 1. `db-check` — Diagnosticar salud de la BD

```bash
coolify-manager db-check --name studio
coolify-manager db-check --name studio --expected-tables infrastructure_servers,server_capacity
```

**Lógica:**
1. SSH al servidor, encontrar contenedor postgres
2. Ejecutar queries de diagnóstico:
   - `SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'` — listar tablas existentes
   - `SELECT version, installed_on FROM _sqlx_migrations ORDER BY version` — estado de migraciones
   - Para cada tabla esperada, verificar que existe
   - Para columnas críticas, verificar que existen
3. Reportar:
   - Tablas que faltan vs esperadas
   - Migraciones pendientes (en disco pero no en `_sqlx_migrations`)
   - Columnas faltantes en tablas conocidas
   - Estado general: `OK`, `WARN`, `ERROR`

**Salida ejemplo:**
```
[db-check] studio — PostgreSQL health diagnostic
  ✅ 42 tables found
  ✅ 28 migrations applied
  ❌ MISSING TABLE: infrastructure_servers (migration 20260522010000)
  ❌ MISSING TABLE: server_capacity (migration 20260522010000)
  ❌ MISSING COLUMN: hosting_plan_configs.cpu_scaling_policy (migration 20260526000000)
  ⚠️  3 issues found — run `db-migrate` to fix
```

#### 2. `db-migrate` — Ejecutar migraciones pendientes

```bash
coolify-manager db-migrate --name studio
coolify-manager db-migrate --name studio --dry-run
coolify-manager db-migrate --name studio --version 20260522010000
```

**Lógica:**
1. SSH al servidor, encontrar contenedor postgres
2. Verificar si el binario de la app tiene migraciones embebidas (SQLx embed migrations)
3. Si las migraciones están embebidas en el binario:
   - Ejecutar el binario con un flag especial para aplicar migraciones (si la app lo soporta)
   - O extraer las migraciones del binario y ejecutarlas vía psql
4. Si las migraciones están en archivos `.sql`:
   - Subir los archivos de migración al servidor
   - Ejecutarlos en orden vía psql
5. Verificar resultado con `db-check`

**Flujo alternativo (si el binario no tiene migraciones embebidas):**
1. Leer los archivos `.sql` del directorio de migraciones local
2. Subirlos al servidor vía SSH
3. Ejecutar cada migración pendiente en orden
4. Insertar registro en `_sqlx_migrations` para cada una aplicada

#### 3. `run-sql` — Ejecutar SQL arbitrario

```bash
coolify-manager run-sql --name studio --file ./restore.sql
coolify-manager run-sql --name studio --query "SELECT COUNT(*) FROM users"
coolify-manager run-sql --name studio --file ./fix.sql --dry-run
```

**Lógica:**
1. SSH al servidor, encontrar contenedor postgres
2. Si `--file`: subir archivo SQL al servidor, ejecutar vía `psql -f`
3. Si `--query`: ejecutar query directamente vía `psql -c`
4. Si `--dry-run`: envolver en `BEGIN`/`ROLLBACK` para no aplicar cambios
5. Reportar resultado (filas afectadas, errores)

**Reutiliza:** patrón de `run-script` pero optimizado para SQL (sin necesidad de interpreter).

#### 4. `restore-client` — Restaurar datos de cliente vía API

```bash
coolify-manager restore-client --name studio --email guillermo@nakomi.com
coolify-manager restore-client --name studio --email guillermo@nakomi.com --stripe-sub-id sub_1TdRDgCdHJpmDkrr69Vn4grz
coolify-manager restore-client --name studio --email guillermo@nakomi.com --dry-run
```

**Lógica:**
1. SSH al servidor, encontrar contenedor app
2. Ejecutar `curl` dentro del contenedor contra `localhost:PORT/api/admin/client-bootstrap/guillermo`
3. Si `--stripe-sub-id`: después del bootstrap, ejecutar UPDATE SQL para vincular la suscripción
4. Si `--dry-run`: solo verificar estado, no ejecutar
5. Reportar resultado

**Ventaja:** encapsula todo el flujo de restauración en un solo comando reutilizable.

---

### Mapa de implementación

| Comando | Archivo nuevo | Reutiliza | Complejidad |
|---|---|---|---|
| `db-check` | `src/commands/db_check.rs` | `docker::find_postgres_container`, `ssh_client` | Media |
| `db-migrate` | `src/commands/db_migrate.rs` | `db_check`, `run_script` pattern | Alta |
| `run-sql` | `src/commands/run_sql.rs` | `run_script` pattern, `import_database` pattern | Baja |
| `restore-client` | `src/commands/restore_client.rs` | `exec_command`, `run_sql` | Media |

### Cambios en CLI

Agregar al enum `Command` en `src/cli/mod.rs`:
```rust
DbCheck { name, expected_tables },
DbMigrate { name, dry_run, version },
RunSql { name, file, query, dry_run },
RestoreClient { name, email, stripe_sub_id, dry_run },
```

Agregar al dispatch en `src/cli/dispatch.rs`.

### Cambios en MCP

Agregar las 4 herramientas al MCP server para que Copilot pueda usarlas directamente.

---

## Orden de Ejecución

1. **Implementar `run-sql`** — base para los otros comandos
2. **Implementar `db-check`** — diagnóstico inmediato
3. **Implementar `db-migrate`** — fix de migraciones
4. **Implementar `restore-client`** — restauración de datos
5. **Ejecutar `db-check`** en nakomi.studio — diagnosticar errores 500
6. **Ejecutar `db-migrate`** si hay migraciones pendientes
7. **Ejecutar `restore-client`** para Guillermo
8. **Deploy** de la nueva versión de coolify-manager-rs
9. **Verificar** que todo funciona

---

## Lecciones Aprendidas

- La recreación de BD debe incluir re-aplicación de migraciones
- El endpoint bootstrap existente es un buen patrón para restauración de datos de clientes
- La vinculación manual de Stripe subscriptions debería tener un endpoint admin dedicado
- Los errores 500 en endpoints de hosting son indicadores tempranos de problemas de migración
- coolify-manager-rs necesita herramienting de diagnóstico de BD para evitar SSH directo
- El patrón de `run-script` (base64 + docker exec) es reutilizable para `run-sql`
