# Plan vps.nakomi.studio — 2026-05-10

## Contexto

La GUI de `coolify-manager-rs` deja de ser solo herramienta local: debe poder publicarse como `vps.nakomi.studio`, con login, administración segura y una evolución hacia compra/gestión de VPS vinculada a Nakomi Studio. Esto se cruza con los planes existentes de hosting, dominios, Contabo, Coolify y Stripe.

## Objetivos

1. Mantener una app local rápida y real para operaciones internas.
2. Preparar una versión online segura en `vps.nakomi.studio` sin exponer secretos ni operaciones peligrosas.
3. Separar la futura venta/gestión de VPS de la página principal de Nakomi Studio.
4. Reutilizar lo ya avanzado en hosting/dominios/Contabo en vez de duplicar flujos.

## Decisiones base

1. `vps.nakomi.studio` será un producto/portal separado de la web principal, no una sección más del marketing de Nakomi Studio.
2. El primer MVP online será read-only para infraestructura: VPS, sitios, métricas, copias y eventos. Las acciones write se habilitan después de auth/RBAC/auditoría.
3. La GUI local y la app online comparten componentes y lenguaje visual, pero no comparten el mismo boundary de permisos.
4. El navegador público nunca debe leer `settings.json`, tokens de Coolify, claves SSH ni paths internos.
5. El deploy de `vps.nakomi.studio` se hará con `coolify-manager-rs` y Coolify, igual que `nakomi.studio`; SSH directo queda solo para emergencia documentada.
6. La compra directa de servidores VPS no está implementada todavía; hoy existe gestión de VPS ya configuradas, DNS Contabo y provisioning de stacks Coolify.

## Arquitectura objetivo

### Capas

- **Frontend público/admin:** React + Vite, estilo consola operativa, login obligatorio para cualquier panel real.
- **Backend portal VPS:** Rust/Axum con API propia para auth, dashboard, auditoría y acciones permitidas.
- **Orquestador interno:** `coolify-manager-rs` como librería/servicio interno para hablar con Coolify, SSH, backups y métricas.
- **Fuentes externas:** Coolify API para servicios/despliegues; Contabo API para DNS y futuro inventario/provisión; Stripe para billing.

### Modos runtime

- **Local operator mode:** GUI local con `gui-api` y acceso completo a configuración local del operador.
- **Online admin mode:** `vps.nakomi.studio` con auth fuerte, endpoints filtrados y operaciones auditadas.
- **Customer mode futuro:** panel de cliente con solo sus VPS/servicios, sin acceso a targets globales ni acciones administrativas.

### Separación de secretos

- Secrets de Coolify/SSH/Contabo/Stripe viven solo en env del backend online o en secret store del deploy.
- El frontend recibe solo datos normalizados: nombres, estados, métricas, timestamps, IDs públicos y mensajes filtrados.
- Logs remotos se exponen truncados, redactados y bajo permiso explícito.

## Modelo funcional

### MVP read-only

- Landing breve en `vps.nakomi.studio` con entrada a login.
- Login admin.
- Dashboard de VPS: host, target, CPU/RAM/disco, servicios por target y alertas.
- Tabla de sitios: dominio, target, estado, plantilla, métricas y último check.
- Tabla de copias: sitio, tipo, fecha, tamaño/artefactos cuando exista.
- Página de eventos/auditoría de lecturas y acciones.

### MVP write controlado

- Refrescar métricas/health bajo rate limit.
- Crear backup manual con confirmación.
- Ver logs filtrados.
- Reiniciar/redeploy solo admin, con confirmación fuerte, auditoría y guardas de bind mounts.

### Producto VPS futuro

- Catálogo de planes VPS administrados.
- Checkout Stripe con estado de provisioning idempotente.
- Provisioning: crear/asignar VPS, instalar Coolify si aplica, crear proyecto/servicio y enlazar dominio.
- Panel cliente: credenciales, accesos, facturación, estado, tickets y copias.

## Seguridad obligatoria

### Auth

- Bootstrap admin por CLI/env one-shot: email + password temporal o invite token.
- Password hash con Argon2.
- Sesión con cookie `HttpOnly Secure SameSite=Lax` o JWT de vida corta + refresh seguro.
- Rate limit por IP/cuenta para login, refresh y acciones críticas.

### RBAC

