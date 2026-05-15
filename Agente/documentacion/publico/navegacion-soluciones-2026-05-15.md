# Navegacion publica de soluciones — 2026-05-15

## Estado

- `/soluciones` y `/soluciones/` no son paginas navegables; deben resolver a 404.
- Las paginas validas son `/soluciones/hosting-wordpress`, `/soluciones/hosting` y `/soluciones/vps`.
- La navegacion principal agrupa Soluciones como menu contextual sin `href` propio.
- Servicios tambien usa menu contextual para exponer nombres de servicios sin cargar una pagina intermedia.
- Las imagenes hero de Hosting/VPS usan el mismo ancho, padding y ratio visual que `galeriaHeroContenedor` de home.
- El boton admin de tres puntos permite cambiar la imagen de forma local como placeholder hasta conectar persistencia CMS/backend.

## Validacion local

- Desde 2026-05-15, `127.0.0.1` y `localhost` renderizan la home principal de Nakomi. El portal VPS no se previsualiza desde este bundle.
- Validar que `/soluciones` muestra 404 y que las tres subrutas reales renderizan sus paginas.
