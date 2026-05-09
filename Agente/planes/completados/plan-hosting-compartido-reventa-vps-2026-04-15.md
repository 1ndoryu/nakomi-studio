# Plan: Hosting Compartido + Reventa VPS Contabo

## Fecha: 2026-04-15

## ID Tarea: 154A-4

## Decisión aplicada: margen bruto objetivo del 20% en hosting compartido y reventa VPS

---

## 1. Contexto actual

### Infraestructura

- **VPS1** (66.94.100.241): Contabo, ejecuta Nakomi Studio (sitio principal) via Coolify
- **VPS2** (173.249.50.44): Contabo, dedicado a hosting de clientes via Coolify
- Cada cliente recibe un stack Docker aislado: **WordPress + MariaDB + SSH/SFTP** (3 containers por hosting)

### Planes actuales (hosting_plan_configs en BD)

| Plan      | Precio/mes | WP CPU | WP RAM | DB CPU | DB RAM | SSH CPU | SSH RAM | Storage | BW    |
| --------- | ---------- | ------ | ------ | ------ | ------ | ------- | ------- | ------- | ----- |
| basico    | $5         | 0.50   | 256MB  | 0.25   | 256MB  | 0.25    | 128MB   | 5GB     | 50GB  |
| pro       | $10        | 1.00   | 512MB  | 0.50   | 512MB  | 0.50    | 256MB   | 20GB    | 200GB |
| ecommerce | $15        | 1.50   | 1024MB | 0.75   | 512MB  | 0.50    | 256MB   | 50GB    | 500GB |

### Recursos totales por cliente

| Plan      | CPU total  | RAM total | Containers          |
| --------- | ---------- | --------- | ------------------- |
| basico    | 1.00 cores | 640MB     | 3                   |
| pro       | 2.00 cores | 1280MB    | 3                   |
| ecommerce | 2.75 cores | 1792MB    | 3 (+backup sidecar) |

### Flujo actual de compra

1. Cliente selecciona plan en la web pública
2. Se crea `hosting_subscription` en BD con status `pending`
3. Redirige a Stripe Checkout (suscripción mensual)
4. Webhook `invoice.paid` activa el hosting → status `provisioning`
5. Backend auto-provisiona en Coolify (crea servicio Docker compose)
6. Status → `active`, credenciales SSH/SFTP generadas

---

## 2. Análisis de costos Contabo (precios abril 2026)

### VPS disponibles para reventa

| VPS         | vCPU | RAM  | SSD   | Precio/mes       | IP         |
| ----------- | ---- | ---- | ----- | ---------------- | ---------- |
| Cloud VPS 1 | 4    | 8GB  | 200GB | ~€4.99 ($5.50)   | 1 incluida |
| Cloud VPS 2 | 6    | 16GB | 400GB | ~€8.99 ($9.90)   | 1 incluida |
| Cloud VPS 3 | 8    | 30GB | 800GB | ~€14.99 ($16.50) | 1 incluida |
| Cloud VPS 4 | 12   | 48GB | 1.6TB | ~€26.99 ($29.70) | 1 incluida |

### Densidad por VPS (cuántos hostings caben)

**Cloud VPS 2 (6 vCPU, 16GB RAM, 400GB SSD) — el actual VPS2:**

| Plan      | Max por CPU | Max por RAM | Max por disco | Realista\* |
| --------- | ----------- | ----------- | ------------- | ---------- |
| basico    | 6           | 25          | 80            | **5-6**    |
| pro       | 3           | 12          | 20            | **3**      |
| ecommerce | 2           | 9           | 8             | **2**      |

_\*Realista = considerando overhead de Coolify (~1 core, ~2GB RAM), OS, y headroom del 20% para picos._

**Cloud VPS 3 (8 vCPU, 30GB RAM, 800GB SSD):**