- `admin`: configura targets, ve todos los servicios, ejecuta acciones críticas.
- `operator`: ve todo y puede ejecutar acciones no destructivas autorizadas.
- `viewer`: solo lectura.
- `customer`: solo recursos asociados a su cuenta.

### Auditoría

- Registrar actor, rol efectivo, acción, target, sitio, input normalizado, resultado, duración e IP.
- Nunca registrar secretos, tokens, passwords, headers completos ni logs crudos.
- Las acciones write deben generar evento antes y después de ejecutarse.

### Red y exposición

- CORS cerrado al dominio de producción.
- HTTPS obligatorio.
- CSP básica para scripts/assets propios.
- Límites de payload y timeout en todas las llamadas externas.
- Health público mínimo; health profundo solo autenticado.

## Integración con lo existente

### Nakomi Studio

- La web principal conserva marketing, links y funnels públicos.
- El launcher de apps apunta a `vps.nakomi.studio`, pero no traslada lógica de infraestructura al frontend público.
- Las secciones actuales de VPS/hosting en Nakomi se deben revisar antes de mover flujos para no romper compras existentes.

### Planes relacionados

- `plan-hosting-automation-2026-04-10.md`: base para provisioning tras Stripe.
- `plan-dominios-2026-04-07.md`: base para dominios/DNS.
- `plan-hosting-v2-2026-04-09.md`: referencia de endpoints y panel existente.
- `coolify-manager-rs`: fuente de operaciones reales y guardas de deploy/backups.

## Backlog ejecutable siguiente

### 105A-40 — Documento de arquitectura técnica
- Crear documento específico con runtime, envs, rutas, tablas, servicios internos y flujo de deploy.
- Decidir si el backend vive en este repo o en un nuevo proyecto Rust derivado.
- Estado: completado para el primer corte. Backend y frontend viven en este repo, con dominio extra apuntando al mismo stack Rust.

### 105A-41 — Auth admin bootstrap
- Implementar login mínimo y bootstrap admin seguro.
- Cubrir tests de hash, rate limit y sesión.

### 105A-42 — API read-only de infraestructura
- Exponer targets, sitios, métricas y backups con DTOs seguros.
- Sin acciones write.

### 105A-43 — Landing + dashboard online
- Crear primera pantalla real de `vps.nakomi.studio`: landing compacta + login + dashboard read-only.
- Estado: primer corte completado. `vps.nakomi.studio/` sirve la experiencia VPS real y `/soluciones/vps` dejó de ser placeholder. Dashboard dedicado read-only queda como siguiente hardening visual.

### 105A-44 — Auditoría y permisos write
- Agregar tabla/eventos de auditoría.
- Habilitar backup manual/logs/restart/redeploy por fases.

### 105A-45 — Migración comercial VPS
- Auditar lo existente en Nakomi Studio sobre VPS/hosting.
- Definir qué se queda como marketing y qué se mueve al portal VPS.

## Backlog anotado

### 105A-28 — Optimización y caché de GUI operativa
- `list_sites` no debe bloquear hasta completar health-check de todos los sitios.
- Health-checks deben ejecutarse en segundo plano y con concurrencia limitada.
- `list_all_backups` debe evitar reconstruir cliente remoto por cada sitio.
- `gui-api` debe cachear lecturas caras con TTL y permitir `force` en acciones de refresco.

### 105A-29 — Selector global de VPS en logoSidebar
- El cambio de VPS debe vivir en la zona de marca/sidebar, no como select suelto en cada vista.
- El selector debe ser global y afectar Panel/Sitios/Copias cuando aplique.
- No usar `<select>` nativo.

### 105A-30 — Sentinel contra selects nativos
- Glory Sentinel debe detectar `<select>` nativo en React/TSX.
- La regla debe recomendar componente base tipo `Select`/`Combobox` como en Nakomi Studio.
- Cubrir con fixture/test de regla.

### 105A-31 — Agregar sitio como modal funcional
- El botón `Agregar sitio` debe abrir modal, no navegar a Ajustes.
- Modal con validación y feedback visible.
- Conectar a API real cuando exista endpoint seguro; si falta, mostrar estado explícito y planificar endpoint.

### 105A-32 — Retirar rutaPagina
- Eliminar `rutaPagina` de las vistas porque añade ruido visual sin valor operativo.
- Ajustar espaciado del header tras quitarlo.

### 105A-33 — Favicons inline para sitios
- La tabla de sitios debe usar favicon real del dominio como icono principal.
- Fallback determinista si el favicon no carga.
- Evitar que errores de favicon ralenticen el listado.

