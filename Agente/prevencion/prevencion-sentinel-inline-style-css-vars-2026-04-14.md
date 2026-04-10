# Falso positivo: inline-style-prohibido con CSS custom properties

## Problema
La regla `inline-style-prohibido` de Code Sentinel detecta `style={{}}` como CSS inline prohibido.
Sin embargo, usar `style={{ '--my-var': value } as CSSProperties}` es el patrón estándar de React
para pasar CSS custom properties dinámicas (no es CSS inline real).

## Archivo afectado
- `frontend/src/components/panel/HostingStats.tsx` (líneas 37 y 59)
- Patrón: `style={{'--hosting-bar-width': '...'} as CSSProperties}`

## Solución propuesta
En la regla `inline-style-prohibido` del analizador React de Code Sentinel, añadir una excepción:
si el contenido del `style={{}}` solo contiene CSS custom properties (keys que empiezan con `--`),
no es una violación. El regex o AST check debería verificar que las keys del objeto son strings
que comienzan con `--`.

## Ubicación del código a modificar
`code-sentinel/src/analyzers/react/reactCssRules.ts` — regla `inline-style-prohibido`
