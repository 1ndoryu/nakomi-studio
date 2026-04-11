# Plan: Delegaciones y Pedidos — Wallet + Admin Assignment + Withdrawal

**Fecha:** 2026-04-18
**Estado:** Completado (T1, T2, T3 terminadas)

## Análisis previo

La mayoría del sistema ya está implementado:

| Feature | Estado |
|---------|--------|
| Wallet system (balances, transacciones) | ✅ Completo |
| Cancellation request (empleado → cliente) | ✅ Completo |
| Cliente acepta/rechaza cancelación | ✅ Completo |
| On reject: orden re-available | ✅ Completo |
| 48h admin-exclusive + auto-open | ✅ Completo |
| Delegaciones entre empleados | ✅ Completo |
| SeccionDelegaciones + SeccionDisponibles UI | ✅ Completo |
| Notificaciones REST + WebSocket | ✅ Completo |
| **Wallet balance en header panel** | ✅ Completo (T3-wallet-header) |
| **Admin UI para asignar órdenes a empleados** | ✅ Completo (T2-assignment) |
| **Withdrawal (retiro de saldo)** | ✅ Completo (T1-withdrawal) |

## Tareas (3 tasks, por complejidad descendente)

### T1: Withdrawal system (backend + frontend)
- Crear migration: `withdrawal_requests` table (id, user_id, amount_cents, status, payment_method, admin_notes, created_at, resolved_at)
- Endpoint `POST /api/wallet/withdraw` — crear solicitud de retiro
- Endpoint `GET /api/wallet/withdrawals` — listar solicitudes del usuario
- Endpoint admin `PATCH /api/admin/withdrawals/{id}` — aprobar/rechazar
- Endpoint admin `GET /api/admin/withdrawals` — listar pendientes
- Frontend: botón "Solicitar retiro" en SeccionWallet + lista de solicitudes
- Frontend: admin view para gestionar solicitudes de retiro
- Notificaciones: admin notificado de nueva solicitud, usuario notificado de resolución

### T2: Admin UI para asignar órdenes
- API `PUT /api/orders/{order_id}/assign/{employee_id}` ya existe
- Crear componente `SeccionOrdenesAdmin.tsx` (o extender SeccionProyectos) con:
  - Tab/filtro para ver órdenes sin asignar (awaiting_assignment)
  - Dropdown/modal para seleccionar empleado y asignar
  - Lista de empleados disponibles (endpoint `GET /api/admin/employees` ya existe)
- Reemplazar el uso genérico de SeccionProyectos en el panel admin por este componente

### T3: Wallet balance en header del panel
- Hook `useWalletBalance()` (llama GET /api/wallet, cache 30s)
- Mostrar balance mini en HeaderPanel.tsx (junto a notificaciones)
- Visible para todos los usuarios autenticados
- Click navega a pestaña wallet

## Notas
- La cancelación empleado→cliente→aprobación ya funciona correctamente
- El 48h auto-open + notificación a empleados ya existe en auto_assign_loop
- Las tabs Delegaciones y Disponibles ya están implementadas con diseño completo
