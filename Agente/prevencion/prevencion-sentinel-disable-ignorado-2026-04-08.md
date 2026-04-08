# Falso positivo: sentinel-disable-file no suprime detección componente-sin-hook

## Contexto
Los componentes `SubTabServicios.tsx`, `SubTabBlog.tsx`, `SubTabProyectos.tsx` y `SubTabEquipo.tsx` tienen el comentario:
```
/* [084A-22] ... 
 * sentinel-disable-file componente-sin-hook: Los callbacks son wiring trivial que
 * delega en useContenido* hooks existentes. No justifica un hook intermedio. */
```

## Problema
Code Sentinel sigue reportando "Componente con N lineas de logica. Extraer a hook dedicado" a pesar del `sentinel-disable-file`.

## Corrección necesaria en Code Sentinel
El analyzer de React (`reactAnalyzer.ts` o `react/componentLogicDetector.ts`) debe:
1. Leer el archivo buscando patrones `sentinel-disable-file {regla}` en comentarios
2. Si la regla deshabilitada coincide con la detección a reportar, omitirla
3. El match debe ser por nombre de regla (`componente-sin-hook`) no por texto exacto

## Estado
Pendiente de implementar en `.agent/code-sentinel`.
