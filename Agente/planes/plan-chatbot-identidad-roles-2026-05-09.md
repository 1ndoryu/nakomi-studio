# Plan de revisión — Chatbot identidad, roles y capacidades

**Fecha:** 2026-05-09
**Tarea origen:** 095A-17
**Estado:** Activo

## Objetivo

Auditar de forma extensa cómo el chatbot reconoce visitantes, usuarios logueados, clientes, empleados y administradores, y cerrar los huecos para que pueda asistir con pedidos, pagos, hosting, reportes y consultas según permisos reales.

## Diagnóstico inicial

- El widget envía JWT por WebSocket cuando el frontend tiene sesión iniciada.
- El backend detecta `user_id` desde el token y lo pasa al prompt de IA.
- Antes de 095A-16, la sesión `chat_sessions` y el `visitor_profile` no quedaban vinculados de forma persistente al usuario autenticado.
- El cierre normal del WebSocket cerraba la sesión al recargar, lo que rompía recuperación de historial y continuidad.
- El prompt base seguía dando mucha prioridad a capturar nombre/email; para usuarios registrados debe prevalecer el contexto de cuenta.

## Fase 1 — Continuidad básica

- Mantener sesión activa al recargar o cerrar pestaña.
- Recuperar historial visual al abrir el widget.
- Asegurar que `/reset` limpia visualmente, localmente y en backend.
- Confirmar que producción usa DeepSeek como primario.

## Fase 2 — Identidad autenticada

- Verificar que el JWT llega siempre en `buildVisitorWsUrl` cuando `authStore` está inicializado.
- Vincular `chat_sessions.user_id` al usuario autenticado.
- Vincular `visitor_profiles.user_id`, `email` y `display_name` al usuario autenticado.
- Revisar comportamiento al hacer logout/login con otro usuario en el mismo navegador.
- Definir si el `visitor_id` debe rotar al cambiar de cuenta para evitar mezcla de conversaciones.

## Fase 3 — Roles y permisos

- Validar diferencia entre `role` real y `effective_role` en admins con impersonación.
- Definir qué puede consultar el chatbot para `admin`, `employee` y `client`.
- Evitar que un cliente consulte datos ajenos aunque intente prompt injection.
- Asegurar que acciones administrativas requieran tools con autorización backend, no solo prompt.

## Fase 4 — Capacidades por dominio

- Pedidos: listar pedidos propios, estado, fases, entregables, empleado asignado y reportes abiertos.
- Pagos: listar facturas/pagos propios, estados, links de pago vigentes y reintentos seguros.
- Hosting: listar hostings propios, dominios, estado, plan, eventos y reportes.
- Reportes: crear y consultar reportes del usuario, escalar a humano cuando corresponda.
- Admin: consultar panel operativo con métricas y entidades globales solo mediante tools protegidas.

## Fase 5 — Contratos de tools

- Inventariar tools existentes y su boundary de permisos.
- Agregar tools faltantes con validación por `user_id` y rol efectivo.
- Estandarizar respuestas `requires_login`, `forbidden`, `not_found` y `ok`.
- Añadir tests unitarios para permisos por rol.

## Fase 6 — UX y observabilidad

- Mostrar continuidad de conversación sin parpadeo vacío.
- Añadir logs seguros: usuario detectado, rol, sesión, tool llamada, resultado sin secretos.
- Añadir métricas de fallback AI y errores de tools.
- Documentar escenarios de prueba manual: visitante anónimo, cliente, admin, admin impersonando cliente.

## Cierre esperado

- El bot reconoce a un usuario logueado por nombre y rol.
- Un admin no recibe preguntas de lead anónimo.
- Un cliente puede preguntar por sus pedidos, pagos y hosting sin repetir identidad.
- `/reset` es el único flujo que borra identidad local y conversación del visitante.
- Cada capacidad sensible queda respaldada por tools autorizadas y tests.