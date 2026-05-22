# VPS2 hosting - capacidad y rentabilidad (2026-05-21)

## Evidencia medida

Target operativo: `standby-vps2` (`173.249.50.44`).

- CPU: 4 vCPU (`AMD EPYC`, 1 thread/core).
- RAM: 7941 MB total, 1449 MB usados, 6492 MB disponibles.
- Disco: 290 GB total, 21 GB usados, 269 GB disponibles.
- Docker: 11 contenedores activos, imagenes 2.767 GB, volumenes locales 355.4 MB.
- Seguridad base: `ufw` inactive, `fail2ban` inactive segun auditoria del manager.
- Load average auditado: `1.14 1.35 1.41`.

Contenedores de hosting actuales:

| Stack                      | Tipo                  | Contenedores              | Limites CPU/RAM   | Uso observado           |
| -------------------------- | --------------------- | ------------------------- | ----------------- | ----------------------- |
| `e1m95ycgcc72zjiles3mov3c` | WordPress basico      | wordpress + mariadb + ssh | 1.0 CPU / 640 MB  | ~289 MB RAM, CPU ~0.02% |
| `mhjskh5eh2nzovdaorgqo6pa` | Hosting normal basico | site + ssh                | 0.75 CPU / 384 MB | ~8 MB RAM, CPU ~0%      |

Volumenes actuales:

- `e1m95ycgcc72zjiles3mov3c_wordpress-data`: 99 MB.
- `e1m95ycgcc72zjiles3mov3c_mariadb-data`: 172 MB.
- `mhjskh5eh2nzovdaorgqo6pa_site-data`: 12 KB.
- No existen `backup-data` en esos stacks todavia; fueron provisionados antes del sidecar de backup universal.

## Correccion frente al plan viejo

El documento historico de abril asumia Cloud VPS 2 con 6 vCPU, 16 GB RAM y 400 GB. El target actual sigue siendo mas pequeno: 4 vCPU, 8 GB RAM y 290 GB. Eso invalida el supuesto viejo de hardware, pero la primera version de este informe tambien fue demasiado conservadora al extrapolar solo desde limites nominales de CPU/RAM.

Con las pruebas manuales reportadas por el usuario, este nodo aguanta bastante mas carga real de la que sugeria aquella tabla: se probaron varios sitios normales y hasta 7 WordPress complejos concurrentes funcionando bien. Por eso la capacidad comercial razonable debe ajustarse al alza, manteniendo vigilancia de disco y backups.

## Capacidad operativa ajustada por pruebas reales

Presupuesto recomendado para hosting en este nodo:

- Reservar ~1 vCPU para Coolify, Traefik, PostgreSQL de Coolify, sistema y picos.
- Reservar ~2 GB RAM para sistema/Coolify/headroom.
- No vender por encima de 70% de disco si los backups quedan en el mismo VPS.
- El cuello principal pasa a ser disco + retencion de backups; la RAM sigue muy holgada en el uso observado.

Densidad por plan usando las pruebas reales como baseline operativo:

| Familia        | Plan     | Limite por hosting         | Tabla anterior | Capacidad operativa ajustada | Lectura                                                                 |
| -------------- | -------- | -------------------------- | -------------- | ---------------------------- | ----------------------------------------------------------------------- |
| WordPress      | basico   | 1.0 CPU / 640 MB / 5 GB    | 3              | 6                            | Baseline comercial razonable con el uso real observado.                 |
| WordPress      | pro      | 2.0 CPU / 1280 MB / 20 GB  | 1-2            | 3                            | Aguanta bastante mejor que lo que sugerian los limites nominales.       |
| WordPress      | avanzado | 2.75 CPU / 1792 MB / 50 GB | 1              | 1-2 segun mix                | Mejor venderlo mezclado con 2 basicos o con disco/backups muy vigilados. |
| Hosting normal | basico   | 0.75 CPU / 384 MB / 5 GB   | 4              | 8                            | Su consumo real sigue siendo muy bajo; el limite practico no es la RAM. |
| Hosting normal | pro      | 1.5 CPU / 768 MB / 20 GB   | 2              | 4                            | Densidad razonable mientras el disco siga controlado.                   |
| Hosting normal | avanzado | 2.0 CPU / 1280 MB / 50 GB  | 1              | 2                            | Factible, pero el disco manda antes que CPU/RAM.                        |

