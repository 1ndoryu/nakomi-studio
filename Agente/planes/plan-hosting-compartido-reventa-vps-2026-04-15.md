# Plan: Hosting Compartido + Reventa VPS Contabo

## Fecha: 2026-04-15
## ID Tarea: 154A-4

---

## 1. Contexto actual

### Infraestructura
- **VPS1** (66.94.100.241): Contabo, ejecuta Nakomi Studio (sitio principal) via Coolify
- **VPS2** (173.249.50.44): Contabo, dedicado a hosting de clientes via Coolify
- Cada cliente recibe un stack Docker aislado: **WordPress + MariaDB + SSH/SFTP** (3 containers por hosting)

### Planes actuales (hosting_plan_configs en BD)

| Plan | Precio/mes | WP CPU | WP RAM | DB CPU | DB RAM | SSH CPU | SSH RAM | Storage | BW |
|------|-----------|--------|--------|--------|--------|---------|---------|---------|-----|
| basico | $5 | 0.50 | 256MB | 0.25 | 256MB | 0.25 | 128MB | 5GB | 50GB |
| pro | $10 | 1.00 | 512MB | 0.50 | 512MB | 0.50 | 256MB | 20GB | 200GB |
| ecommerce | $15 | 1.50 | 1024MB | 0.75 | 512MB | 0.50 | 256MB | 50GB | 500GB |

### Recursos totales por cliente

| Plan | CPU total | RAM total | Containers |
|------|----------|-----------|------------|
| basico | 1.00 cores | 640MB | 3 |
| pro | 2.00 cores | 1280MB | 3 |
| ecommerce | 2.75 cores | 1792MB | 3 (+backup sidecar) |

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

| VPS | vCPU | RAM | SSD | Precio/mes | IP |
|-----|------|-----|-----|-----------|-----|
| Cloud VPS 1 | 4 | 8GB | 200GB | ~€4.99 ($5.50) | 1 incluida |
| Cloud VPS 2 | 6 | 16GB | 400GB | ~€8.99 ($9.90) | 1 incluida |
| Cloud VPS 3 | 8 | 30GB | 800GB | ~€14.99 ($16.50) | 1 incluida |
| Cloud VPS 4 | 12 | 48GB | 1.6TB | ~€26.99 ($29.70) | 1 incluida |

### Densidad por VPS (cuántos hostings caben)

**Cloud VPS 2 (6 vCPU, 16GB RAM, 400GB SSD) — el actual VPS2:**

| Plan | Max por CPU | Max por RAM | Max por disco | Realista* |
|------|-----------|-----------|-------------|-----------|
| basico | 6 | 25 | 80 | **5-6** |
| pro | 3 | 12 | 20 | **3** |
| ecommerce | 2 | 9 | 8 | **2** |

*\*Realista = considerando overhead de Coolify (~1 core, ~2GB RAM), OS, y headroom del 20% para picos.*

**Cloud VPS 3 (8 vCPU, 30GB RAM, 800GB SSD):**

| Plan | Max por CPU | Max por RAM | Max por disco | Realista |
|------|-----------|-----------|-------------|----------|
| basico | 7 | 43 | 160 | **7-8** |
| pro | 4 | 22 | 40 | **4-5** |
| ecommerce | 2 | 16 | 16 | **3** |

---

## 3. Modelo de negocio propuesto: Hosting Compartido

### 3.1 Propuesta de precios cliente vs costo real

**Escenario: VPS2 actual (Cloud VPS 2 — $9.90/mes)**

| Plan | Precio cliente | Clientes por VPS | Ingreso/VPS | Costo VPS | Margen bruto | % Margen |
|------|---------------|-----------------|-------------|-----------|-------------|----------|
| basico | **$5/mes** | 5 | $25/mes | $9.90 | **$15.10/mes** | 60% |
| pro | **$10/mes** | 3 | $30/mes | $9.90 | **$20.10/mes** | 67% |
| ecommerce | **$15/mes** | 2 | $30/mes | $9.90 | **$20.10/mes** | 67% |
| mixto (3 basico + 1 pro) | — | 4 | $25/mes | $9.90 | **$15.10/mes** | 60% |

**Precios sugeridos para mejorar márgenes (sin perder competitividad):**

| Plan | Precio actual | Precio sugerido | Justificación |
|------|--------------|----------------|---------------|
| basico | $5/mes | **$8-10/mes** | Hostinger cobra $9.99, SiteGround $17.99. $8 es competitivo y triplica margen. |
| pro | $10/mes | **$15-20/mes** | SSH + wp-cli + stats incluidos. Más que lo que ofrece hosting compartido standard. |
| ecommerce | $15/mes | **$25-35/mes** | Backup automático + 50GB + recursos dedicados. WooCommerce hosting típico: $25-45. |

**Con precios sugeridos (VPS2, 4 clientes mixtos):**

| Mix | Ingreso/mes | Costo VPS | Margen | % |
|-----|------------|-----------|--------|---|
| 3 basico ($10) + 1 pro ($20) | $50 | $9.90 | **$40.10** | 80% |
| 2 pro ($20) + 1 ecom ($30) | $70 | $9.90 | **$60.10** | 86% |

