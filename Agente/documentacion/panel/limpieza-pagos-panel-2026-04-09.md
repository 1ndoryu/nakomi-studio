# Limpieza del panel de pagos — 2026-04-09

## Objetivo

Eliminar contenedores y textos redundantes en las secciones de pagos del panel sin alterar el flujo funcional.

## Cambios aplicados

- `SeccionPagos` ya no renderiza el wrapper `pagosDatos`.
- El espaciado del detalle se mueve a una clase de bloque (`pagosBloque`) aplicada a spinner, errores, tabla, vacíos y totales.
- `SeccionMetodosPago` elimina `metodosPagoIntro` para que la sección arranque directamente en el contenido útil.

## Criterio

- Si un contenedor solo existe para dar `gap`, preferir aplicar separación a los bloques reales cuando eso no complique el árbol.
- Si un texto introductorio no aporta contexto, instrucción ni estado, no debe ocupar espacio en la UI.

## Validación

- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`