Mixes que cuadran mejor con las pruebas del usuario:

- 6 WordPress basicos.
- 1 WordPress Pro + 4 basicos.
- 3 WordPress Pro.
- 1 WordPress Avanzado + 2 basicos.
- Hosting normal: tomar como referencia el doble de la tabla inicial, es decir, 8 basicos o 4 Pro si el disco y los backups siguen bajo control.

## Rentabilidad con coste real del nodo

Coste real reportado para este VPS: `$7.44/mes`.

Ingresos por nodo con la densidad operativa ajustada:

| Mix operativo                                   | Ingreso mensual | Margen vs $7.44 | Lectura                                                     |
| ----------------------------------------------- | --------------: | --------------: | ----------------------------------------------------------- |
| 6 WordPress basico (`$2.48`)                    |        `$14.88` |          `100%` | El basico si cubre bien el nodo con el coste real actual.   |
| 1 WordPress Pro (`$4.13`) + 4 basicos (`$2.48`) |        `$14.05` |           `89%` | Mezcla solida para vender sin infrautilizar el VPS.         |
| 3 WordPress Pro (`$4.13`)                       |        `$12.39` |           `67%` | Buen equilibrio entre ingreso y complejidad operativa.      |
| 1 Avanzado (`$6.19`) + 2 basicos (`$2.48`)      |        `$11.15` |           `50%` | Rentable, pero mas sensible a disco y backups.              |
| 8 hosting normal basico (`$3.23`)               |        `$25.84` |          `247%` | Hosting normal sigue siendo el mix mas holgado del nodo.    |
| 4 hosting normal pro (`$5.37`)                  |        `$21.48` |          `189%` | Tambien muy rentable mientras el disco no sea el cuello.    |

Conclusion comercial: con coste real de `$7.44/mes`, este VPS2 si es rentable como nodo comercial inicial. El cuello operativo deja de ser CPU/RAM y pasa a ser, sobre todo, el disco y la retencion local de backups.

## Backups y disco

El sidecar nuevo crea backups para todos los planes nuevos:

- Basico: backup semanal.
- Pro/Avanzado: backup diario + copia semanal.
- WordPress: archivos + MariaDB.
- Hosting normal: archivos del sitio.
- Retencion local objetivo: maximo 5 backups por hosting.

Riesgo: esos backups viven en `backup-data` del mismo VPS. Incluso con la capacidad revisada al alza, si un plan avanzado usa 50 GB y se le dejan demasiadas copias locales, un solo cliente puede comerse gran parte del disco. Para que las densidades anteriores sigan siendo realistas hace falta una de estas medidas antes de escalar:

1. limite duro de 5 backups por hosting;
2. backup remoto a otro VPS/storage externo;
3. retencion menor para planes grandes mientras no haya storage externo;
4. calculo de capacidad que reserve multiplicador de backups.

## Recomendaciones operativas

- Este VPS2 si puede funcionar como nodo comercial inicial para hosting compartido si se usa como baseline 6 WordPress basicos, 3 Pro o mezclas equivalentes como 1 Avanzado + 2 basicos.
- En hosting normal, la referencia razonable es 8 basicos o 4 Pro mientras el disco siga controlado.
- Umbral para contratar otro nodo: CPU load sostenido > 2.5, RAM disponible < 2 GB, disco usado > 65%, o al acercarse a 6 WordPress basicos / 3 Pro / 1 Avanzado + 2 basicos.
- Estas cifras deben tratarse como capacidad operativa basada en pruebas reales, no como SLA fijo; si el perfil de clientes cambia, hay que re-medirlo.
- Activar `ufw`/`fail2ban` o documentar por que Coolify/Traefik cubren ese riesgo; hoy la auditoria los marca inactivos.
- Implementar cuanto antes el limite de 5 backups por hosting y, despues, backup remoto para no convertir el disco en el cuello principal.
- Refrescar hostings existentes tras deploy si se quiere que reciban sidecar `backup-data`; los stacks actuales no lo tienen.
