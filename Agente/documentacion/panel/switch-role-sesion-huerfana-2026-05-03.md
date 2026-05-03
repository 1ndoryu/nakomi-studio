# Switch-role y Sesiones Huérfanas — 2026-05-03

## Problema

`POST /api/auth/switch-role` devolvía `500` cuando el navegador conservaba un JWT de impersonación cuyo `impersonator` apuntaba a un admin ya inexistente tras reseed o limpieza local. El ciclo fresco `admin -> employee -> client -> admin` seguía funcionando, por eso la falla solo aparecía en sesiones persistidas del navegador.

## Cambio aplicado

- `src/handlers/order_lifecycle.rs` valida primero que el admin original siga existiendo y conserve rol `admin`.
- Si esa sesión quedó huérfana, el handler responde `403 forbidden` con mensaje explícito: `La sesión de impersonación ya no es válida. Inicia sesión de nuevo.`
- `frontend/src/components/panel/SidebarPanel.tsx` usa `extraerMensajeError()` y `toast.error()`; cuando detecta `401` o este `403`, ejecuta `logout()` para limpiar el estado persistido y evitar el bucle de errores.

## Validación

- Se reprodujo el fallo con un JWT stale fabricado usando el `JWT_SECRET` local.
- En el backend viejo, la llamada devolvía `500 internal_error`.
- Con la corrección cargada en una segunda instancia local (`127.0.0.1:3001`), el mismo token devolvió `403 forbidden` con el mensaje esperado.
- El ciclo normal `admin -> employee -> client -> admin` siguió respondiendo correctamente.

## Nota operativa

Si `:3000` sigue levantado con un proceso viejo, la validación de este fix puede requerir reiniciar el backend principal o levantar una segunda instancia temporal para comprobar el comportamiento nuevo sin perder la sesión en curso.