### 105A-34 — Despliegue online `vps.nakomi.studio`
- Definir si se despliega como servicio Rust independiente en Coolify.
- Dominio: `vps.nakomi.studio`.
- La app online debe detectar su propio deployment y esconder/limitar operaciones locales no aplicables.
- Todo deploy/health/logs via `coolify-manager-rs`, sin SSH directo salvo emergencia documentada.

### 105A-35 — Login y usuario admin seguro
- Login obligatorio antes de cualquier dato operativo.
- Usuario admin inicial creado de forma segura por env/CLI one-shot, nunca hardcodeado.
- Password hash con Argon2 o equivalente ya usado por Nakomi.
- Sesiones/JWT con expiración, CSRF/cookies seguras si aplica.

### 105A-36 — Seguridad operativa online
- RBAC: admin vs viewer vs futuro cliente VPS.
- Rate limiting para login y endpoints de operación.
- Auditoría de acciones: quién hizo qué, cuándo, target y resultado.
- No exponer paths, secrets, tokens, SSH keys ni logs completos sin filtros.
- Confirmación fuerte para acciones destructivas.

### 105A-37 — Integración VPS/hosting/Nakomi Studio
- Revisar lo ya avanzado en hosting, dominios, Stripe y Contabo.
- Mover compra/gestión de VPS fuera de la página principal hacia `vps.nakomi.studio`.
- Mantener en Nakomi Studio solo marketing/entrada comercial y enlaces al portal VPS.
- Definir modelo de producto: VPS administrada, VPS con Coolify, hosting compartido y dominios.

## Fases de trabajo

### Fase 1 — Rendimiento local y UX base
- 105A-28 completado: listados y caché de GUI operativa.
- Luego 105A-29, 105A-31, 105A-32 y 105A-33 en bloques UI separados.
- Implementar 105A-30 en Glory Sentinel cuando el nuevo patrón de Select esté claro.

### Fase 2 — Diseño de arquitectura online
- Documento de arquitectura para `vps.nakomi.studio`: runtime, deployment, config, auth, RBAC, auditoría y límites de red. Completado en `Agente/documentacion/vps/arquitectura-vps-nakomi-studio-2026-05-10.md`.
- Decidir qué endpoints se exponen online y cuáles quedan solo en modo local.
- Definir bootstrap seguro del admin.

### Fase 3 — MVP online seguro
- Primer corte online: root del subdominio sirve la experiencia VPS real y reutiliza auth/checkout/panel existentes.
- Dashboard read-only dedicado de VPS/sitios queda pendiente como mejora incremental antes de habilitar nuevas acciones write.
- Operaciones write detrás de permisos, confirmaciones y auditoría.
- Deploy por Coolify con health y rollback.

### Fase 4 — Producto VPS
- Revisar y extraer flujos actuales de hosting/dominios en Nakomi Studio.
- Diseñar compra/gestión de VPS directa en `vps.nakomi.studio`.
- Integrar Stripe/Contabo/Coolify progresivamente con idempotencia y estados de provisioning.

## Riesgos

- Exponer una herramienta interna online sin auth/RBAC suficiente.
- Mezclar app local con app pública y filtrar secretos de `settings.json`.
- Acciones lentas o destructivas disparadas desde navegador público sin auditoría.
- Duplicar lógica ya existente en Nakomi Studio en vez de reutilizar servicios.

## Criterios de listo

- Existe una arquitectura documentada antes de implementar el portal público.
- Ningún endpoint online expone secretos, configuración cruda o operaciones sin auth.
- El MVP online arranca read-only y solo luego habilita write con RBAC/auditoría.
- Las rutas de compra/provisioning tienen estados idempotentes y recuperación documentada.
- `nakomi.studio` queda como marketing/entrada, `vps.nakomi.studio` como operación/producto.

## No hacer todavía

- No mover pagos de hosting/VPS sin auditar los endpoints actuales.
- No exponer la GUI local tal cual en internet.
- No permitir deploy/restart desde navegador público sin auditoría y confirmaciones.
- No implementar compra directa de VPS hasta confirmar proveedor, costos, cuotas y flujo antifraude.

## Estado

- Plan terminado y primer corte online implementado. Completados: 105A-28 optimización/caché, 105A-40 arquitectura técnica, 105A-43 landing/routing inicial de `vps.nakomi.studio`.