### 3.2 Modelo de crecimiento: cuándo comprar nuevo VPS

**Trigger de provisioning de nuevo VPS:**
- Cuando un VPS alcanza 70% de capacidad de CPU o RAM
- Auto-detección: query `hosting_plan_configs` × count de suscripciones activas en cada VPS

**Escalado propuesto:**

| Fase | VPS | Clientes | Ingreso estimado | Costo | Margen |
|------|-----|----------|-----------------|-------|--------|
| Fase 1 (actual) | VPS2 (Cloud VPS 2) | 4-6 | $40-60/mes | $9.90 | $30-50 |
| Fase 2 (10+ clientes) | + Cloud VPS 3 | 10-14 | $100-180/mes | $26.40 total | $74-154 |
| Fase 3 (20+ clientes) | + Cloud VPS 4 | 20-30 | $200-400/mes | $56.40 total | $144-344 |

---

## 4. Reventa de VPS completos (producto premium)

Para clientes que necesitan VPS dedicados (apps custom, alto tráfico, ML, etc.).

### 4.1 Modelo de reventa

| Contabo VPS | Costo | Precio reventa sugerido | Margen |
|-------------|-------|------------------------|--------|
| Cloud VPS 1 (4c/8GB) | $5.50 | **$15-20/mes** | 63-73% |
| Cloud VPS 2 (6c/16GB) | $9.90 | **$25-35/mes** | 60-72% |
| Cloud VPS 3 (8c/30GB) | $16.50 | **$40-55/mes** | 59-70% |
| Cloud VPS 4 (12c/48GB) | $29.70 | **$70-95/mes** | 58-69% |

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

**Lo que falta para automatizar reventa:**
1. **Endpoint `POST /api/vps/provision`** — Compra un VPS via Contabo API, lo registra en BD
2. **Tabla `vps_subscriptions`** — Similar a `hosting_subscriptions` pero para VPS dedicados
3. **Setup automático post-compra** — SSH al nuevo VPS, instalar Coolify/Docker, configurar hostname, MOTD, firewall
4. **Stripe product para VPS** — Price IDs separados por tier

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

| # | Tarea | Esfuerzo |
|---|-------|----------|
| 1 | Actualizar precios en `hosting_plan_configs` (basico $8-10, pro $15-20, ecommerce $25-35) + crear Stripe Prices correspondientes | Bajo |
| 2 | Auto-detección de capacidad de VPS: endpoint que muestre uso actual vs límite por VPS | Bajo |
| 3 | Hostname + MOTD branding en containers SSH existentes (ya parcialmente hecho) | Bajo |
| 4 | Rate limit en checkout de hosting para prevenir abuso de provisioning | Bajo |

### Fase 2 — Reventa de VPS

| # | Tarea | Esfuerzo |
|---|-------|----------|
| 5 | Migración `vps_subscriptions` table | Medio |
| 6 | Modelo + repositorio + handlers para VPS subscriptions | Medio |
| 7 | Flujo de aprobación admin (status machine + notificaciones) | Medio |
| 8 | Automatización de setup post-compra via SSH (hostname, MOTD, Coolify, firewall) | Alto |
| 9 | Frontend: panel de VPS para admin y cliente | Medio |
| 10 | Stripe products/prices para planes VPS | Bajo |
| 11 | Landing page pública de VPS con pricing | Medio |

### Fase 3 — Optimización

| # | Tarea | Esfuerzo |
|---|-------|----------|
| 12 | Dashboard de capacidad: uso real de CPU/RAM/disco por VPS con alertas | Medio |
| 13 | Auto-scaling trigger: notificación cuando un VPS alcanza 70% → sugerir comprar más | Bajo |
| 14 | Bastion SSH para white-labeling completo (si volumen lo justifica) | Alto |

---

## 7. Decisiones pendientes del usuario

1. **Precios finales:** ¿Los sugeridos están bien o ajustar? (basico $8-10, pro $15-20, ecom $25-35)
2. **¿Ofrecer reventa VPS desde el inicio o solo hosting WP?** — VPS es más margen pero más complejidad
3. **¿Auto-provisión para hosting WP o también requiere aprobación?** — Sugerencia: auto para WP, manual para VPS
4. **Cuenta Contabo para provisionamiento de VPS nuevos:** ¿Usar contabo-vps2 o crear cuenta nueva?
5. **Branding SSH:** ¿Hostname custom es suficiente o necesitamos bastion desde el inicio?

---

## 8. Resumen ejecutivo

| Concepto | Estado |
|----------|--------|
| Hosting WP compartido | ✅ Funcional, solo falta ajustar precios y capacidad |
| Reventa VPS | ❌ Requiere tabla, flujo aprobación, setup automático |
| SSH sin branding | ⬜ Parcial (containers aislados), falta hostname custom en VPS revendidos |
| Márgenes | ✅ 60-86% en hosting WP, 58-73% en VPS revendidos |
| Aprobación humana | ❌ No existe, solo para VPS; hosting WP puede ser auto |

**Margen potencial con 10 clientes mixtos en 2 VPS (~$26/mes costo):**
- Rango conservador: $80-150/mes ingreso → $54-124/mes margen
- Rango optimista: $150-250/mes ingreso → $124-224/mes margen
