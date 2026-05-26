# Limites dinamicos en hosting administrado

> Fecha: 2026-05-26
> Origen: investigacion posterior a `255A-4` sobre limites reales vs. plan comercial en `/panel?seccion=infraestructura`
> Estado: implementado parcialmente
> Decision vigente: CPU burst en Coolify ya implementado como fase 1; RAM dinamica sigue diferida hasta tener persistencia de overrides y observabilidad especifica

## Resumen ejecutivo

- El panel ya no debe adivinar limites. La fuente de verdad del runtime es `docker inspect`, no el plan comercial ni `docker stats` por si solo.
- Se confirmo que hay despliegues legacy sin caps Docker reales; en esos casos la UI debe mostrar "sin limite runtime detectado" y no sintetizar enforcement inexistente.
- La idea de limites dinamicos es tecnicamente viable, pero no debe implementarse como parche sobre el compose actual.
- La ruta sensata es separar tres conceptos que hoy se mezclan:
  - garantia comercial del plan
  - limite runtime realmente aplicado en el contenedor
  - burst temporal otorgado por disponibilidad del host
- La primera fase ya implementada es CPU dinamica para hostings Coolify. RAM dinamica tambien es posible, pero su riesgo operativo es bastante mayor.

## Implementado 2026-05-26

- Se añadió `src/services/cpu_burst.rs`, un loop en background que usa las últimas muestras persistidas para decidir si un hosting Coolify necesita más CPU o debe volver a su baseline.
- La aplicación del cambio se hace en caliente con `docker update --cpus`, resolviendo el contenedor real por `coolify_site_name` y los servicios `site` o `wordpress` del compose.
- La política actual solo actúa sobre el contenedor principal del hosting, mantiene una reserva fija de CPU por VPS y exige ventanas de estabilidad antes de subir o bajar el cap runtime.
- Cada ajuste deja evento operativo en la suscripción (`cpu_burst_applied` / `cpu_burst_restored`), pero todavía no existe UI dedicada para mostrar el burst activo ni persistencia separada de overrides.

## Problema de producto

Los limites fijos actuales son demasiado agresivos cuando la VPS tiene capacidad libre. Eso genera dos problemas distintos:

1. Producto: un cliente ve un hosting "capado" incluso cuando el servidor esta holgado.
2. Semantica: el panel puede inducir a creer que existe un cap real cuando en runtime no hay ninguno.

La correccion de `255A-4` resolvio el segundo problema: el panel ya separa plan vs. runtime real. El siguiente paso, si se quiere mejorar el producto, es que el sistema pueda asignar recursos de forma dinamica cuando haya margen real en la VPS.

## Situacion actual del sistema

### Lo que ya existe

- `src/services/infrastructure_metrics.rs` ya captura muestras de infraestructura y ahora tambien limites runtime efectivos por contenedor via `docker inspect`.
- `src/models/hosting/responses.rs` y `frontend/src/components/panel/DeploymentRow.tsx` ya distinguen recursos del plan y limites runtime detectados.
- `src/services/bandwidth_enforcement.rs` ya demuestra el patron operativo correcto para este tipo de decisiones: loop en background, histeresis, restauracion y eventos.
- `src/main.rs` ya arranca loops de enforcement y observabilidad en segundo plano.
- `src/services/hosting_runtime.rs` ya separa el concepto de runtime (`coolify` vs `lightweight`), aunque hoy la investigacion y la propuesta solo son realistas para Coolify.

### Lo que hoy impide limites dinamicos reales

- `src/repositories/hosting.rs` trata la configuracion del plan como fuente de verdad de CPU/RAM/disco comerciales.
- `src/services/coolify.rs` convierte esos valores en limites estaticos dentro del compose al provisionar o actualizar.
- `src/services/coolify.rs::update_compose_and_restart` obliga a pasar por rewrite de compose + restart, que no sirve para rebalanceo frecuente.
- No existe un loop que observe holgura de VPS y ajuste recursos en caliente.
- No existe una capa que persista overrides runtime temporales ni los reaplique tras un redeploy.

## Conclusiones tecnicas

## 1. Si se puede hacer

Docker permite ajustar CPU y memoria en runtime sin recrear necesariamente el contenedor. Eso hace viable un balancer que cambie limites efectivos segun carga y capacidad libre.

## 2. No conviene hacerlo desde el compose

Reescribir compose y reiniciar en cada cambio seria demasiado caro, fragil y visible para el usuario. El compose debe seguir representando el baseline estable del servicio, no el estado dinamico minuto a minuto.

## 3. CPU y RAM no tienen el mismo nivel de riesgo

- CPU dinamica: viable con riesgo medio. Es el mejor primer objetivo.
- RAM dinamica: viable con riesgo alto. Reducir memoria en caliente o mover caps agresivamente puede provocar OOM, degradacion brusca o reinicios.

## 4. El plan no debe seguir significando "cap permanente"

Para soportar burst real, el plan debe reinterpretarse como garantia minima o baseline contractual, no como techo tecnico fijo en todo momento.

## Arquitectura recomendada

## A. Separar garantia, cap runtime y burst

El modelo correcto pasa por tres capas:

- Garantia del plan: el minimo comercial prometido al cliente.
- Cap runtime actual: el limite efectivo que hoy tiene el contenedor.
- Burst temporal: aumento o relajacion aplicada por el balancer cuando hay holgura real en la VPS.

Eso permite responder preguntas distintas sin mezclar conceptos:

- "Que prometimos en el plan?"
- "Que cap real tiene ahora mismo el contenedor?"
- "Cuanto extra le estamos prestando mientras el host esta libre?"

## B. Nuevo loop de balanceo

Crear un loop nuevo, por ejemplo `resource_balancer_loop`, con estas caracteristicas:

