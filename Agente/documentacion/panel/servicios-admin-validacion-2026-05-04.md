# Servicios admin: validacion y errores de escritura

Fecha: 2026-05-04

## Cambio

Se endurecio el update de servicios del CMS para que no dependa de errores SQL genericos como control de flujo.

## Decisiones

- `UpdateServiceRequest` ahora deriva `Validate` y corta payloads invalidos antes de tocar la base.
- El handler admin usa `map_service_write_error()` para traducir conflictos y constraints comunes a respuestas 4xx.
- `ServiceRepository::update_service()` dejo de usar la macro `query_as!` y paso a `query_as::<_, ServiceRecord>(...).bind(...)` con `UpdateServiceParams<'_>` para mantener el update alineado con el patron usado en proyectos.

## Efecto esperado

- Slugs duplicados en `services` responden conflicto explicito en vez de `500` generico.
- Longitudes fuera de rango o violaciones simples de constraint dejan mensaje de validacion mas accionable para el CMS.
- El slice backend de servicios queda mas cercano al contrato y ergonomia del slice de proyectos.

## Alcance

No cambia el contrato del endpoint ni introduce nuevos campos. Solo endurece validacion y traduccion de errores para writes admin de servicios.
