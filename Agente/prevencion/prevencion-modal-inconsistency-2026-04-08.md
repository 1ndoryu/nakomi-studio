# Prevención: Inconsistencia de modales

**Fecha:** 2026-04-08
**Tarea:** 084A-8

## Problema detectado
Modales usando clases CSS custom para títulos y acciones en vez de las clases estándar `.modalTitulo` y `.modalAcciones` definidas en `Modal.css`. Esto causa inconsistencia visual entre modales.

## Regla a implementar en Code Sentinel

### `modal-inconsistency`
- **Severidad:** warning
- **Detección en TSX:** Dentro de un `<Modal>` (o hijo de Modal), detectar:
  1. Elementos `<h2>` o `<h3>` con `className` que NO sea `modalTitulo` → sugerir usar `.modalTitulo`
  2. Divs con `className` que contenga "Acciones" o "acciones" que NO sea `modalAcciones` → sugerir usar `.modalAcciones`
  3. Componentes que rendericen overlay custom (`position: fixed; inset: 0`) en vez de usar `<Modal>` base
- **Detección en CSS:** Clases con patrón `.*ModalTitulo` o `.*ModalAcciones` que dupliquen estilos definidos en `.modalTitulo`/`.modalAcciones`

### Modales excluidos
- CMS editors (EditorServicio, EditorProyecto, EditorBlog) usan `.modalSinPadding` con tabs propios — excluir de la regla de títulos
