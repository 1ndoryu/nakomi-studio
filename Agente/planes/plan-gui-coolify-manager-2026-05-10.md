# Plan GUI coolify-manager-rs — 2026-05-10

## Contexto

El diseño actual de la GUI no alcanza el nivel esperado: se ve como panel técnico temporal, no como consola operativa. La referencia visual deseada es una app ultra limpia tipo tabla/CRM: sidebar estrecho, topbar funcional, lista densa, acciones por registro, estado visible y poco ruido.

## Tareas anotadas

### 105A-9 — Plan de rediseño profundo
- Registrar todas las decisiones pedidas por el usuario antes de tocar más código.
- Separar trabajo ejecutable inmediato de la migración grande a componentes compartidos.

### 105A-10 — Modo navegador sin bloqueo de Tauri
- La GUI debe abrir en navegador sin mostrar “necesita Tauri”.
- Si no hay runtime nativo, usar un cliente browser-safe con datos locales/demo y estados explícitos.
- Mantener el runtime Tauri para operaciones reales sin romper el preview web.

### 105A-11 — Alcance de herramientas Glory en coolify-manager-rs
- Asegurar que Sentinel/VarSense/Glory puedan analizar `coolify-manager-rs` en el workspace multi-root.
- Documentar cómo ejecutar análisis sobre el repo hermano sin depender de abrir otro VS Code.

### 105A-12 — Rediseño visual tipo tabla/CRM
- Sidebar compacto similar a la referencia.
- Vista principal de sitios como tabla densa con toolbar, contador, filtros simples y acciones por fila.
- Usar iconos `lucide-react`, el mismo proveedor de iconos que Nakomi Studio.
- Eliminar estados visuales que parecen error cuando solo es preview.

### 105A-13 — Modelo operativo real
- El botón “Health” suelto no tiene sentido; el estado debe vivir en cada fila.
- Cargar estado por sitio de forma periódica cuando Tauri esté disponible.
- Enlistar backups por sitio dentro de la experiencia principal.
- Preparar acciones por fila: abrir sitio, refrescar estado, ver backups, backup manual, logs, restart/redeploy/restore como acciones explícitas.

### 105A-14 — Retirar vistas sueltas
- Quitar navegación “Salud” y “Auditoría”.
- Mantener Backups como panel contextual/detalle, no como sección aislada que pide escribir nombre.

### 105A-15 — Centralizar componentes Nakomi en glory-rs
- Decisión arquitectónica: los componentes base reutilizables deben vivir en `glory-rs`, no copiados ad-hoc dentro de cada app.
- Trabajo grande posterior: extraer Button/Badge/Input/Table/Shell/Icon recipes de Nakomi a un paquete compartido, clonar/consumir `glory-rs` desde `coolify-manager-rs` y migrar la GUI a esos componentes.
- No bloquear la mejora inmediata de la GUI por esta migración; diseñar la GUI usando nombres y estructura compatibles con esa futura extracción.

## Fases

### Fase 1 — Ejecutable en este bloque
- Crear cliente de datos para GUI que soporte Tauri y navegador.
- Reemplazar la navegación por una sola consola de sitios.
- Rediseñar layout, tabla, toolbar, fila, estados y panel de backups.
- Usar `lucide-react` y eliminar el mensaje de Tauri requerido.
- Documentar el alcance Sentinel/VarSense sobre `coolify-manager-rs`.
- Validar con `npm --prefix gui run build`, `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test`.

### Fase 2 — Acciones nativas completas
- Exponer comandos Tauri/API para logs, backup manual, restart, redeploy y restore.
- Añadir confirmaciones para acciones destructivas.
- Registrar resultados como toasts/event log.

### Fase 3 — Componentes compartidos en glory-rs
- Crear paquete compartido frontend en `glory-rs` con componentes agnósticos.
- Migrar Nakomi a consumirlo sin romper su diseño actual.
- Consumir el paquete desde `coolify-manager-rs` por git/path y retirar duplicados locales.
- Ajustar Sentinel/VarSense para reconocer las recetas compartidas como sistema base.

## Estado

- Fase 1: completada. GUI table-first implementada, modo navegador demo activo, estado por fila, backups contextuales, `lucide-react`, vistas Salud/Auditoría retiradas y alcance Sentinel/VarSense corregido.
- Fase 2: pendiente. Acciones nativas destructivas requieren comandos Tauri, confirmaciones y registro de eventos antes de activarse.
- Fase 3: pendiente por alcance multi-repo. La GUI ya usa `Button/IconButton` local como puente hacia componentes compartidos en `glory-rs`.