| Plan      | Max por CPU | Max por RAM | Max por disco | Realista |
| --------- | ----------- | ----------- | ------------- | -------- |
| basico    | 7           | 43          | 160           | **7-8**  |
| pro       | 4           | 22          | 40            | **4-5**  |
| ecommerce | 2           | 16          | 16            | **3**    |

---

## 3. Modelo de negocio propuesto: Hosting Compartido

### 3.1 Precios finales con margen bruto objetivo del 20%

**Escenario: VPS2 actual (Cloud VPS 2 — $9.90/mes)**

| Plan                     | Costo unitario estimado | Precio cliente | Clientes por VPS | Ingreso/VPS | Costo VPS | Margen bruto  | % Margen |
| ------------------------ | ----------------------- | -------------- | ---------------- | ----------- | --------- | ------------- | -------- |
| basico                   | $1.98                   | **$2.48/mes**  | 5                | $12.40/mes  | $9.90     | **$2.50/mes** | 20%      |
| pro                      | $3.30                   | **$4.13/mes**  | 3                | $12.39/mes  | $9.90     | **$2.49/mes** | 20%      |
| ecommerce                | $4.95                   | **$6.19/mes**  | 2                | $12.38/mes  | $9.90     | **$2.48/mes** | 20%      |
| mixto (3 basico + 1 pro) | —                       | —              | 4                | $11.57/mes  | $9.90     | **$1.67/mes** | 14%      |

**Precios operativos fijados por el usuario:**

| Plan      | Precio anterior | Precio final | Fórmula usada |
| --------- | --------------- | ------------ | ------------- |
| basico    | $5/mes          | **$2.48/mes** | costo unitario / 0.80 |
| pro       | $10/mes         | **$4.13/mes** | costo unitario / 0.80 |
| ecommerce | $15/mes         | **$6.19/mes** | costo unitario / 0.80 |

**Nota operativa:**

- El 20% se cumple por tier cuando el VPS se llena según la capacidad conservadora usada para fijar precio: 5 básicos, 3 pro o 2 ecommerce.
- En mezclas con capacidad ociosa, el margen efectivo cae por debajo del 20% hasta completar ocupación del nodo.
- El checkout quedó implementado con `price_data` dinámico en Stripe, así que no depende de Price IDs fijos por plan.

### 3.2 Modelo de crecimiento: cuándo comprar nuevo VPS

**Trigger de provisioning de nuevo VPS:**

- Cuando un VPS alcanza 70% de capacidad de CPU o RAM
- Auto-detección: query `hosting_plan_configs` × count de suscripciones activas en cada VPS

**Escalado propuesto:**

| Fase                  | VPS                | Clientes | Ingreso objetivo | Costo        | Margen objetivo |
| --------------------- | ------------------ | -------- | ---------------- | ------------ | ---------------- |
| Fase 1 (actual)       | VPS2 (Cloud VPS 2) | 4-6      | ~$12.40/mes      | $9.90        | ~$2.50           |
| Fase 2 (10+ clientes) | + Cloud VPS 3      | 10-14    | ~$33.00/mes      | $26.40 total | ~$6.60           |
| Fase 3 (20+ clientes) | + Cloud VPS 4      | 20-30    | ~$70.13/mes      | $56.10 total | ~$14.03          |

---

## 4. Reventa de VPS completos (producto premium)

Para clientes que necesitan VPS dedicados (apps custom, alto tráfico, ML, etc.).

### 4.1 Modelo de reventa

| Contabo VPS            | Costo  | Precio final | Margen |
| ---------------------- | ------ | ------------ | ------ |
| Cloud VPS 1 (4c/8GB)   | $5.50  | **$6.88/mes**  | 20% |
| Cloud VPS 2 (6c/16GB)  | $9.90  | **$12.38/mes** | 20% |
| Cloud VPS 3 (8c/30GB)  | $16.50 | **$20.63/mes** | 20% |
| Cloud VPS 4 (12c/48GB) | $29.70 | **$37.13/mes** | 20% |

### 4.2 SSH sin branding de Contabo

