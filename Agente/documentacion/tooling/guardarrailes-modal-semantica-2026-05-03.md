# Guardarrailes de semantica compartida

Fecha: 2026-05-03

## Principio base

- Toda pieza compartida debe partir de una receta base del sistema antes de crear una variante local.
- Modales, paneles, cards, tablas, formularios, headers, footers y estados vacios siguen la misma regla.

## Primer caso cubierto

- `.modalTitulo`
- `.modalTexto`
- `.modalAcciones`

## Segundo caso cubierto

- `.modalFormulario`
- `.modalCampo`
- Helpers `ModalBody` y `ModalField` en `frontend/src/components/ui/Modal.tsx`

## Regla Sentinel implementada en este bloque

- `modal-semantica-no-canonica` detecta clases como `.ordenDetalleModalTexto`, `.usuariosModalTexto` o `.modalCompraDescripcion` cuando redefinen tipografia, color, alineacion o layout semantico que ya pertenece al sistema compartido de modales.
- `modal-estructura-no-canonica` detecta cuerpos o campos locales como `.hostingFormCrear`, `.agregarTarjetaFormulario` o `.usuariosCrearCampo` y fuerza el uso de `modalFormulario` / `modalCampo` o de los helpers del componente base.
- Si un componente necesita un ajuste puntual, la clase local debe quedarse solo con spacing o estado, no con la receta visual base.
- Estas reglas cubren ya semantica y estructura; el principio debe extenderse al resto de componentes compartidos del sistema.

## Tokens

- `--border-default` queda normalizado en `#dcdcdc`.
- `--bg-item-active` no debe reutilizarse como borde generico cuando el borde no representa un estado activo, hover o seleccionado.