# Producto de Correo para Clientes de Hosting — Análisis y Plan

> **Creado:** 2026-05-26
> **Contexto:** Decisión previa de ocultar TabCorreo hasta tener un producto de correo real. Este documento analiza proveedores, costos y hoja de ruta.

---

## Estado actual

Hoy Nakomi solo tiene SMTP transaccional (Brevo via `lettre`) para emails propios de la app. No hay buzones IMAP/POP3, aliases, webmail ni ningún producto de correo para clientes.

---

## Comparativa de proveedores económicos

### 1. MXroute — RECOMENDADO (más económico)

| Plan | Precio/año | Precio/mes | Almacenamiento | Dominios | Cuentas |
|------|-----------:|-----------:|:--------------:|:--------:|:-------:|
| Small | $59 | $4.92 | 10 GB | Ilimitados | Ilimitadas |
| Medium | $69 | $5.75 | 25 GB | Ilimitados | Ilimitadas |
| Large | $79 | $6.58 | 50 GB | Ilimitados | Ilimitadas |
| 100GB | $100 | $8.33 | 100 GB | Ilimitados | Ilimitadas |

**Puntos clave:**
- **Sin costo por buzón** — precio fijo anual por无限 cuentas de correo
- SMTP/IMAP/POP3 completo
- 400 emails/hora por cuenta
- IP pool con alta reputación, reenvío automático si falla
- Roundcube + Crossbox (webmail)
- Sin API de provisioning pública (habría que ver si tienen API privada)
- Reseller plans con panel white-label desde $30/trimestre

**Costo efectivo por cliente:** Con 10 clientes usando correo → ~$0.50/cliente/mes. Con 20 → ~$0.25/cliente/mes.

### 2. Migadu — Alternativa con API

| Plan | Precio/año | Precio/mes | Envíos/día | Recep/día | Almacenamiento |
|------|-----------:|-----------:|:----------:|:---------:|:--------------:|
| Micro | $19 | — | 20 out | 200 in | 5 GB (soft) |
| Mini | $90 | $9 | 100 out | 1000 in | 30 GB (soft) |
| Mid | $290 | $29 | 500 out | 3000 in | 100 GB (soft) |
| Maxi | $990 | $99 | 2000 out | 10000 in | 500 GB (soft) |

**Puntos clave:**
- **API REST documentada** — provisioning automático de buzones
- Dominios y buzones ilimitados en todos los planes
- Pensado para agencias/hosting (hosting third-party domains explícitamente permitido)
- Micro plan es muy barato ($19/año) pero muy limitado (20 out/día)
- Soft limits de almacenamiento (no rebota, pide upgrade si excede)
- Multi-admin en planes Mid+
- Suiza (privacidad)

### 3. Forward Email

| Plan | Precio/mes | Almacenamiento | Dominios |
|------|:----------:|:--------------:|:--------:|
| Free | $0 | Solo forwarding | Ilimitados |
| Enhanced | $3 | 10 GB pool | Ilimitados |
| Team | $9 | 10 GB pool | Ilimitados |

**Puntos clave:**
- Open source, API pública
- 9000 out/mes en Enhanced
- Bueno pero menos maduro para reventa
- Sin panel white-label

### 4. Zoho Mail

| Plan | Precio/usuario/mes | Almacenamiento |
|------|:------------------:|:--------------:|
| Free | $0 (5 users) | 5 GB c/u, sin IMAP |
| Mail Lite | $1 | 5 GB |
| Mail Premium | $4 | 50 GB |

**Puntos clave:**
- **Costo por usuario** — no escala bien para reventa
- API pesada (OAuth)
- Sin panel white-label
- No recomendado para revender

---

## Recomendación

### Ganador: MXroute Small ($59/año)

