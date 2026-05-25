Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi

## Stack

| Capa          | Herramienta                    |
| ------------- | ------------------------------ |
| Framework web | Axum 0.7                       |
| OpenAPI       | utoipa 4 + utoipa-swagger-ui 7 |
| Base de datos | SQLx 0.8 (PostgreSQL)          |
| Validación    | validator 0.18                 |
| Auth          | jsonwebtoken + argon2          |
| Frontend      | React 18 + TypeScript + Vite   |
| State         | React Query + Zustand          |
| Codegen       | Orval 8                        |
| Deploy        | coolify-manager-rs             |

# Nakomi Studio — Roadmap

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

## Estado interno reciente

- `245A-9`: el runtime de hosting `lightweight` ya expone backup/restore remoto por manager y por API de suscripciones (`/api/hosting/subscriptions/{id}/backups`, `/api/hosting/subscriptions/{id}/restore`). Pendiente siguiente del frente: smoke operativo real del restore, observabilidad/panel lightweight y receta WordPress premium.
- `245A-10`: la compra de hosting ya deja fijado el runtime por plan en vez de depender del provider global; `normal-*` solo cae en `lightweight` cuando el target está configurado, WordPress sigue en Coolify, y en producción se corrigió `COOLIFY_BASE_URL` al alias interno de Coolify con el bypass de checkout test desactivado antes de la compra real.
- `255A-2`: `/api/hosting/deployments` ya no interpreta un fallo de Coolify/runtime como inventario vacío; reconstruye un fallback mínimo desde `hosting_subscriptions` y solo devuelve `503` si ni siquiera puede recomponer una lista útil. En el diagnóstico de VPS2 también se confirmó que un `500` global de Coolify 4.1.0 podía venir de `personal_access_tokens.abilities='[*]'` en vez de JSON válido.
- `255A-3`: los governors de auth y API se subieron a límites productivos con `SmartIpKeyExtractor` para evitar `429` cruzados detrás de Coolify/Traefik cuando la SPA abre polling y varias requests concurrentes.

---

## Tareas pendientes

- **065A-4 — Resolver bloqueo BDP `[300035]` fuera de horario (sin escrituras reales).**
  - Estado actual: `sync-dry-run` ya valida lecturas reales; `CreateOrder` se ejecuta en `OnlyCheck` (`OrderOperationType=1`) con `escritura_real=false` y sin `Payments`.
  - Bloqueo vigente: BDP devuelve `[300035]-NO SE HA DEFINIDO UNA SERIE DE FACTURACION VALIDA`.
  - Nota clave: no es un bug del backend Rust; es configuracion interna de BDP-Net (serie/destino para pedidos WebLink).
  - Pendiente operativo para el usuario (cuando el restaurante no este en horas de trabajo):
    1. Conectarse por RDP a `100.83.196.35`.
    2. Abrir BDP-Net y entrar a `Utilidades -> Configuracion Servicios Web`.
    3. Localizar el ajuste de serie de destino/serie para pedidos externos WebLink (o nombre equivalente).
    4. Guardar evidencia de valores actuales antes de cambiar (captura/foto).
    5. Asignar una serie valida de facturacion simplificada para pedidos WebLink (segun configuracion del local).
    6. Guardar cambios y volver a la app.
    7. Ejecutar `Probar sincronizacion segura` en produccion.
  - Criterio de cierre: `listo_para_sincronizar=true` manteniendo `escritura_real=false`.
  - Restriccion: no crear/modificar ventas, comandas, clientes, articulos ni pagos reales en el restaurante.

