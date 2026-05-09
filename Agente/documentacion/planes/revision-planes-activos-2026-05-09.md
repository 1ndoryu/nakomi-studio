# Revisión de planes activos — 2026-05-09

## Criterio usado

Se revisó cada archivo bajo `Agente/planes/` antes de continuar trabajo. Los planes con objetivo ya integrado o superado se movieron a `Agente/planes/completados/`. Los planes parcialmente hechos o estratégicos quedan activos con próxima acción.

## Resultado 1 por 1

### 1. `plan-cms-api-agente-2026-05-15.md`
- **Estado:** Activo, parcialmente hecho.
- **Evidencia:** Plan maestro para API de contenido del agente; fase servicios indicada como implementada, quedan config global/blog.
- **Acción:** Conservar. Próximo bloque: fase config global si vuelve al roadmap.

### 2. `plan-delegaciones-pedidos-2026-04-15.md`
- **Estado:** Activo, parcialmente hecho.
- **Evidencia:** Flujo de órdenes existe; wallet/delegación completa sigue siendo frente grande.
- **Acción:** Conservar. No se mezcla con chatbot hosting por dominio distinto.

### 3. `plan-dominios-2026-04-07.md`
- **Estado:** Pendiente/bloqueado por proveedor e infra.
- **Evidencia:** Requiere decisión de proveedor/API de dominios.
- **Acción:** Conservar como backlog de infraestructura.

### 4. `plan-glory-rs-seed-system-2026-04-07.md`
- **Estado:** Completado.
- **Evidencia:** El repo ya usa `content/` y fixtures TOML como sistema operativo.
- **Acción:** Movido a `Agente/planes/completados/`.

### 5. `plan-hosting-automation-2026-04-10.md`
- **Estado:** Activo, parcialmente hecho.
- **Evidencia:** Existe self-service + checkout Stripe; sigue pendiente cierre completo de provisioning post-checkout, sincronización de impagos/cancelación y DNS.
- **Acción:** Conservar. Relacionado con el plan nuevo de chatbot, pero no se resuelve desde el chat porque provisioning sigue admin-only.

### 6. `plan-hosting-compartido-reventa-vps-2026-04-15.md`
- **Estado:** Completado como decisión técnica/comercial.
- **Evidencia:** Define estrategia, márgenes y densidades; ya alimentó precios/configuración.
- **Acción:** Movido a `Agente/planes/completados/`.

### 7. `plan-seguridad-hosting-2026-04-16.md`
- **Estado:** Activo, casi completo.
- **Evidencia:** Quedan validaciones de ownership DNS/monitoreo y no conviene mezclarlo con chatbot.
- **Acción:** Conservar. Próximo bloque de seguridad hosting.

### 8. `plan-sentinel-problemas-reales-2026-04-12.md`
- **Estado:** Activo como backlog técnico.
- **Evidencia:** Describe refactor grande de handlers/repositorios; no era plan listo para archivar.
- **Acción:** Conservar como referencia para futuros bloques de deuda backend.

### 9. `plan-sentinel-solid-rust-2026-04-11.md`
- **Estado:** Completado.
- **Evidencia:** Las reglas ya están operativas en Sentinel.
- **Acción:** Movido a `Agente/planes/completados/`.

### 10. `plan-sentinel-varsense-editor-agnostico-2026-05-08.md`
- **Estado:** Activo, en ejecución.
- **Evidencia:** VarSense LSP/Zed ya avanzó, pero Sentinel equivalente/server combinado queda pendiente.
- **Acción:** Conservar.

### 11. `plan-seo-completo-2026-04-04.md`
- **Estado:** Completado.
- **Evidencia:** Prerender/sitemap/SEO técnico ya forman parte del proyecto.
- **Acción:** Movido a `Agente/planes/completados/`.

### 12. `plan-ssh-sftp-seguro-2026-04-16.md`
- **Estado:** Activo, casi completo.
- **Evidencia:** Acceso SSH/SFTP está diseñado/implementado parcialmente; queda verificación operativa de VFS/cuotas.
- **Acción:** Conservar. Requiere bloque infra separado.

### 13. `plan-testing-chatbot-e2e-2026-04-10.md`
- **Estado:** Completado/obsoleto.
- **Evidencia:** Era plan inicial de smoke/unit tests del chatbot; el chatbot ya superó esa etapa y ahora existe plan específico de hosting.
- **Acción:** Movido a `Agente/planes/completados/`.

### 14. `plan-chatbot-hosting-asistencia-2026-05-09.md`
- **Estado:** Completado en fase accionable.
- **Evidencia:** Documenta huecos reales del bot en compra/cobro/gestión/asesoría de hosting.
- **Acción:** Movido a `Agente/planes/completados/` tras implementar tools, prompt, rich message y tests de guardrails.
