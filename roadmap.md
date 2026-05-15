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

- 155A-3 — Diagnosticar y corregir los errores actuales de compilación Rust antes de continuar con el resto del bloque.
- 155A-4 — Actualizar `menuContextual` y `menuContextualApps`: donde dice “task” debe decir “catask”; reemplazar el logo por `catask.svg`, convirtiendo el SVG de blanco a negro; si los SVG no aparecen en el repo, buscarlos en Descargas; en el menú contextual de apps reemplazar el icono por `apps.svg`.
- 155A-5 — Hacer que el botón “Comenzar proyecto” abra el chat; ahora no hace nada.
- 155A-6 — Quitar la página accesible de “Soluciones”: no debe poder navegarse a `/soluciones`, pero sí deben mantenerse accesibles sus páginas subyacentes de hosting y VPS.
- 155A-7 — En servicios, mostrar un menú contextual con el nombre de los servicios para navegar directamente a cada servicio.
- 155A-8 — Cambiar el grid de proyectos en `/servicios` a 2 columnas.
- 155A-9 — Agregar imágenes tipo `galeriaHeroContenedor`, con el mismo estilo pero sin comportamiento de galería, en las páginas de VPS y Hosting; si el usuario es admin debe aparecer un botón de tres puntos en la esquina como en servicios para cambiar la imagen; la imagen debe optimizarse igual que `galeriaHeroContenedor`; mientras tanto usar una imagen de prueba.
- 155A-10 — Corregir el botón de volver dentro del detalle de hosting (`/panel?seccion=hosting&hostingId=...`): actualmente no funciona y al abrir un hosting el detalle se abre y se cierra rápidamente.
- 155A-11 — Crear en producción un usuario de pruebas `test@test.com` tipo cliente con la contraseña indicada por el usuario en conversación (no versionarla en roadmap) y con una particularidad: no debe pagar nada al comprar servicios, hosting o VPS, para poder probar con exactitud qué ocurre en los tres flujos de compra.
- 155A-12 — Preparar el agente para atender solicitudes relacionadas con VPS: debe poder ofrecer el servicio, cobrarlo y responder consultas relacionadas a VPS; revisar lo que ya existe para hosting y extenderlo a VPS.
- 155A-13 — Separar “hosting WordPress” y “hosting normal”: crear y ejecutar un plan técnico que cubra Coolify/provisioning, catálogo, precios y panel; el cliente debe poder elegir entre hosting normal y hosting WordPress; el hosting normal debe ser 30% más caro que la referencia actual; el panel ya no debe referirse a todo como hosting WordPress sino administrar ambos tipos de hosting.
- 105A-29 — Mover el selector global de VPS a la zona `logoSidebar` y usar un selector personalizado, no `<select>` nativo.
- 105A-30 — Agregar regla en Glory Sentinel para detectar `<select>` nativos en React/TSX y recomendar el componente personalizado de Nakomi.
- 105A-31 — Convertir `Agregar sitio` en modal funcional con validación, feedback visible y verificación real.
- 105A-32 — Retirar `rutaPagina` de la GUI porque no aporta valor operativo y limpiar la jerarquía visual.
- 105A-33 — Mostrar favicons online de los sitios en la tabla, con fallback seguro y sin ralentizar el listado.
- 105A-34 — Completar publicación de `vps.nakomi.studio` vía Coolify/coolify-manager-rs con dominio extra, health y verificación del subdominio.
- 105A-35 — Implementar login/admin seguro para el portal VPS sin hardcodear credenciales.
- 105A-36 — Endurecer seguridad operativa online: RBAC, rate limits, auditoría y límites de exposición de logs/secrets.
- 105A-37 — Planificar y migrar progresivamente compra/gestión de VPS desde Nakomi principal hacia `vps.nakomi.studio`, reutilizando Stripe/Contabo/Coolify existentes.
- Respecto a vps.nakomi.studio, la publicación y el acceso a controlar la API de Coolify deben estar protegidos solo para el usuario admin. Tiene que ser seguro, con página de inicio, login modal y opción de cerrar sesión en el panel.