**Razones:**
1. **Costo fijo más bajo del mercado**: $59/año por buzones ilimitados. No importa si son 5 o 50 clientes.
2. **Sin costo por mailbox**: los planes de competidores cobran por usuario (Zoho $1-4/user) o tienen límites diarios muy ajustados (Migadu Micro: 20 out/día).
3. **Almacenamiento compartido**: 10 GB inicial, Ampliable a 25 GB ($69/año) o 50 GB ($79/año).
4. **400 emails/hora por cuenta**: suficiente para PYMES.
5. **Webmail incluido**: Roundcube + Crossbox.
6. **Reseller plans disponibles**: para cuando crezca.

**Contras:**
- Sin API pública documentada para provisioning automático (habría que automatizar vía panel de management o preguntar a soporte).
- Menos "enterprise" que Migadu.

### Alternativa si necesitamos API: Migadu Mini ($9/mes o $90/año)

- API REST documentada para crear/eliminar buzones programáticamente
- 30 GB, 1000 in/100 out por día
- $9/mes sigue siendo económico

---

## Estructura de producto sugerida

### Incluido en planes de hosting

| Plan | Alias/Reenvío | Buzón IMAP |
|------|:-------------:|:----------:|
| Básico ($2.48) | ❌ | ❌ |
| Pro ($4.13) | ✅ 3 alias (info@, ventas@, soporte@) vía Cloudflare Email Routing | ❌ |
| Avanzado ($6.19) | ✅ 5 alias + 1 buzón IMAP incluido | ✅ 1 mailbox |

### Add-on para cualquier plan

| Add-on | Precio sugerido | Descripción |
|--------|:--------------:|-------------|
| 1 buzón IMAP adicional | **$1.50/mes** | Cuenta individual IMAP/SMTP con 5 GB, webmail |
| Pack 5 buzones | **$5/mes** | 5 cuentas, 25 GB compartidos |
| 1 buzón + dominio extra | **$2.50/mes** | Buzón para dominio adicional del mismo cliente |

**Justificación del precio:**
- MXroute Medium ($69/año = $5.75/mes) nos da 25 GB y buzones ilimitados
- Si 10 clientes contratan 1 mailbox cada uno a $1.50 → $15/mes ingresos vs $5.75/mes costo → **61% margen**
- El primer buzón va incluido en Avanzado para hacer el plan más atractivo
- Los aliases/reenvíos via Cloudflare Email Routing son **gratis**

---

## Lo que habría que implementar

### Fase 1 — Aliases gratis (Cloudflare Email Routing)
- **Sin costo** para Nakomi
- Configurar MX, SPF, DKIM, DMARC del dominio del cliente apuntando a Cloudflare
- Solo reenvío a Gmail/Outlook del cliente
- No requiere IMAP/SMTP
- **Tiempo estimado:** 4-6h backend + 4h frontend

### Fase 2 — Buzones IMAP (MXroute o Migadu)
- Contratar MXroute Small ($59/año) o Migadu Mini ($9/mes)
- Implementar provisioning: crear mailbox vía API (Migadu) o automatización vía panel (MXroute)
- TabCorreo en frontend: listar buzones, crear nuevo, reset password, eliminar
- DNS automático: MX, SPF, DKIM, DMARC
- Billing: Stripe add-on vinculado a la suscripción de hosting
- **Tiempo estimado:** 20-26h (backend + frontend + billing)

### Fase 3 — Reseller (crecimiento)
- MXroute Reseller 75 ($30/trimestre = $10/mes)
- Panel white-label para que clientes gestionen sus buzones
- **Tiempo estimado:** 8-12h integración

---

## Conclusión

**La opción más económica es MXroute Small a $59/año** (o Medium a $69/año) con costo fijo sin importar cuántos buzones se vendan.

Precio al cliente: **$1.50/buzón/mes** como add-on, con 1 buzón incluido en el plan Avanzado para hacerlo más atractivo.

Aliases/reenvíos via Cloudflare Email Routing son gratis y pueden incluirse en planes Pro+ sin costo para Nakomi.

**Próximo paso:** Decidir proveedor (MXroute vs Migadu) y si empezamos solo con aliases (Fase 1) o vamos directo a buzones IMAP (Fase 2).
