# Plan vps.nakomi.studio — 2026-05-10

## Contexto

La GUI de `coolify-manager-rs` deja de ser solo herramienta local: debe poder publicarse como `vps.nakomi.studio`, con login, administración segura y una evolución hacia compra/gestión de VPS vinculada a Nakomi Studio. Esto se cruza con los planes existentes de hosting, dominios, Contabo, Coolify y Stripe.

## Objetivos

1. Mantener una app local rápida y real para operaciones internas.
2. Preparar una versión online segura en `vps.nakomi.studio` sin exponer secretos ni operaciones peligrosas.
3. Separar la futura venta/gestión de VPS de la página principal de Nakomi Studio.
4. Reutilizar lo ya avanzado en hosting/dominios/Contabo en vez de duplicar flujos.

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
- Implementar 105A-28.
- Luego 105A-29, 105A-31, 105A-32 y 105A-33 en bloques UI separados.
- Implementar 105A-30 en Glory Sentinel cuando el nuevo patrón de Select esté claro.

### Fase 2 — Diseño de arquitectura online
- Documento de arquitectura para `vps.nakomi.studio`: runtime, deployment, config, auth, RBAC, auditoría y límites de red.
- Decidir qué endpoints se exponen online y cuáles quedan solo en modo local.
- Definir bootstrap seguro del admin.

### Fase 3 — MVP online seguro
- Login + landing simple.
- Dashboard read-only de VPS/sitios primero.
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

## Estado

- Activo. Primer bloque: 105A-28 optimización/caché.
