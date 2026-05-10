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

### 105A-16 — Dev real con `npm run dev`
- La raíz de `coolify-manager-rs` debe abrir Tauri con `npm run dev`.
- El preview web queda separado como `npm run dev:web` para evitar confundir demo con operación real.

### 105A-17 — Acciones por menú contextual
- Las acciones por fila viven en un menú contextual único.
- Acciones activas: abrir sitio, verificar estado, copias, registros, copia manual, reinicio y redespliegue protegido.

### 105A-18 — Navegación visible y útil
- Quitar Inventario/Operaciones.
- Activar Panel, Sitios, Copias y Ajustes como pestañas reales.

### 105A-19 — Última verificación legible
- Reemplazar timestamps iguales por texto relativo y semántico.

### 105A-20 — Español completo
- Traducir navegación, tablas, estados, acciones y paneles.

### 105A-21 — Panel VPS
- Mostrar VPS configurados, selector de target, carga, RAM, disco, Docker, seguridad y recomendaciones.

### 105A-22 — CPU/RAM por despliegue
- Exponer métricas reales por stack desde Docker (`docker stats --no-stream`) y pintarlas en la tabla.

### 105A-23 — VPS reales configuradas
- Corregir la segunda VPS: `standby-vps2`, `173.249.50.44`, `http://173.249.50.44:8000`.
- Evitar que la GUI use targets fake cuando se abre sin Tauri.

### 105A-24 — Navegador real sin Tauri
- Añadir `gui-api` local en Rust para que Vite consuma la misma API real que Tauri.
- `npm run dev:web` debe levantar API local + Vite.
- Demo solo se permite con `VITE_COOLIFY_MANAGER_DEMO=1`.

### 105A-25 — Copias como tabla general
- Sustituir la vista por sitio por una tabla global equivalente a Sitios.
- Incluir sitio, dominio, VPS, id, tipo, estado, fecha, etiqueta y artefactos.

### 105A-26 — Menús contextuales sin recorte
- Mover los menús de acciones a portal/fixed para que no queden cortados por `panelTabla`.

### 105A-27 — RAM/CPU real en todas las vistas
- Confirmar que Dashboard y tabla de Sitios llaman API/Tauri real, no datos demo silenciosos.

### 105A-28 — Optimización y caché
- Mostrar la tabla de sitios sin esperar health-check secuencial.
- Cachear lecturas caras en `gui-api` y permitir refresco forzado.
- Acelerar backups globales evitando reconstruir cliente remoto por sitio.

### 105A-29 — Selector VPS global en sidebar
- Mover el cambio de VPS a `logoSidebar`.
- Usar componente personalizado, no select nativo.

### 105A-30 — Sentinel para select nativo
- Detectar `<select>` en React y recomendar componente base personalizado.

### 105A-31 — Agregar sitios como modal
- Convertir `Agregar sitio` en modal funcional con validación y feedback.

### 105A-32 — Retirar rutaPagina
- Quitar `rutaPagina` de las vistas y ajustar jerarquía visual.

### 105A-33 — Favicons inline
- Usar favicon real por dominio en filas de sitios, con fallback seguro.

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
- Fase 2: completada. `npm run dev` abre Tauri real, la navegación quedó en Panel/Sitios/Copias/Ajustes, hay métricas por despliegue y las acciones contextuales usan comandos reales con confirmaciones.
- Corrección 105A-23..27: completada. El navegador pasa por `gui-api`, Copias es tabla global y los menús salen del contenedor de tabla.
- 105A-28: completada. Sitios renderiza antes de los health-checks, backups globales reutilizan cliente remoto y GUI/API cachean lecturas caras con refresco forzado.
- Siguiente bloque 105A-29..33: selector VPS global, regla Sentinel para select nativo, modal de sitio, limpieza visual y favicons.
- Fase 3: pendiente por alcance multi-repo. La GUI ya usa `Button/IconButton` local como puente hacia componentes compartidos en `glory-rs`.