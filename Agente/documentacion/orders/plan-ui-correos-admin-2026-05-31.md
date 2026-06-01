# Plan UI: Panel de Correos Enviados (Admin)

**Fecha:** 2026-05-31
**Estado:** Planificación — pendiente de implementación

---

## 1. Resumen

Nueva sección en el panel admin para visualizar todos los correos enviados desde la plataforma, con filtro por tipo de plantilla. Sigue los patrones visuales existentes (SeccionReembolsos, SeccionHosting).

---

## 2. Backend API

### 2.1 Endpoint

| Método | Ruta | Descripción |
|--------|------|-------------|
| `GET` | `/api/admin/email-logs` | Lista paginada de correos enviados |

**Query params:**
| Param | Tipo | Default | Descripción |
|-------|------|---------|-------------|
| `template` | `string` (opcional) | `null` | Filtrar por tipo de plantilla (`order_confirmation`, `new_order_admin`, etc.) |
| `limit` | `i32` | `50` | Máximo 100 |
| `offset` | `i32` | `0` | Paginación |

**Response:**
```json
{
  "logs": [
    {
      "id": "uuid",
      "to_email": "cliente@example.com",
      "subject": "🆕 Nueva orden #123 — Nakomi Studio",
      "template": "new_order_admin",
      "reference_type": "order",
      "reference_id": "uuid",
      "status": "sent",
      "sent_at": "2026-05-31T12:00:00Z",
      "created_at": "2026-05-31T12:00:00Z"
    }
  ],
  "total": 150
}
```

### 2.2 Template types (para el filtro)

Basado en las funciones existentes + nuevas:

| Template | Etiqueta | ¿Implementado? |
|----------|----------|:--------------:|
| `order_confirmation` | Confirmación al cliente | ✅ |
| `new_order_admin` | Nueva orden (admin) | ✅ |
| `payment_received_admin` | Pago recibido (admin) | ✅ |
| `order_completed_client` | Orden completada (cliente) | Pendiente Fase 4 |
| `order_cancelled_client` | Orden cancelada (cliente) | Pendiente Fase 4 |
| `phase_delivered_client` | Fase entregada (cliente) | Pendiente Fase 4 |
| `problem_reported_client` | Problema reportado (cliente) | Pendiente Fase 4 |
| `escalation` | Escalación de chat (admin) | ✅ |
| `chat_invoice_paid_client` | Factura chat pagada (cliente) | ✅ |
| `chat_invoice_paid_admin` | Factura chat pagada (admin) | ✅ |
| `vps_pending_approval` | VPS pendiente (admin) | ✅ |
| `vps_approved` | VPS aprobado (cliente) | ✅ |
| `vps_rejected` | VPS rechazado (cliente) | ✅ |
| `profile_email_changed_new` | Email cambiado (nuevo) | ✅ |
| `profile_email_changed_old` | Email cambiado (anterior) | ✅ |
| `profile_password_changed` | Contraseña cambiada | ✅ |

---

## 3. Frontend

### 3.1 Componentes a crear/modificar

| Archivo | Acción |
|---------|--------|
| `frontend/src/api/admin-email.ts` | **CREAR** — API client para email-logs |
| `frontend/src/components/panel/SeccionCorreo.tsx` | **CREAR** — Componente de la sección |
| `frontend/src/components/panel/SeccionCorreo.css` | **CREAR** — Estilos de la sección |
| `frontend/src/data/panel.ts` | **MODIFICAR** — Agregar tab `correos` a TABS_ADMIN |
| `frontend/src/components/panel/SidebarPanel.tsx` | **MODIFICAR** — Agregar icono `Mail` para `correos` |
| `frontend/src/islands/PanelIsland.tsx` | **MODIFICAR** — Agregar import + case para `correos` |

### 3.2 Estructura de SeccionCorreo.tsx

```
SeccionCorreo
├── Select (filtro por template)
│   ├── Opción: "Todas las plantillas" (default)
│   └── Opciones: cada template con su etiqueta
├── Tabla de correos
│   ├── Columnas: Plantilla, Destinatario, Asunto, Estado, Fecha
│   ├── Badge de estado (enviado/fallido)
│   └── Fila clickeable → abre detalle en modal
└── Paginación
    ├── "Anterior" / "Siguiente"
    └── Contador: "Mostrando 1-50 de 150"
```

### 3.3 Patrón visual

Seguir el mismo patrón que `SeccionReembolsos`:
- `.correosContenedor` — flex column, gap md
- `.correosTabla` — tabla con bordes, filas con hover
- `.correosFiltro` — select/barra de filtros arriba
- `.correosPaginacion` — controles de paginación abajo
- Badge de estado: `sent` → verde, `failed` → rojo, `pending` → gris
- Usar `lucide-react` icons: `Mail`, `Filter`, `ChevronLeft`, `ChevronRight`

### 3.4 Estados

| Estado | Qué mostrar |
|--------|-------------|
| Cargando | Loader2 spinner (igual que reembolsos) |
| Error | Alerta con mensaje (igual que reembolsos) |
| Vacío | "No hay correos enviados" con icono Mail |
| Con datos | Tabla + filtros + paginación |

---

## 4. Dependencias

- Backend: Fase 5 (email_logs migration + repository + API) debe estar completo
- No requiere nuevas dependencias npm (lucide-react ya incluido)

---

## 5. Orden de implementación

1. Migración SQL `email_logs`
2. Repositorio `email_log.rs`
3. Integrar logging en `EmailService::send()`
4. Endpoint API `GET /api/admin/email-logs`
5. Registrar ruta en `handlers/mod.rs`
6. Fase 4: funciones de email lifecycle + integración en handlers
7. UI: `admin-email.ts` API client
8. UI: `SeccionCorreo.tsx` + `SeccionCorreo.css`
9. UI: Registrar tab en panel.ts, SidebarPanel, PanelIsland
10. Validar compilación backend + frontend
