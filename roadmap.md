# Roadmap — NAKOMI

Nakomi Studio — sitio web de agencia creativa migrado de WordPress a Rust (Axum 0.7 + SQLx/PostgreSQL,
utoipa) con SPA React 18 + TypeScript + Vite (React Query, Zustand, Orval). Producción en
https://nakomi.studio (VPS1, servicio Coolify `do8k4w8swccwwogoc0os0ck0`); deploy solo vía
`coolify-manager-rs`. Rama `main`. Admin: `andoryyu@gmail.com`. WhatsApp público de soporte (CTA del
chat): `+1 (814) 957-5416`, vía `PUBLIC_SUPPORT_WHATSAPP`; el canal de la API de Meta
(`META_WHATSAPP_NUMBER`) es otro número.

## Siguiente bloque

- [ ] `25AA-1` — **Quitar los estilos de botón propios de los CSS de componente y usar `Button`.**
  Criterio: `problemas --proyecto NAKOMI` sin `estiloBotonEspecifico` (8 casos: chat, panel de correo); el CSS
  de componente solo conserva posición o indicadores; type-check y tests de lo tocado en verde en la 89.
- [ ] `25AA-2` — **Cerrar las alertas de vulnerabilidad de npm y cargo.**
  Criterio: `problemas --proyecto NAKOMI` sin `[vulnerabilidad]`; parche mínimo o excepción documentada en el
  fichero de excepciones del proyecto.

## Pendientes

- [ ] `19AA-1` — **Deploy producción: verificar el estado real (solo diagnóstico).** `coolify-manager-rs
  health --name studio` y comparar `git rev-parse HEAD` con el commit desplegado; reportar. Sin deploy.
- [ ] `19AA-2` — **Hotfix SQL: confirmar en producción.** Los bugs `ON CONFLICT` ya están corregidos en
  código; falta confirmar que la versión desplegada los incluye.
- [ ] `229A-1` — excepción reabierta por `09AA-1` (cambia firmas públicas).

## Bloqueos

- Bloqueo `12AA-1` — probar el deploy desde los repos propios: `settings.json` ya apunta `studio` a `nakomi-studio@main` y `kamples` a `kamples@main`, pero los contenedores sirven la imagen anterior. Requiere autorización de la usuaria para un deploy/`sync` de prueba y actualizar el `REPO_URL` del compose; sin ese paso el próximo deploy fallaría por rama inexistente.
- **Deploy a producción** — bloqueado en autorización de la usuaria (§9.1). Incluye `277A-1`–`277A-5`
  (fixes preexistentes), `277A-6` (watchdog doble señal; implementado, tests 8/8, commits `a944de64`
  + `4d95fb4`), `277A-7` (migración SQL pagos/reembolsos) y `277A-8`/`9`/`10` (fixes SEO + banner 50%).
  No ejecutar deploy sin autorización.
- **Correo para Hosting** — bloqueado en elegir proveedor: MXroute ($59/año, sin API) vs Migadu
  ($9/mes, API REST). Ver `Agente/documentacion/hosting/producto-correo-proveedores-2026-05-26.md`.
  Afecta a `265A-11` (Fase 1: aliases/reenvíos con Cloudflare Email Routing, ~8-10 h), `265A-12`
  (Fase 2: buzones IMAP, ~20-26 h) y `265A-13` (1 buzón IMAP gratis en plan Avanzado, ~3-4 h).
- **Excepción `cargo-audit` RUSTSEC-2023-0071 (`rsa`)** — `rsa` solo entra en `Cargo.lock` vía
  `sqlx-mysql` (no compilado). Vence 2026-11-13; ver `.cargo/audit.toml`. Quitar si hay parche o
  `sqlx-mysql` sale del lock.

## Planes activos

