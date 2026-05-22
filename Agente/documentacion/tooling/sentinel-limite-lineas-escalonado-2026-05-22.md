# Sentinel limite-lineas escalonado — 2026-05-22

## Qué cambió

`limite-lineas` dejó de ser el único guardarrail para archivos grandes. Code Sentinel ahora reporta niveles adicionales cuando un archivo duplica, triplica o quintuplica el límite de su capa:

- `limite-lineas`: excede el límite base.
- `limite-lineas-nivel-2`: supera 2x el límite.
- `limite-lineas-nivel-3`: supera 3x el límite.
- `limite-lineas-nivel-4`: supera 5x el límite.

## Por qué

Un único `sentinel-disable-file limite-lineas` permitía silenciar archivos que seguían creciendo sin control. Los niveles nuevos tienen IDs independientes: desactivar el primer aviso no oculta los niveles graves.

## Validación

- `npm run compile` en `.agent/code-sentinel`.
- `npx mocha --no-config --ui tdd --require out/test/registerMocks.js out/test/suite/sentinelDisableFile.test.js out/test/suite/lineCounter.test.js`.
