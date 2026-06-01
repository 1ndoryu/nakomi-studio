# Plan: Correcciones urgentes y refactor de emails

> **Fecha:** 2026-06-01
> **Rama:** glory-rust-nakomi
> **Estado:** Pendiente de aprobación

---

## Resumen

7 problemas reportados por el usuario, agrupados en bloques coherentes. Cada bloque = un commit separado.

---

## Bloque A — Auto-asignación al administrador

**ID:** `016A-4`
**Estimación:** ~1-2h
**Archivos a modificar:** `src/handlers/orders.rs`, `src/services/order.rs`, `src/repositories/order.rs`
**Dependencias:** Ninguna

### Qué hacer
Cuando se crea una orden (`create_order`), asignarla automáticamente al usuario administrador (el que reportó el problema) sin límite de órdenes concurrentes.

### Implementación
1. En `src/handlers/orders.rs` → `create_order()`, después de crear la orden:
   - Obtener el ID del admin via `UserRepository::admin_ids()`
   - Llamar `OrderRepository::assign_order()` con el admin ID
2. En `src/services/assignment.rs` → `take_order()`: saltar verificación de `max_concurrent_orders` si el empleado es admin
3. Asegurar que `open_to_employees` se establezca en `false` tras asignación

### Gotchas
- El admin puede tener pedidos del cliente directo (marketplace) + pedidos propios
- No debe interferir con el flujo actual de empleados tomando órdenes de "Disponibles"
- La notificación al admin debe seguir funcionando

---

## Bloque B — Correcciones de UI/UX en asignación

**ID:** `016A-3` (B1), `016A-2` (B3), `016A-5` (B2)
**Estimación:** ~2-3h
**Archivos a modificar:** Múltiples (ver abajo)
**Dependencias:** Bloque A (puede ser paralelo)

### B1: Modal de asignar empleado se ve mal
**Qué:** El modal `ModalAsignar` no se ve como los otros modales del sistema.
**Archivos:** `frontend/src/components/panel/ModalAsignar.tsx`, `frontend/src/components/panel/ModalAsignar.css`
**Implementación:**
- Investigar qué modales se usan como referencia (ej: `ModalConfirmar`, `ModalPago`)
- Adaptar `ModalAsignar` para usar el mismo patrón visual (mismas variables CSS, misma estructura layout)
- Verificar que use `MenuContextual` o el sistema de modales compartido

### B2: No se puede cancelar orden o cambiar empleado tras asignación
**Qué:** Actualmente no hay endpoint para que admin desasigne/reasigne una orden.
**Archivos:** `src/handlers/assignment.rs`, `src/repositories/order.rs`, `frontend/src/components/panel/DetalleOrden.tsx`
**Implementación:**
- Backend: `POST /api/orders/{order_id}/unassign` (admin only) — quita `assigned_employee_id`, vuelve a `AwaitingAssignment`
- Backend: `PUT /api/orders/{order_id}/reassign/{employee_id}` (admin only) — cambia el empleado asignado
- Frontend: Botón "Desasignar" / "Cambiar empleado" en detalle de orden (solo visible para admin)

### B3: "Empleado asignado" → "Freelancer asignado"
**Qué:** Cambio de texto en frontend.
**Archivos:** Buscar en `frontend/src/` por el string "Empleado asignado" y reemplazar.
**Implementación:** grep + replace simple.

---

## Bloque C — Chat en tiempo real + notificaciones

**ID:** `016A-6`
**Estimación:** ~5-8h (el más complejo)
**Archivos a modificar:** Múltiples (ver abajo)
**Dependencias:** Ninguna

### C1: Notificaciones en tiempo real cuando el cliente envía mensaje en orden
**Qué:** Cuando un cliente envía un mensaje en el chat de una orden, el admin no recibe notificación push.
**Archivos:** `src/handlers/chat/rest_messages.rs`, `src/services/chat.rs`, `src/handlers/notifications.rs`
**Implementación:**
- En `send_message` endpoint o en `ChatHub::send_message`: después de guardar el mensaje, crear notificación `new_message` para el `assigned_employee_id` de la orden
- La notificación debe incluir `link` al panel de chat con la sesión
- Asegurar que el WebSocket de notificaciones (`/ws/notifications`) emita el evento

### C2: Email automático si mensaje pasa 20 min sin respuesta
**Qué:** Enviar email al admin si un mensaje del cliente lleva 20+ minutos sin respuesta del staff.
**Archivos:** `src/services/email.rs` (nueva función `send_unanswered_chat_admin`), `src/services/chat.rs` (background loop)
**Implementación:**
- Nuevo método `EmailService::send_chat_unanswered_admin()` con plantilla HTML
- Background loop en `ChatHub` o servicio separado: cada 1-2 min revisa mensajes no respondidos con `created_at > 20 min` y sin respuesta del staff
- Enviar email al admin y marcar como "notificado" para no re-enviar
- Log en `email_logs`
- **Nota:** Esta función debe crearse en el nuevo sistema centralizado (Bloque E) si se implementa primero