**Problema:** Si damos acceso SSH directo al VPS de Contabo, el cliente ve el hostname de Contabo, MOTD de Contabo, y podría descubrir que está en un VPS de $5.

**Soluciones (de menor a mayor esfuerzo):**

1. **Custom MOTD + hostname (mínimo viable):**
    - Cambiar hostname: `hostnamectl set-hostname cliente-vps.nakomi.studio`
    - Reemplazar `/etc/motd` y `/etc/issue` con branding Nakomi
    - Provisionar via coolify-manager-rs al crear el VPS
    - **Esfuerzo: bajo. Cubre 90% de casos.**

2. **Jump host / bastion (intermedio):**
    - Cliente conecta a `ssh.nakomi.studio` (nuestro bastion)
    - Bastion redirecciona a su VPS real via ProxyJump
    - El cliente nunca ve la IP ni hostname de Contabo
    - **Esfuerzo: medio. Requiere un mini VPS dedicado como bastion.**

3. **Container SSH aislado (ya implementado para hosting WP):**
    - El container SSH de `linuxserver/openssh-server` ya corre con hostname custom
    - No expone el host real — el cliente solo ve el container
    - Para VPS revendidos: extender el container SSH para dar acceso a TODO el VPS via Docker socket (con restricciones)
    - **Esfuerzo: bajo si se reutiliza lo existente.**

**Recomendación:** Opción 1 para VPS revendidos + Opción 3 (ya existente) para hosting WP. Si crece el volumen de VPS revendidos, migrar a Opción 2.

### 4.3 Gestión automática de VPS Contabo

**Lo que existe:**

- Contabo API ya integrada (2 cuentas: `contabo-vps1` y `contabo-vps2` en settings.json)
- Panel admin ya lista VPS de Contabo (`/api/hosting/vps`)
- Dominios Contabo: compra, transfer, cambio DNS ya implementados

**Implementado en esta iteración:**

1. **Catálogo y tabla `vps_subscriptions`** — Ya existe junto con `vps_plan_configs` y `vps_events`
2. **Handlers y repositorio de VPS** — Listado público, suscripción, aprobación y rechazo admin
3. **Checkout Stripe para VPS** — Implementado con `price_data` dinámico por tier
4. **Provisioning inicial en Contabo** — Crea la instancia y aplica bootstrap inicial de hostname, MOTD, Docker y firewall vía cloud-init

**Lo que sigue faltando para endurecer operación real:**

1. Instalar y validar stack adicional post-compra cuando el VPS lo requiera (por ejemplo Coolify completo)
2. Telemetría de capacidad y alertas por nodo
3. White-labeling avanzado con bastion si el volumen comercial lo justifica

---

## 5. Aprobación humana obligatoria

### 5.1 Por qué es necesaria

- Un VPS revendido cuesta $5-30/mes — provisionar sin revisión puede generar fraude
- Hosting compartido es más tolerable para auto-provisión (costo marginal bajo)
- Reventa VPS requiere comprar hardware real a Contabo — irreversible a corto plazo

### 5.2 Flujo propuesto

```
HOSTING COMPARTIDO (auto-provisión OK):
  Cliente paga en Stripe → webhook → auto-provision en VPS existente → active

VPS REVENDIDO (aprobación manual):
  Cliente paga en Stripe → webhook → status = "pending_approval"
  → Admin recibe notificación (email + panel)
  → Admin revisa y aprueba en panel
  → POST /api/vps/{id}/approve → compra VPS en Contabo → setup automático → active
  → Si admin rechaza → refund en Stripe → status = "rejected"
```

### 5.3 Implementación técnica

**Tabla `vps_subscriptions`:**

- Campos: id, user_id, tier (vps1/vps2/vps3/vps4), status, contabo_instance_id, ip, hostname
- Status: pending_payment → pending_approval → provisioning → active → suspended → cancelled

**Endpoint `POST /api/admin/vps/{id}/approve`:**

- Solo admin
- Llama a Contabo API para instanciar VPS
- SSH setup automático (hostname, MOTD, firewall, Docker)
- Registra IP y credenciales en BD
- Notifica al cliente

