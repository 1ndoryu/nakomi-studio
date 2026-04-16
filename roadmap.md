Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi

## Stack

| Capa | Herramienta |
|------|-------------|
| Framework web | Axum 0.7 |
| OpenAPI | utoipa 4 + utoipa-swagger-ui 7 |
| Base de datos | SQLx 0.8 (PostgreSQL) |
| Validación | validator 0.18 |
| Auth | jsonwebtoken + argon2 |
| Frontend | React 18 + TypeScript + Vite |
| State | React Query + Zustand |
| Codegen | Orval 8 |
| Deploy | coolify-manager-rs |

# Nakomi Studio — Roadmap

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes 

### ~~1. Imágenes pesadas en inicio~~ ✅ COMPLETADO
Causa raíz: image-webp 0.2.4 solo soporta WebP lossless → 1.5MB por imagen. Solucionado: backend ahora devuelve JPEG lossy para peticiones WebP (196KB vs 1.5MB). Frontend deshabilitó srcSet WebP. Cache limpiada.

### ~~2. Auditoría completa del sistema de hosting~~ ✅ COMPLETADO (excepto auto-login WP)

- ✅ Que comprar un hosting wordpress funcione
- ✅ Que el usuario pueda administrar su hosting wordpress
- ⬜ Que pueda ir al wp-admin ya logeado con un boton *(requiere plugin WP — fuera de scope actual)*
- ✅ Que pueda comprar un dominio *(API Contabo Domains integrada — backend + frontend)*
- ✅ Que pueda administrar los archivos de wordpress (SFTP)
- ✅ Que pueda asignar un dominio *(BD + Coolify FQDN update automático)*
- ✅ Que pueda cambiar el dominio *(ídem anterior — reconfigura Coolify al cambiar)*
- ✅ Que pueda transferir un dominio *(API Contabo auth-code + transfer)*
- ✅ Que pueda reiniciar el wordpress, detener, arrancar el wordpress *(endpoints + UI implementados)*
- ✅ Que pueda acceder por ssh
- ✅ Que pueda ver su almacenamiento y status *(du vía SSH ahora en storage_used_mb)*
- ✅ Que pueda ver su ram y status (Docker stats vía SSH)
- ✅ Que no pueda cambiar sus valores asignados
- ✅ Necesito una forma de testear la compra de hosting *(admin-test-subscribe: bypass Stripe)*
- ✅ Que todos los despliegues de vps2 salgan en panel admin (Contabo API)
- ✅ Eliminar hosting del panel borra el hosting real en Coolify
- ✅ Asegurar que VPS1 no esté relacionada (solo VPS2 configurada en CoolifyConfig)
- ✅ Que el cliente pueda cambiar las DNS de su hosting *(DnsManager + client endpoints)*

### ~~3. Error react-helmet-async + redirect a localhost~~ ✅ COMPLETADO
npm install restauró react-helmet-async. Redirect arreglado con resolve_public_base_url() que lee Origin/Referer headers.

### ~~4. Sincronización de env locales con producción~~ ✅ COMPLETADO
Implementado en coolify-manager-rs: comando `sync-env --name <sitio> --direction diff|push|pull [--dry-run] [--env-file <path>]`. Compara .env local con vars de Coolify API, soporta push bulk y pull a archivo local con output coloreado y mascarado para secrets.

### ~~5. API del CMS — edición de contenido~~ ✅ COMPLETADO
Implementado `GET/POST /api/admin/fixtures` para sincronizar archivos TOML de content/ con BD desde el panel admin. Frontend: hook `useFixtureSync` + bloque UI en SeccionConfiguracion.

### Pendientes menores
- ~~Cambiar "Hosting" → "Hosting WordPress" en nav/soluciones con logo WP~~ ✅ COMPLETADO
- ~~Planificar solución de hosting compartido + reventa VPS Contabo~~ ✅ COMPLETADO — Plan implementado en backend/frontend con margen bruto objetivo del 20%, hosting auto-provisionado y VPS con aprobación manual.

- ~~el backend local no funciona~~ ✅ CORREGIDO — BD tenía migraciones de proyecto anterior (restaurant/CRM). Reset DB resolvió `VersionMissing(20260325100000)`.

- ✅ Ya que tenemos una forma de comprar y adquirir dominios tenemos margenes de ganacias (pequeños). 
- ✅ Me falto decir que el cliente pueda cambiar las dns de su hosting *(DnsManager component + client endpoints)*



## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`
- **[164A-18] Fix aplicado**: Dockerfile.rust actualizado con `gosu` entrypoint para corregir permisos de volúmenes montados (`root:root` → `appuser`). Fix inmediato aplicado en producción via chown directo. Detalle en `Agente/completados/tareas-2026-04-16.md`.