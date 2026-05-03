# Guardarrailes de semantica modal

Fecha: 2026-05-03

## Canonico

- `.modalTitulo`
- `.modalTexto`
- `.modalAcciones`

## Regla Sentinel

- `modal-semantica-no-canonica` detecta clases como `.ordenDetalleModalTexto`, `.usuariosModalTexto` o `.modalCompraDescripcion` cuando redefinen tipografia, color, alineacion o layout semantico que ya pertenece al sistema compartido de modales.
- Si un componente necesita un ajuste puntual, la clase local debe quedarse solo con spacing o estado, no con la receta visual base.

## Tokens

- `--border-default` queda normalizado en `#dcdcdc`.
- `--bg-item-active` no debe reutilizarse como borde generico cuando el borde no representa un estado activo, hover o seleccionado.