# Guardarrailes de semantica compartida

Fecha: 2026-05-03

## Principio base

- Toda pieza compartida debe partir de una receta base del sistema antes de crear una variante local.
- Modales, paneles, cards, tablas, formularios, headers, footers y estados vacios siguen la misma regla.

## Primer caso cubierto

- `.modalTitulo`
- `.modalTexto`
- `.modalAcciones`

## Regla Sentinel implementada en este bloque

- `modal-semantica-no-canonica` detecta clases como `.ordenDetalleModalTexto`, `.usuariosModalTexto` o `.modalCompraDescripcion` cuando redefinen tipografia, color, alineacion o layout semantico que ya pertenece al sistema compartido de modales.
- Si un componente necesita un ajuste puntual, la clase local debe quedarse solo con spacing o estado, no con la receta visual base.
- Esta regla resuelve el primer frente visible; el principio debe extenderse al resto de componentes compartidos del sistema.

## Tokens

- `--border-default` queda normalizado en `#dcdcdc`.
- `--bg-item-active` no debe reutilizarse como borde generico cuando el borde no representa un estado activo, hover o seleccionado.