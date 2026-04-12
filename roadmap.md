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

- Corregir ~126 problemas reales del sentinel-report → ver plan detallado en `Agente/planes/plan-sentinel-problemas-reales-2026-04-12.md`
- El input orden dentro dentro de los contenido de cms es innecesario. 
- Los proyectos elegir si aparecen en el carrusel o si aparecen en Selected Work es confuso, tiene que ser interno en el modal, puede aparecer en ambos o en ninguno y por defecto todo tiene que aparecer. Igualmente un blog no tiene opcion aparente para mostrarse dentro de la pagina de inicio o no, tiene que interno, con el icono de estrella.
- Las env del chatbot ya están en producción. El fix de IA (asign_staff + toggle panel) requiere un deploy (commit 465f2eb).
- Cuando la IA hace una escalación: notificación push + correo. En el panel: verificar/mostrar indicador y toggle de IA apagado cuando hay escalación activa.
- Hay que asegurar que una vez que el chatbot le envía la factura, detectar cuando la factura se pague, notificar al admin, y avisarle al cliente que se registre con el correo con el que hizo el pago.
- En chat del panel en info de visitante falta que diga el país.
- Las imágenes de proyectos tienen problema: la misma imagen carga 2 veces (carrusel = carruselImagen + Selected Work = proyectoImagen con srcset diferente). La resolución del carrusel es mala y el peso es excesivo. Hay que unificar el pipeline de imágenes y optimizar srcset/sizes para LCP.



## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`