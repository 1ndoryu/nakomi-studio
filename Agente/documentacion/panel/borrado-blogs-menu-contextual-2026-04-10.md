# Borrado de blogs en CMS — 2026-04-10

## Problema

- En el panel CMS parecía que no se podían borrar artículos del blog desde el menú de 3 puntos.
- El síntoma era visual/interactivo, no de backend.

## Diagnóstico

- Validación local real del endpoint:
  - login admin con `admin@admin.com` / `admin`
  - creación de un post temporal con `POST /api/admin/blog`
  - borrado permanente con `POST /api/admin/blog/{id}/destroy`
  - respuesta `204 No Content` y desaparición confirmada al relistar `/api/admin/blog`
- La card de `ListaBlog.css` usaba `overflow: hidden`.
- El wrapper del menú contextual solo ganaba visibilidad con `:hover` de la card.
- Cuando el panel del menú salía del contenedor, la opción destructiva podía quedar clipeada o perder visibilidad durante la interacción.

## Corrección

- `ListaBlogCard` ahora usa `overflow: visible` para no recortar el panel contextual.
- `ListaBlogImagen` conserva las esquinas superiores redondeadas para no perder acabado visual.
- `listaBlogMenu` mantiene visibilidad también con `:hover` y `:focus-within`, no solo con hover del card.
- Se documentó una excepción `sentinel-disable-file css-adhoc-button-style` porque esta hoja define cards clickeables del CMS, no botones artesanales del sistema.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- Prueba HTTP local create/delete del blog con confirmación del borrado

## Gotcha

- Si un `MenuContextual` se renderiza fuera de una card clickeable, la card no puede seguir con `overflow: hidden` ni depender solo del hover del contenedor para mostrar el wrapper del menú.