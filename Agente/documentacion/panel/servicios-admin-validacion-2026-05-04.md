# Servicios admin: validacion y errores de escritura

Fecha: 2026-05-04

## Cambio

Se endurecio el update de servicios del CMS para que no dependa de errores SQL genericos como control de flujo.

## Decisiones

- `UpdateServiceRequest` ahora deriva `Validate` y corta payloads invalidos antes de tocar la base.
- El handler admin usa `map_service_write_error()` para traducir conflictos y constraints comunes a respuestas 4xx.
- `ServiceRepository::update_service()` dejo de usar la macro `query_as!` y paso a `query_as::<_, ServiceRecord>(...).bind(...)` con `UpdateServiceParams<'_>` para mantener el update alineado con el patron usado en proyectos.
- `useContenidoServicios()` extrae `response.data.message` de Axios y lo muestra en el CMS con toast/error local, para no degradar un `409 conflict` a `Request failed with status code 409`.
- `SubTabServicios` hace un preflight de slugs contra la lista ya cargada y corta el guardado si detecta que el slug ya pertenece a otro servicio.
- `SubTabServicios` ahora valida también los planes antes del submit: slug y nombre obligatorios, slug único dentro del servicio, al menos una fase por plan y título obligatorio en cada fase.
- El `PUT /api/admin/services/:id/plans` ya no falla en silencio en consola: si el backend responde `422`, el editor extrae `response.data.message` y lo muestra en toast.
- `SubTabServicios` compara el snapshot original de planes con el payload actual y solo llama a `/plans` cuando realmente hubo cambios en la tab de planes.

## Efecto esperado

- Slugs duplicados en `services` responden conflicto explicito en vez de `500` generico.
- Longitudes fuera de rango o violaciones simples de constraint dejan mensaje de validacion mas accionable para el CMS.
- El slice backend de servicios queda mas cercano al contrato y ergonomia del slice de proyectos.
- Si el conflicto es visible desde el listado ya cargado, el usuario recibe feedback inmediato sin esperar al roundtrip del API.
- Si el conflicto solo existe en servidor, el panel sigue mostrando el mensaje real devuelto por el backend.
- El editor evita guardados parciales cuando el problema real está en `plans`: corta antes del `PUT` principal si detecta payload inválido en la tab de planes.
- Editar imagen, metadatos o campos generales ya no dispara un `PUT /plans` redundante que pueda ensuciar el flujo con un `422` ajeno.

## Alcance

No cambia el contrato del endpoint ni introduce nuevos campos. Solo endurece validacion y traduccion de errores para writes admin de servicios.
