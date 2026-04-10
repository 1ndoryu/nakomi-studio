# Regla Sentinel: button-clase-especifica

Fecha: 2026-04-09

## Objetivo

Evitar que vuelvan a aparecer botones con `className` visual propio como `reembolsoBotonRevisar`, `ctaBotonPrincipal` o variantes equivalentes que duplican estilos ya cubiertos por `Button`.

## Alcance

- Analyzer: React (`reactComponentRules.ts`)
- Regla registrada: `button-clase-especifica`
- Severidad default: `warning`

La regla inspecciona bloques JSX que empiezan con `<Button>`, `<Boton>` o `<button>` y revisa el `className` cercano. Si encuentra tokens con patrón de botón específico (`...Boton...`, `...button...`, `cta-boton-principal`, etc.), reporta violación.

## Qué NO marca

- Clases del propio sistema (`botonBase`, `botonPrimario`, `botonTexto`, etc.)
- Elementos sin `className`
- Clases que no son de botón aunque vivan sobre un `Button` (`perfilReviewTab`)
- Archivos del propio componente base (`Button`, `Boton`, `BotonBase`)

## Uso recomendado

- Preferir `variante`, `tamano`, `disabled` y composición interna de `Button`.
- Si hace falta separar icono/texto o layout, aplicar clase al contenido interno (`span`, `div`) y no al botón en sí.
- Si un caso excepcional justifica saltarse la regla, usar `sentinel-disable-next-line button-clase-especifica` o `sentinel-disable-file button-clase-especifica`.

## Validación hecha

- `npm run compile` en `code-sentinel`
- Test aislado: `npx mocha --no-config --ui tdd --color --timeout 10000 --require out/test/registerMocks.js out/test/suite/buttonSpecificClass.test.js`

## Caso corregido en Nakomi

`SeccionReembolsos` dejó de usar `reembolsoBotonRevisar`, `reembolsoBotonAprobar`, `reembolsoBotonRechazar` y `reembolsoBotonCancelar`. Ahora usa variantes del componente `Button` y una clase neutra sobre contenido interno (`reembolsoAccionContenido`).