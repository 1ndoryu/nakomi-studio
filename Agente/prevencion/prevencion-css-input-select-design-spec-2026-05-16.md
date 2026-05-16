# Prevención: overrides CSS locales en inputs/selects

## Caso 165A-16

Sentinel no señaló `.dnsFormField input, .dnsFormField select` aunque redefinía padding, borde, fondo, color y tipografía de campos que ya tienen componentes base (`Input`, `Select`).

## Regla propuesta

Detectar selectores CSS de componentes que apunten a `input`, `select`, `textarea` o `button` dentro de una clase local y redefinan propiedades visuales resueltas por el sistema: `padding`, `border`, `border-radius`, `background`, `color`, `font-size`, `font-family`, `box-shadow`.

## Excepción válida

Permitir ajustes estructurales mínimos (`width`, `min-width`, `flex`, `grid-column`) cuando no alteran la receta visual del componente base.

## Acción pendiente

Agregar fixture en `code-sentinel` y regla específica para overrides de controles nativos/componentes base dentro de CSS de feature.