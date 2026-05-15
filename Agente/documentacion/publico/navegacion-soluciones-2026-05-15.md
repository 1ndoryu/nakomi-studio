# Navegacion publica de soluciones — 2026-05-15

## Estado

- `/soluciones` y `/soluciones/` no son paginas navegables; deben resolver a 404.
- Las paginas validas son `/soluciones/hosting` y `/soluciones/vps`.
- La navegacion principal muestra enlaces directos a Hosting y Servidores VPS.
- Las imagenes hero de Hosting/VPS usan el mismo ancho, padding y ratio visual que `galeriaHeroContenedor` de home.
- El boton admin de tres puntos permite cambiar la imagen de forma local como placeholder hasta conectar persistencia CMS/backend.

## Validacion local

- En desarrollo, `127.0.0.1:5173` esta reservado por una regla previa para previsualizar el portal VPS.
- Para validar la home principal de Nakomi usar `http://localhost:5173/`.