- `Agente/planes/plan-barrido-consola-07AA-2026-10-07.md` — `07AA` — barrido de consola (73 → 0 accionables); cerrado.
- `Agente/planes/plan-cierre-bloques-dificiles-nakomi-2026-07-23.md` — bloques A-E implementados; pendientes canary de recepción, prueba visual del CTA y gateway WhatsApp automático.
- `Agente/planes/plan-cierre-whatsapp-continuacion-contacto-2026-07-26.md` — `267A-3` — WhatsApp cerrado; canary de continuación pendiente.
- `Agente/planes/plan-cms-api-agente-2026-05-15.md` — `035A-34` — API CMS privada; gap real: tabla `site_config` + endpoints.
- `Agente/planes/plan-deploy-produccion-2026-07-27.md` — `277A` — deploy del bloque 277A a producción; pendiente de autorización.
- `Agente/planes/plan-dominios-2026-04-07.md` — gestión de dominios (~80%); faltan auto-renew Stripe, recordatorios de caducidad y UI de DNS.
- `Agente/planes/plan-estabilidad-nakomi-2026-07-23.md` — `237A-3` — estabilidad integral (freeze, precedencia humano–IA); B-E desplegadas, F-K pendientes.
- `Agente/planes/plan-fixes-pendientes-2026-06-01.md` — correcciones urgentes y refactor de emails; no cerrado hasta la aceptación funcional.
- `Agente/planes/plan-fixes-permanentes-studio-2026-05-23.md` — fixes permanentes (build Rust, claves SSH, compose de Coolify).
- `Agente/planes/plan-galeria-og-seo-2026-07-27.md` — galería de imágenes SEO + recorte + JSON-LD editable.
- `Agente/planes/plan-gui-coolify-manager-2026-05-10.md` — `105A-9`/`105A-10` — rediseño de la GUI y modo navegador sin Tauri.
- `Agente/planes/plan-hosting-automation-2026-04-10.md` — hosting automation (~92%); falta política de escalada por impago prolongado.
- `Agente/planes/plan-hosting-lightweight-sin-coolify-2026-05-24.md` — hosting administrado ligero sin Coolify (plan maestro).
- `Agente/planes/plan-implementacion-seo-nakomi-2026-07-27.md` — `277A` — implementación SEO; bloque 277A hecho, pendiente aprobación.
- `Agente/planes/plan-incidente-freeze-bad-gateway-nakomi-2026-07-24.md` — causa raíz corregida; fase 4 (rediseño del watchdog) pendiente.
- `Agente/planes/plan-latencia-hostings-vps2-2026-05-23.md` — latencia en hostings VPS2; el WordPress de VPS2 sigue lento.
- `Agente/planes/plan-mejoras-coolify-manager-incidencias-2026-07-25.md` — comandos de incidentes en `coolify-manager-rs`; README/skill sin actualizar.
- `Agente/planes/plan-migracion-consumidores-glory-rs-main-2026-07-25.md` — migrar consumidores de Glory RS a `main`; bloqueado hasta estabilizar.
- `Agente/planes/plan-recursos-hosting-2026-05-22.md` — enforcement de recursos (~85%); faltan corte por impago y leer `usage_alert_threshold_pct`.
- `Agente/planes/plan-restauracion-guillermo-2026-07-22.md` — restauración Guillermo + errores 500; pendiente de aprobación.
- `Agente/planes/plan-seguridad-hosting-2026-04-16.md` — seguridad del hosting; backlog residual (SSH access logging, fail2ban, MaxAuthTries, SSH key auth).
- `Agente/planes/plan-sentinel-problemas-reales-2026-04-12.md` — problemas reales del Sentinel report (~40-50% ejecutado).
- `Agente/planes/plan-sentinel-varsense-editor-agnostico-2026-05-08.md` — Sentinel+VarSense agnóstico (~85%); faltan LSP, CI remoto y `FileWatcherProvider`.
- `Agente/planes/plan-seo-dashboard-blog-descuento-2026-07-25.md` — dashboard SEO, blog y descuento; planificado, pendiente de aprobación.
- `Agente/planes/plan-seo-editable-panel-2026-07-27.md` — SEO editable en el panel.
- `Agente/planes/plan-ssh-guard-2026-06-03.md` — SSH Guard: prohibir técnicamente los comandos SSH directos.
- `Agente/planes/plan-ssh-sftp-seguro-2026-04-16.md` — SSH/SFTP seguro del servicio de hosting.
- `Agente/planes/plan-vps-nakomi-studio-2026-05-10.md` — publicar `vps.nakomi.studio` (GUI online con login).
