# Plan: Hosting Profesional v2 — Nakomi Studio

**Fecha:** 2026-04-09  
**Contexto:** El usuario reporta que la interfaz actual "no parece un hosting real". Se necesita rediseñar el panel de hosting para que se vea y funcione como cPanel/Hostinger/Bluehost pero simplificado.  
**Restricciones del usuario:**
- NO compras automáticas a Contabo (requiere confirmación humana)
- NO gestor de archivos ni features complejas tipo cPanel (algo básico pero profesional)
- Todo debe tener tests
- Auditoría de seguridad obligatoria como tarea final

---

## Estado actual

### Backend (funcional)
- 11 endpoints REST para suscripciones, status, eventos, checkout, VPS
- Stripe Checkout Sessions + webhooks (hosting_stripe.rs)
- Contabo API (OAuth2, list/get VPS)
- Prices en Stripe: Básico $5/mo, Pro $10/mo, E-commerce $15/mo

### Frontend (incompleto — aquí está el problema)
- Vista de cards (lista plana) sin página individual por hosting
- Stats simulados (hash-based, no datos reales)
- Solo admin crea suscripciones; cliente no puede contratar desde panel
- Sin info de dominio, DNS, SSL, SSH
- Sin upgrade de plan desde panel
- Sin acceso rápido a soporte

---

## Fases de implementación

### Fase 1 — Página de detalle individual (094A-2)
**Lo más difícil e impactante. Transforma la experiencia.**

**Componentes nuevos:**
- `HostingDetalle.tsx` — vista completa por hosting con tabs internos
- `HostingDetalle.css` — estilos
- `useHostingDetalle.ts` — hook de estado + lógica

**Navegación:**
- Click en HostingCard → setSelectedHostingId → render HostingDetalle
- Back button → volver a lista

**Secciones del detalle (tabs internos):**
| Tab | Contenido |
|-----|-----------|
| **General** | Status, plan, precio, dominio, fecha creación/renovación, acciones rápidas (visitar sitio, reiniciar) |
| **Recursos** | Storage bar, bandwidth bar, uptime — datos simulados mejorados (luego reales en Fase 6) |
| **Dominio & SSL** | Dominio actual, nameservers del VPS, instrucciones DNS, status SSL (Let's Encrypt auto) |
| **Acceso** | SSH host/port/user, instrucciones de conexión, info SFTP |
| **Facturación** | Plan actual, precio, botón upgrade, historial de pagos (Stripe links) |
| **Eventos** | Historial de actividad (reutilizar EventsPanel existente) |

### Fase 2 — Auto-servicio cliente (094A-3)
**El cliente puede contratar hosting desde su panel.**

- Botón "Contratar Hosting" visible para clientes (no solo admin)
- Flujo: selección de plan → Stripe Checkout → suscripción creada → webhook activa
- Reutiliza HostingStripeService.create_checkout_session()
- Necesita ajuste: actualmente create_subscription requiere admin; para self-service el backend crea la suscripción Y el checkout en un solo flujo

### Fase 3 — Dominio & DNS (094A-4)
- Mostrar info de nameservers del VPS (ns1/ns2 de Contabo o Cloudflare)
- Instrucciones para apuntar dominio
- Status SSL (Let's Encrypt auto por Coolify)
- Futuro: Contabo DNS API para gestión de registros DNS

### Fase 4 — Acceso SSH (094A-5)
- Mostrar credenciales SSH del contenedor Coolify
- Host = IP del VPS, Port = puerto SSH del contenedor
- Instrucciones de conexión (terminal, PuTTY)
- Futuro: SSH key management

### Fase 5 — Upgrade de plan (094A-6)
- Stripe subscription update API (cambiar price en la subscription)
- Frontend: botón "Cambiar plan" en la tab Facturación
- Prorateo automático (Stripe lo maneja)

### Fase 6 — Stats reales (094A-8)
- Opción A: Coolify API /api/v1/applications/{uuid}/metrics
- Opción B: Monitoring agent en el VPS (Prometheus + simple exporter)
- Opción C: cAdvisor por contenedor (más ligero)
- Por ahora: mejorar los stats simulados con indicadores más realistas
- Futuro: endpoint backend que consulte Coolify y cache resultados

### Fase 7 — Soporte (094A-7)
- Botón "Contactar soporte" en el detalle
- Navega al chat del panel (reutilizar SeccionChat)
- Pre-llena contexto: "Soporte para hosting {nombre} - Plan {plan}"

### Fase 8 — Auditoría de seguridad (094A-9)
- Revisión de permisos (cliente solo ve sus propios hostings)
- SQL injection review (todos prepared statements)
- Rate limiting en endpoints de hosting
- Validación de inputs en frontend y backend
- Aislamiento de contenedores Coolify (red, filesystem)
- Protección contra caídas: resource limits por contenedor
- Plan de monitoreo y alertas

### Fase 9 — Tests (094A-10)
- Backend: tests de endpoints hosting (crear, listar, status, checkout)
- Frontend: tests de componentes (HostingDetalle, HostingCard, flujo checkout)
- Integration: flujo completo contratar → pagar → activar → ver detalle

---

## Arquitectura de archivos

```
frontend/src/
  components/panel/
    SeccionHosting.tsx          (MODIFICAR: agregar navegación a detalle)
    HostingDetalle.tsx          (NUEVO: página de detalle)
    HostingDetalle.css          (NUEVO: estilos)
    HostingSubComponents.tsx    (MODIFICAR: card clickable)
    HostingStats.tsx            (MANTENER: luego reemplazar datos)
  hooks/
    useSeccionHosting.ts        (MODIFICAR: agregar selectedHostingId)
    useHostingDetalle.ts        (NUEVO: lógica del detalle)
  api/
    hosting.ts                  (MODIFICAR: agregar endpoints nuevos)

src/
  handlers/hosting.rs           (MODIFICAR: endpoints nuevos si necesario)
  services/hosting_stripe.rs    (MANTENER: ya funcional)
```

---

## Restricciones importantes

1. **No compras Contabo automáticas** — cualquier acción que genere costo en Contabo requiere aprobación humana explícita
2. **Componentes max 300 líneas** — dividir en sub-componentes si excede
3. **Hooks max 120 líneas** — extraer lógica pesada
4. **Max 3 useState por componente** — usar hook dedicado
5. **CSS variables obligatorias** — no hex directos
6. **Responsive: 320px, 768px, 1024px** — probar en 2 resoluciones mínimo
