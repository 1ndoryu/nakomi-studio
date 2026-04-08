# Plan: Auditoría SOLID Frontend — 084A-22

## Hallazgos

### Archivos sobre límite de líneas

#### CSS (máx 300 líneas)

| Archivo                  | Líneas | Exceso | Acción                |
| ------------------------ | ------ | ------ | --------------------- |
| SeccionProyectos.css     | 632    | +332   | Dividir en 4 archivos |
| SeccionChat.css          | 371    | +71    | Dividir bubbles       |
| UsuarioPublicoIsland.css | 370    | +70    | Dividir reviews       |
| SeccionUsuarios.css      | 349    | +49    | Dividir tabla         |
| SeccionPagos.css         | 315    | +15    | Tolerable             |
| SeccionHosting.css       | 298    | -2     | OK                    |

#### Componentes (máx 300 líneas)

| Archivo              | Líneas | useState | Acción                      |
| -------------------- | ------ | -------- | --------------------------- |
| SeccionContenido.tsx | 344    | 9        | Extraer 4 SubTab components |

#### Hooks (máx 120 líneas)

| Archivo              | Líneas | Exceso | Acción                            |
| -------------------- | ------ | ------ | --------------------------------- |
| useChat.ts           | 192    | +72    | Separar useChat + useChatWs       |
| useChatWidget.ts     | 168    | +48    | Cohesivo, tolerable (1 propósito) |
| useEditorServicio.ts | 167    | +47    | Form state, tolerable             |
| useNotifications.ts  | 149    | +29    | Separar hooks + utils             |
| useAutenticacion.ts  | 138    | +18    | Cohesivo, tolerable               |

#### Data/Utils (máx 150 líneas)

| Archivo                | Líneas | Notas                               |
| ---------------------- | ------ | ----------------------------------- |
| planes.ts              | 422    | Datos estáticos (sentinel-disabled) |
| contentTranslations.ts | 246    | Datos i18n, no refactorizable       |

### Violaciones SOLID

1. **SRP — SeccionContenido.tsx**: Maneja 4 tipos de contenido CMS en un solo componente. 9 useState. Cada sub-tab debería ser su propio componente.
2. **SRP — useChat.ts**: Contiene 2 hooks independientes (REST + WebSocket) en un archivo.
3. **SRP — useNotifications.ts**: 2 hooks + 3 funciones utilitarias en un archivo.
4. **SRP — SeccionProyectos.css**: Mezcla estilos de lista, cards, detalle, timeline, filtros en 632 líneas.

### Patrones duplicados (no detectados por Sentinel)

- Spinner de carga (`.proyectosSpinner`, `.usuariosSpinner`, etc.) — código CSS idéntico
- Estados vacíos con ícono + texto — misma estructura en múltiples componentes
- Badges de estado — renderizado repetido con clases CSS similares

## Ejecución (por prioridad)

1. ✅ Extraer SubTabServicios, SubTabBlog, SubTabProyectos, SubTabEquipo de SeccionContenido.tsx
2. ✅ Separar useChat.ts → useChat.ts + useChatWs.ts
3. ✅ Separar useNotifications.ts → useNotifications.ts + useNotificationWs.ts + notificationUtils.ts
4. ✅ Dividir SeccionProyectos.css → SeccionProyectos.css + OrdenCard.css + OrdenDetalle.css + FasesTimeline.css
5. ✅ Dividir SeccionChat.css → SeccionChat.css + ChatBurbujas.css
6. ✅ Dividir UsuarioPublicoIsland.css → UsuarioPublicoIsland.css + PerfilReviews.css
7. ✅ Dividir SeccionUsuarios.css → SeccionUsuarios.css + UsuariosTabla.css
8. Documentar hallazgos menores como mejoras futuras
