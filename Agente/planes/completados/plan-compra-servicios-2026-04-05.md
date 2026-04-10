# Plan: Reestructuración compra de servicios (044A-40)
Fecha: 2026-04-05

## Alcance
1. Restructurar página de servicios: portada primero, botón "Conversar" debajo de CTA
2. Planes: quitar "A medida" → solo Básico, Medio, Pro
3. Modal de compra al dar click en "Empezar" — muestra info breve + precio + botón "Continuar ($X)"
4. Guest checkout: si no está logueado, pedir email+contraseña en el modal
5. Flujo: Modal → (auth si necesario) → crear orden → Stripe → panel

## Fases de implementación

### Fase 1: Datos (planes.ts)
- Quitar tercer tier "A medida"/"Personalizado" de todos los servicios
- Renombrar tiers: Básico, Avanzado → Básico, Pro (o dejar como están: Básico, Avanzado)
- Quitar `esPersonalizado` flag

### Fase 2: Página de servicio individual
- Reordenar secciones: portada/hero primero
- Debajo de botones CTA de plan, agregar "Conversar" que abre chat
- El botón conversar debe pasar contexto: qué servicio y plan estaba mirando el usuario

### Fase 3: ModalCompra (nuevo componente)
- Se abre al click en CTA de plan
- Muestra: nombre servicio, nombre plan, precio, breve descripción
- Botón "Continuar ($X)" → avanza a paso 2
- Texto "Aún no se te cobrará"
- Si NO logueado: mostrar form email+contraseña inline (registro rápido)
- Si logueado: directo a crear orden + redirect Stripe

### Fase 4: Integración
- SeccionPlanesServicio.tsx invoca ModalCompra en vez de redireccionar a /contacto/
- Crear orden via API existente
- Iniciar pago via API existente
- Redirigir al panel después del pago

## Estado
- [ ] Fase 1
- [ ] Fase 2
- [ ] Fase 3
- [ ] Fase 4
