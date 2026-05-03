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

- Completar el barrido de bordes neutrales: `--border-default` ya es `#dcdcdc`, pero todavia quedan componentes usando `border: 1px solid var(--bg-item-active)` como borde generico cuando no representan un estado activo/seleccionado.
- Estoy revisando un proyecto activo de esos que se generan para pruebas, veo el apartado de fases pero en modo empleado pero no veo que se puedan modificar las fases lo cual esta mal, tampoco se como se generan las fases porque ajam, ahora los servicios son dimaicos y se generan a través del cms, si el cliente elige pagar por fases creo que lo mas logico es que las fases esten prehechas para cada servicio a través del cms, un servicio necesita obligatoriamente fases en caso de que el cliente vaya a pagar por fase. 
- Simplificar `ModalCompra`: `modalCompraResumen` sobra y `Describe tu proyecto` deberia ocurrir despues de la compra, no dentro del modal de checkout inicial.
- Investigar y corregir el `POST /api/orders` local que falla con `403 (Forbidden)` al contratar un servicio.
- si estoy dentro un proyecto (en el panel) debería generar una url para poder compartir en el panel y recargar estar dentro de ese proyecto (obviamente ), lo mismo con cualquier otra cosa si estoy dentro de los detalles de un hosting, etc.
- En la vista de un proyecto dentro de un panel en el usuario de cliente veo que puedo cambiar la descripción de un proyecto, los cliente no deben cambiar la descripciones de sus proyectos.
- Veo que al cliente llega la notificación de que recibio un mensaje dentro del un proyecto que tiene activo pero al dar click no hace nada, debería redirigirlo al proyecto
- En los proyectos los empleados tienen un boton de IA para activar, esto hace que la IA responda por ellos cuando el cliente escribe pero no funciona, hice la prueba (con los datos de prueba locales), y no recibi ninguna respuesta enviando mensaje desde el modo cliente. 
- Necesito datos de prueba para, necesito ver como se ve. 
"Historial de movimientos
Sin movimientos aún
Solicitudes de retiro
No has solicitado retiros aún"
- esto que pusiste de "**Refuerzo operativo obligatorio para modales y paneles:**
- Reutiliza siempre las clases semanticas compartidas del sistema antes de crear una local. Para modales: `.modalTitulo`, `.modalTexto`, `.modalAcciones`.
- Queda prohibido crear `.algoModalTitulo`, `.algoModalTexto`, `.algoModalDescripcion`, `.algoModalAcciones` si solo redefinen tipografia, color, spacing o alineacion ya resueltos por el sistema.
- Si falta una receta compartida, se crea primero en el componente base del sistema y luego se consume desde el componente concreto." No parece que sea generico, no es algo especifico que debe hacerse con solo con los modales tiene que ser con todo los componentes el refeurzo.
- Veo muchas cosas en planes que no estan en completados, hay que revisarlos todo 


## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **[164A-19] Panel admin de infraestructura**: `Despliegues VPS2` ahora lista servicios reales desde Coolify y `Contabo VPS` queda separado para mostrar solo la capa proveedor.
- **[164A-20] Carrusel del inicio**: las imágenes del showcase ahora salen por `/api/img/...?...w=1200&q=80` con ancho fijo de optimización, sin escalar a buckets mayores por DPR.
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`
- **[164A-18] Fix aplicado**: Dockerfile.rust actualizado con `gosu` entrypoint para corregir permisos de volúmenes montados (`root:root` → `appuser`). Fix inmediato aplicado en producción via chown directo. Detalle en `Agente/completados/tareas-2026-04-16.md`.