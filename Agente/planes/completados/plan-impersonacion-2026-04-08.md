# Plan: 084A-1 — Switch de vista con impersonación real

> **Fecha:** 2026-04-08
> **Objetivo:** Cuando admin cambie de vista (switch-role), debe "impersonar" un usuario real (cliente o empleado), viendo sus datos genuinos.

## Problema actual
- `switch_role` solo cambia `active_role` en la BD del mismo admin
- `user_id` nunca cambia → órdenes siempre muestran todas (admin override 064A-58)
- Chat mensajes todos en el mismo lado (sender_id = admin siempre)
- El admin no puede probar la funcionalidad real

## Solución: Impersonación en JWT

### Backend (6 archivos)

**1. `src/services/auth.rs` — Claims + generate_token**
- Agregar `impersonator: Option<Uuid>` a Claims
- Modificar `generate_token` para aceptar `impersonator` param
- Todos los callers existentes (login, register, quick_register) pasan `None`

**2. `src/middleware/auth.rs` — AuthUser**
- Agregar `impersonator: Option<Uuid>` a AuthUser
- Parsear desde claims

**3. `src/models/user.rs` — AuthResponse**
- Agregar `impersonating: bool` a AuthResponse
- Todos los callers existentes pasan `false`

**4. `src/repositories/user.rs` — find_first_by_role**
- Nueva query: `SELECT ... FROM users WHERE role = $1 AND status = 'active' LIMIT 1`

**5. `src/handlers/orders.rs` — switch_role handler**
- Si target es client/employee: buscar usuario real con ese rol, generar token con su ID + impersonator=admin_id
- Si target es admin: restaurar token del admin original (usar impersonator del JWT actual)
- Security: verificar que el caller es admin real o está impersonando (impersonator is Some)

**6. `src/handlers/orders.rs` — list_orders**
- Remover el override 064A-58 (admin siempre ve todo)
- Usar effective_role directamente: si admin con effective_role=admin → todo. Si impersonando como client → filtrar por user_id

### Frontend (4 archivos)

**7. `frontend/src/api/auth.ts` — AuthResponse type**
- Agregar `impersonating: boolean`

**8. `frontend/src/stores/authStore.ts`**
- Agregar `impersonating: boolean` a AuthUser
- `login()` y `actualizarRol()` manejan el campo

**9. `frontend/src/components/panel/SidebarPanel.tsx`**
- Botón switch visible si `isAdmin || impersonating`
- Mostrar badge "Impersonando: [rol]" cuando impersonating=true

**10. `frontend/src/components/panel/SeccionProyectos.tsx`**
- Admin view: mostrar assigned_employee_name en OrdenCard footer
- Admin view: agregar campo de búsqueda + filtro por empleado
- Solo para admin (no impersonando)

### Orden de implementación
1. Backend Claims + generate_token (auth.rs service)
2. Backend AuthUser (middleware/auth.rs)
3. Backend AuthResponse (models/user.rs)
4. Backend find_first_by_role (repositories/user.rs)
5. Backend switch_role handler (handlers/orders.rs)
6. Backend list_orders fix (handlers/orders.rs)
7. cargo check + clippy
8. cargo sqlx prepare (por nueva query)
9. Frontend auth types + store
10. Frontend SidebarPanel
11. Frontend SeccionProyectos (admin filters)
12. npx tsc --noEmit
13. Test completo
