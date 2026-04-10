# Consistencia de iconos de cards — 2026-04-09

## Objetivo

Eliminar la divergencia entre `panelCardIcono` y la variante de hosting que terminó convirtiéndose en la referencia visual correcta.

## Cambios aplicados

- `panelCardIcono` pasa a ser la base única para cards del panel.
- La base adopta el look que ya estaba validado en hosting: ancho amplio y fondo de acento.
- `HostingCard` deja de usar `hostingCardIcono` y compone directamente con `panelCardIcono`.
- `SeccionHosting.css` ya no mantiene una receta paralela para el bloque de icono.

## Prevención

- Se añadió en `code-sentinel` la regla `card-icono-debe-extender-base`.
- La regla detecta clases `*CardIcono` que vuelvan a declarar `display`, alineación o `flex-shrink` en CSS fuera de `PanelIsland.css`.
- La expectativa es componer con `.panelCardIcono` en JSX y dejar en la clase variante solo overrides puntuales.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- `npm run compile` en `code-sentinel`
- `npx mocha --no-config --ui tdd --color --timeout 10000 --require out/test/registerMocks.js out/test/suite/cardIconBase.test.js`