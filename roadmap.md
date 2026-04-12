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

- EN EL CMS DONDE CARAJO SE MODIFICA EL showcaseTituloCategoria!!!  O SEA NO SE PUEDE NECESITA ESPECIFICARSE EN CADA PROYECTO Y SI TIENEN El MISMOS PUES SE AGRUPAAAN Y YA!
- Hace falta un buscador en cada tab del cms, que funcione en tiempo real.
- Los blog tambien deben poder elegirse cual aparece en el inicio, igual que los servicios, proyectos destacados y cuales proyectos aparecen el carrusel de inicio.
- Tambien debería poder elegirse entre la imagen de portada externa y la primera imagen dentro de la pagina individual de proyecto
- Las imagenes de la galería dentro de los proyectos no se estan ordenando como te dije 1/1 una y luego 2 1/2, y asi.
- proyectoCaseIntroDescripcion maxmo debería ocupar 1/2 de la pantalla.
- El boton de eliminar uploadImagenPreview se ve mal, no se ve el icono

### Delegaciones y pedidos (diseño completo pendiente)

Flujo esperado tipo Fiverr:
1. Cliente crea pedido → visible solo para admin por 48 horas.
2. Si admin no delega en 48h → empleados ven notificación y pueden tomarlo.
3. Cancelaciones: empleado envía solicitud → cliente acepta (dinero a wallet) o rechaza.
4. Wallet: dinero de clientes y empleados, posibilidad de retiro.

### Hosting / Infraestructura

- Hosting/SSH seguro: Pendiente solo Fase 1 (verificación VFS disco en VPS2).
- Seguridad hosting: Pendiente Fase 4.1 DNS ownership, Fase 5 monitoreo (depriorizados).
- Segunda auditoría profunda de seguridad al sistema de hosting.
- Contabo rechazó autenticación. Revisar CONTABO_API_PASSWORD y credenciales OAuth2.
- Hosting Automation Fase 4: Dominios y DNS — falta registrar API, gestión DNS via Contabo, auto-SSL.
- COOLIFY_PROJECT_UUID pendiente de actualizar en prod (env vars del servicio Coolify de nakomi.studio).

### Planes activos

> Chatbot v2 (Fases I y II completadas, testing completado), SEO (Fases 2-3 completadas), Seed system (Fase 5 completada)
> Status hosting: `Agente/documentacion/hosting/status-hosting-administrado-2026-04-07.md`

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`