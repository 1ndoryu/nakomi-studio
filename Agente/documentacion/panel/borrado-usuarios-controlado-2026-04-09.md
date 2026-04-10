# Borrado controlado de usuarios en panel admin

Fecha: 2026-04-09

## Objetivo

Permitir que el admin elimine usuarios desde la tabla del panel sin introducir fallos referenciales en la base de datos.

## Decisión

El endpoint no hace un `DELETE FROM users` ciego. Antes calcula bloqueos reales de negocio y solo elimina cuando el usuario no tiene relaciones activas en:

- pedidos como cliente
- pedidos asignados como empleado
- reviews
- reembolsos
- delegaciones
- entregables subidos
- hosting
- sesiones de chat
- artículos de blog

Si alguna de esas relaciones existe, la API responde `400 bad_request` con un mensaje legible para el panel. La recomendación operativa pasa a ser suspender al usuario o limpiar primero esas relaciones.

## Frontend

- `SeccionUsuarios` añade la acción `Eliminar usuario` en el menú contextual de cada fila.
- No se muestra para el usuario autenticado actual.
- La confirmación usa un modal y deja visible el mensaje exacto del backend si el borrado es bloqueado.
- La lógica de la sección se extrajo a `useUsersSection` para no romper los límites de tamaño/SRP del componente.

## Backend

- Nuevo endpoint: `DELETE /api/admin/users/{user_id}`.
- Reglas:
  - solo admin
  - no permite auto-borrado
  - exige que el usuario exista
  - hace preflight de dependencias antes del delete
- Si pasa el preflight, ejecuta hard delete y registra `user_delete` en audit log.

## Validación aplicada

- `cargo sqlx prepare`
- `cargo check`
- `cargo clippy -- -D warnings`
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`