- periodicidad corta pero no agresiva, por ejemplo cada 2-5 minutos
- lectura desde snapshots persistidos, no SSH directo por decision
- histeresis para evitar subidas y bajadas constantes
- reserva minima por VPS para no consumir toda la capacidad del host
- auditoria de cada cambio aplicado

Entradas minimas del loop:

- uso promedio reciente de CPU/RAM por VPS
- uso promedio reciente por despliegue
- limites runtime actuales detectados
- estado del deployment y del host
- baseline garantizado por suscripcion o plan

Salidas del loop:

- aumento temporal de CPU para despliegues elegibles
- restauracion al baseline cuando desaparece la holgura o baja la demanda
- eventos para trazabilidad y panel admin

## C. Ejecutor por runtime

Aunque la primera implementacion debe enfocarse en Coolify, el ajuste runtime no deberia vivir mezclado con reglas de producto. Conviene introducir una capa de ejecucion por runtime:

- `CoolifyDynamicResourceExecutor` para `docker update` y deteccion de contenedores reales
- adaptador futuro para `lightweight`, si algun dia se quiere comportamiento equivalente alli

La politica decide "que" recursos asignar. El executor decide "como" aplicarlos en cada runtime.

## MVP recomendado

## Fase 1: CPU dinamica solo para hostings Coolify

Objetivo:

- mantener un baseline garantizado por plan
- permitir burst de CPU cuando la VPS este holgada
- restaurar al baseline cuando el host vuelva a tener presion

Reglas sugeridas para el MVP:

- aplicar solo sobre despliegues `coolify`
- empezar por el contenedor `site`; `db` y `ssh` pueden quedar fuera en la primera iteracion
- exigir varias muestras consecutivas antes de subir o bajar recursos
- mantener una reserva fija de CPU para sistema, proxy, base de datos compartida y picos inesperados
- no perseguir precision al segundo; basta una reaccion estable y explicable

Estado 2026-05-26: implementado para el contenedor principal del hosting (`site` o `wordpress`) usando snapshots y `docker update --cpus`. Quedan fuera por ahora `db`, `ssh`, persistencia específica de overrides y visualización explícita del burst en panel.

## Fase 2: persistencia de overrides runtime

Agregar una tabla o modelo equivalente para recordar el ultimo override aplicado por despliegue y rol. Sirve para:

- re-aplicar el estado deseado tras restart o redeploy
- detectar drift entre lo esperado y lo observado
- alimentar la UI y auditoria

Campos minimos deseables:

- deployment id
- runtime kind
- container role (`site`, `db`, `ssh`)
- baseline garantizado
- target actual decidido por el balancer
- ultimo valor aplicado con exito
- timestamp y motivo del cambio

## Fase 3: mostrar burst real en el panel

El panel de infraestructura deberia evolucionar para mostrar tres cosas por separado:

- garantia del plan
- limite runtime actual
- burst u override activo

Si no hay cap real, debe seguir viendose como tal. Si hay burst temporal, tambien debe ser visible como dato operativo y no como parte del plan.

## Fase 4: RAM dinamica, solo despues de estabilizar CPU

Antes de tocar memoria, deben existir estas protecciones:

- reserva dura por VPS
- maximo burst por plan o por tier
- histeresis mas lenta que en CPU
- reaplicacion confiable tras restart
- politica explicita para no bajar memoria de forma peligrosa en caliente

Mientras esas piezas no existan, RAM dinamica no debe ser el primer entregable.

## Riesgos criticos

## 1. Coolify puede sobrescribir el estado runtime

Un redeploy o recreate puede perder overrides aplicados en caliente. El balancer debe detectar drift y re-aplicar el estado si sigue siendo valido.

## 2. Oscilacion de recursos

Sin histeresis y ventanas de decision, el sistema puede entrar en ciclos de subir y bajar caps cada pocos minutos.

## 3. Starvation del host

Si el balancer reparte todo el margen libre sin reserva, puede perjudicar al proxy, la base de datos, jobs internos o contenedores no contemplados por la politica.

## 4. Memoria dinamica mal ajustada

Es el riesgo tecnico mas serio. Un ajuste agresivo o una bajada inapropiada puede empujar contenedores a OOM.

## 5. Mezclar runtime logic con logica de producto

La politica de negocio debe vivir en backend agnostico. La aplicacion concreta del cambio debe vivir detras de un executor por runtime.

## Recomendacion final

- Mantener la correccion ya hecha: nunca volver a mostrar limites inventados cuando no haya cap runtime real.
- Implementar primero burst dinamico de CPU para hostings Coolify, con baseline garantizado y reserva por VPS.
- No tocar RAM dinamica en la primera fase.
- No intentar soportar `lightweight` desde el dia uno; primero validar la politica en el runtime que hoy controla los hostings administrados mas relevantes.

## Anclajes del repo para una futura implementacion

- Observabilidad y limites reales: `src/services/infrastructure_metrics.rs`
- Contrato backend del panel: `src/models/hosting/responses.rs`
- UI actual de plan vs runtime: `frontend/src/components/panel/DeploymentRow.tsx`
- Baseline comercial actual: `src/repositories/hosting.rs`
- Compose y restart actuales: `src/services/coolify.rs`
- Patron de enforcement en background: `src/services/bandwidth_enforcement.rs`
- Arranque de loops: `src/main.rs`
- Abstraccion de runtime: `src/services/hosting_runtime.rs`

## Decision operativa propuesta

Si este frente se aborda, la secuencia correcta es:

1. CPU burst para Coolify.
2. Persistencia y reconciliacion de overrides runtime.
3. UI con baseline + runtime + burst.
4. Evaluar RAM dinamica solo despues de ver estabilidad real en produccion.