### C3: Punto rojo en sidebar para mensajes no leídos
**Qué:** Mostrar indicador visual (badge/punto rojo) en el botón "Mensajes" del sidebar cuando hay sesiones de chat con mensajes no leídos.
**Archivos:** `frontend/src/components/panel/SidebarPanel.tsx`, `frontend/src/hooks/useSeccionChat.ts`, `frontend/src/stores/chatStore.ts`
**Implementación:**
- Calcular total de mensajes no leídos de sessions activas (staff) | O usar endpoint `GET /api/notifications/unread-count` con filtro de chat
- Mostrar badge rojo en el botón del sidebar si count > 0
- El badge debe actualizarse en tiempo real vía WebSocket

---

## Bloque D — Bug visual en notificaciones

**ID:** `016A-1`
**Estimación:** ~0.5h
**Archivos a modificar:** `frontend/src/components/panel/NotificationBell.css`
**Dependencias:** Ninguna

### Qué hacer
El texto de las notificaciones en el dropdown aparece centrado cuando no debería.

### Diagnóstico preliminar
- `.notificationBell__item` usa `align-items: flex-start` que es correcto
- `.notificationBell__itemContent` no tiene `text-align`
- Posible causa: `.notificationBell__item` tiene `justify-content` o el botón padre hereda `text-align: center`
- Revisar también `.notificationBell__empty` que tiene `text-align: center` (correcto para estado vacío)

### Implementación
1. Revisar el CSS del item y asegurar `text-align: left`
2. Verificar que el botón padre no herede alineación centrada
3. Testear visualmente

---

## Bloque E — Refactor de plantillas de email (centralizar)

**ID:** `016A-7`
**Estimación:** ~6-10h (el más grande en líneas de código)
**Archivos a modificar:** `src/services/email.rs`, `src/services/email_preview.rs`, **NUEVO** `src/services/email_templates.rs`
**Dependencias:** Ninguna (pero ideal después de los demás para no mezclar)

### Problema actual
Cada plantilla de email existe en dos lugares:
1. `email.rs`: función `send_*()` con el HTML inline + lógica SMTP
2. `email_preview.rs`: función `render_*()` con el MISMO HTML copiado + datos de ejemplo

Esto duplica ~12,000 líneas de HTML y cada cambio requiere editar ambos archivos.

### Solución propuesta
Crear `src/services/email_templates.rs` como el ÚNICO lugar donde vive el HTML:

```rust
// email_templates.rs — SOLO plantillas HTML, sin lógica de envío
pub struct TemplateData {
    pub to_email: String,
    pub client_name: String,
    pub order_number: i32,
    // ... campos específicos por plantilla
}

impl EmailTemplates {
    pub fn order_confirmation(data: &OrderData) -> String { ... }
    pub fn new_order_admin(data: &NewOrderData) -> String { ... }
    // etc.
}
```

Luego:
- `email.rs` importa las templates de `email_templates.rs`, les pasa datos reales y las envía
- `email_preview.rs` importa las mismas templates con datos de ejemplo

### Archivos resultantes
| Archivo | Responsabilidad |
|---|---|
| `src/services/email_templates.rs` | **NUEVO** — Solo HTML de plantillas (renderizado puro) |
| `src/services/email.rs` | Solo lógica SMTP + logging + orquestación |
| `src/services/email_preview.rs` | Solo routing de preview + sample data |

### Migración
1. Crear `email_templates.rs` con todas las funciones de renderizado
2. `email_preview.rs` refactorizado para llamar a `email_templates` con sample data
3. `email.rs` refactorizado para llamar a `email_templates` con datos reales
4. Eliminar HTML duplicado de `email.rs` y `email_preview.rs`
5. Verificar que cada template preview funcione igual que antes

---

## Pendiente pre-existente: 5 nuevas plantillas admin

**Estado:** ✅ Implementado, compilado, pendiente de commit
**IDs:** Las 5 funciones `send_*_admin` + `render_*` ya existen en `email.rs` y `email_preview.rs`
**Riesgo:** Si se hace el Bloque E primero, estas funciones pueden integrarse directamente en el nuevo sistema

---

## Orden sugerido

```
Bloque D (½h) → Bloque B3 (¼h) → Bloque B1 (1-2h) → Bloque A (1-2h)
→ Bloque B2 (1h) → Bloque C (5-8h) → Commit parcial de A+B+C+D
→ Bloque E (6-10h) → Commit final
```

O, alternativamente, si el usuario prefiere:

```
Primero Bloque E (refactor) → luego todo lo demás con las templates ya centralizadas
```

---

## Preguntas al usuario

1. **Orden:** ¿Prefieres que haga el refactor de plantillas (Bloque E) primero antes de las nuevas funcionalidades, o al final?
2. ✅ **Auto-asignación:** Todas las órdenes nuevas, con opción de delegar/cancelar.
3. ✅ **Chat 20 min:** Email a cliente + admin + empleado asignado.
4. ✅ **Modal referencia:** `ModalCrearUsuario` (usa `ModalBody`/`ModalField`/`ModalLabel`).
