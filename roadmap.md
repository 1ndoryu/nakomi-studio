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

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes

- 155A-11 — Crear en producción un usuario de pruebas `test@test.com` tipo cliente con la contraseña indicada por el usuario en conversación (no versionarla en roadmap) y con una particularidad: no debe pagar nada al comprar servicios, hosting o VPS, para poder probar con exactitud qué ocurre en los tres flujos de compra.
- 155A-12 — Preparar el agente para atender solicitudes relacionadas con VPS: debe poder ofrecer el servicio, cobrarlo y responder consultas relacionadas a VPS; revisar lo que ya existe para hosting y extenderlo a VPS.
- 155A-13 — Separar “hosting WordPress” y “hosting normal”: crear y ejecutar un plan técnico que cubra Coolify/provisioning, catálogo, precios y panel; el cliente debe poder elegir entre hosting normal y hosting WordPress; el hosting normal debe ser 30% más caro que la referencia actual; el panel ya no debe referirse a todo como hosting WordPress sino administrar ambos tipos de hosting.
