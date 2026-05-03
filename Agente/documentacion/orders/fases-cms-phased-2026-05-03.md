# Fases CMS en pagos por fases

Fecha: 2026-05-03

## Contrato

- `service_plan_phases` es la fuente de verdad para la estructura de trabajo de un plan.
- Cuando una orden se crea con `payment_mode = phased`, las fases de `order_phases` deben copiar exactamente el título, descripción, porcentaje, días estimados y revisiones definidas en el CMS.
- El backend no debe inventar títulos como "Fase n por definir" ni sobrescribir la plantilla del plan con texto derivado del brief.

## Guardarrailes

- `POST /api/orders` rechaza compras `phased` si el plan no tiene fases configuradas.
- `PUT /api/admin/services/{id}/plans` rechaza guardar un plan sin fases.

## Edición posterior del plan

- `PUT /api/admin/services/{id}/plans` debe preservar `service_plans.id` para los planes existentes.
- Una orden ya creada puede seguir apuntando a su `plan_id` histórico aunque el CMS cambie slug, nombre, precio o fases del plan.
- Solo las filas omitidas del payload se intentan borrar; si una orden todavía referencia ese plan, la FK sigue actuando como guardarraíl y evita borrados inválidos.

## Motivo

El catálogo público ya vende planes dinámicos desde CMS. Si las fases se vuelven genéricas al crear la orden, el empleado termina viendo un proyecto activo sin una estructura clara y parece que la edición manual fuera el flujo principal, cuando debería ser solo una excepción.

Además, si el guardado admin elimina y recrea `service_plans`, en cuanto existe una orden apuntando al plan la edición deja de ser estable y da la falsa impresión de que "no se pueden cambiar las fases del proyecto". La persistencia correcta es actualizar el plan existente y recrear solo sus fases hijas.