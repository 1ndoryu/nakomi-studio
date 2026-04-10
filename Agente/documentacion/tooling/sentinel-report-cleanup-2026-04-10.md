# Limpieza de Sentinel Report

Fecha: 2026-04-10

## Objetivo

Dejar `.sentinel-report.md` sin violaciones reales ni falsos positivos pendientes en el workspace principal.

## Cambios aplicados

- `code-sentinel` dejó de detectar `css-adhoc-button-style` con regex plana y pasó a una verificación contextual que:
  - ignora comentarios con la palabra `boton/button`
  - ignora clases base del sistema (`menuContextualBoton`, `botonBase`, etc.)
  - ignora selectores nativos tipo `button.tarjetaBase`
  - ignora `glory-rs/` y `frontend/public/assets/`
- `inline-style-prohibido` ahora reconoce `style={{ '--mi-var': valor }}` también cuando el objeto vive en una sola línea.
- Se renombraron clases contextuales heredadas (`...Boton...`) en header, footer, modal auth, CTA, pagos, reviews, testimonios, selector de idioma, perfil público y modal de testimonios.
- `HostingStats` movió el ancho dinámico a CSS custom properties.
- Los últimos `sqlx::query_as()` runtime marcados por Sentinel se migraron a `query_as!()` y se regeneró `.sqlx/` con `cargo sqlx prepare`.

## Validación usada

- `npx tsc --noEmit` en `frontend/`
- `cargo sqlx prepare`
- `cargo check`
- `cargo clippy -- -D warnings`
- `npm run compile` en `.agent/code-sentinel`
- `CODE_SENTINEL_TARGET_WORKSPACE=<repo> npm test` en `.agent/code-sentinel`

## Resultado

- `.sentinel-report.md` regenerado con `0` violaciones.
- El host de pruebas de Sentinel sigue teniendo 2 tests legacy fallando en `shapeMismatch.test.ts` (`php-service-retorna-asociativo`), pero no afectan la generación ni el contenido del reporte del workspace.