**Notifications:**

- `NOTIF_VPS_PENDING_APPROVAL` → admin ve badge en panel
- `NOTIF_VPS_APPROVED` → cliente recibe email con credenciales
- `NOTIF_VPS_REJECTED` → cliente recibe email con razón + refund

---

## 6. Tareas de implementación (ordenadas por prioridad)

### Fase 1 — Mejoras inmediatas al hosting compartido existente

| #   | Tarea                                                                                                                            | Esfuerzo |
| --- | -------------------------------------------------------------------------------------------------------------------------------- | -------- |
| 1   | Actualizar precios en `hosting_plan_configs` a margen bruto 20% (basico $2.48, pro $4.13, ecommerce $6.19)                     | Bajo     |
| 2   | Auto-detección de capacidad de VPS: endpoint que muestre uso actual vs límite por VPS                                            | Bajo     |
| 3   | Hostname + MOTD branding en containers SSH existentes (ya parcialmente hecho)                                                    | Bajo     |
| 4   | Rate limit en checkout de hosting para prevenir abuso de provisioning                                                            | Bajo     |

### Fase 2 — Reventa de VPS

| #   | Tarea                                                                           | Esfuerzo |
| --- | ------------------------------------------------------------------------------- | -------- |
| 5   | Migración `vps_subscriptions` table                                             | Medio    |
| 6   | Modelo + repositorio + handlers para VPS subscriptions                          | Medio    |
| 7   | Flujo de aprobación admin (status machine + notificaciones)                     | Medio    |
| 8   | Automatización de setup post-compra via SSH (hostname, MOTD, Coolify, firewall) | Alto     |
| 9   | Frontend: panel de VPS para admin y cliente                                     | Medio    |
| 10  | Checkout Stripe dinámico para planes VPS                                        | Bajo     |
| 11  | Landing page pública de VPS con pricing                                         | Medio    |

### Fase 3 — Optimización

| #   | Tarea                                                                              | Esfuerzo |
| --- | ---------------------------------------------------------------------------------- | -------- |
| 12  | Dashboard de capacidad: uso real de CPU/RAM/disco por VPS con alertas              | Medio    |
| 13  | Auto-scaling trigger: notificación cuando un VPS alcanza 70% → sugerir comprar más | Bajo     |
| 14  | Bastion SSH para white-labeling completo (si volumen lo justifica)                 | Alto     |

---

## 7. Decisiones y ajustes abiertos

1. **Precios finales:** fijados a margen bruto objetivo del 20% en hosting y VPS.
2. **Alcance comercial:** reventa VPS habilitada desde el inicio, con aprobación manual.
3. **Provisioning:** hosting WP sigue auto-provisionado; VPS sigue con aprobación humana obligatoria.
4. **Cuenta Contabo para provisionamiento de VPS nuevos:** pendiente definir si se mantiene la cuenta actual o se separa por facturación.
5. **Branding SSH:** hostname custom + MOTD quedan como baseline; bastion sigue siendo optimización futura.

---

## 8. Resumen ejecutivo

| Concepto              | Estado                                                                    |
| --------------------- | ------------------------------------------------------------------------- |
| Hosting WP compartido | ✅ Funcional, con pricing ajustado a 20% bruto; falta dashboard de capacidad |
| Reventa VPS           | ✅ Base implementada: catálogo, checkout, approval flow, panel y landing  |
| SSH sin branding      | ✅ Baseline resuelto con hostname + MOTD custom; bastion queda opcional   |
| Márgenes              | ✅ Objetivo fijado en 20% bruto para hosting WP y VPS revendidos          |
| Aprobación humana     | ✅ VPS manual; hosting WP sigue auto                                      |

**Margen potencial con 10+ clientes y 2 VPS (~$26.40/mes costo):**

- Ingreso objetivo a 20% bruto: ~$33.00/mes
- Margen objetivo agregado: ~$6.60/mes
