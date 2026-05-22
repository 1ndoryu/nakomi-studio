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

El documento historico de abril asumia Cloud VPS 2 con 6 vCPU, 16 GB RAM y 400 GB. El target actual tiene 4 vCPU, 8 GB RAM y 290 GB. Por tanto las densidades viejas (5 basicos / 3 pro / 2 avanzado con margen 20%) son demasiado optimistas para este nodo si usamos limites de CPU estrictos.

## Capacidad conservadora por limites actuales

Presupuesto recomendado para hosting en este nodo:

- Reservar ~1 vCPU para Coolify, Traefik, PostgreSQL de Coolify, sistema y picos.
- Reservar ~2 GB RAM para sistema/Coolify/headroom.
- No vender por encima de 70% de disco si los backups quedan en el mismo VPS.

Densidad por plan si el nodo se llena con un solo tipo:

| Familia        | Plan     | Limite por hosting         | Cap CPU conservadora | Cap RAM conservadora | Cap disco sin backups | Recomendacion                      |
| -------------- | -------- | -------------------------- | -------------------- | -------------------- | --------------------- | ---------------------------------- |
| WordPress      | basico   | 1.0 CPU / 640 MB / 5 GB    | 3                    | 9                    | 40+                   | 3 activos por nodo                 |
| WordPress      | pro      | 2.0 CPU / 1280 MB / 20 GB  | 1                    | 4                    | 10+                   | 1-2, mejor 1 si hay trafico real   |
| WordPress      | avanzado | 2.75 CPU / 1792 MB / 50 GB | 1                    | 3                    | 4                     | 1 por nodo si no hay backup remoto |
| Hosting normal | basico   | 0.75 CPU / 384 MB / 5 GB   | 4                    | 15                   | 40+                   | 4 activos por nodo                 |
| Hosting normal | pro      | 1.5 CPU / 768 MB / 20 GB   | 2                    | 7                    | 10+                   | 2 activos por nodo                 |
| Hosting normal | avanzado | 2.0 CPU / 1280 MB / 50 GB  | 1                    | 4                    | 4                     | 1 activo por nodo                  |

La RAM real esta muy holgada; el cuello de botella serio es CPU vendida por limites y disco si las copias automaticas viven en el mismo VPS.

## Rentabilidad con precios actuales

Coste historico usado para pricing: Cloud VPS 2 ~`$9.90/mes`. Si este nodo actual cuesta menos, el margen real mejora; si cuesta igual, el margen empeora frente al plan original porque la capacidad medida es menor.

Ingresos por nodo con densidad conservadora:

| Mix ideal                                      | Ingreso mensual | Margen vs $9.90 | Lectura                                       |
| ---------------------------------------------- | --------------: | --------------: | --------------------------------------------- |
| 3 WordPress basico (`$2.48`)                   |         `$7.44` |        negativo | No cubre coste historico si solo hay basicos. |
| 1 WordPress pro (`$4.13`) + 1 basico (`$2.48`) |         `$6.61` |        negativo | No rentable con el supuesto viejo.            |
| 2 WordPress pro                                |         `$8.26` |        negativo | Mejor, pero aun bajo si el nodo cuesta $9.90. |
| 1 avanzado + 1 basico                          |         `$8.67` |        negativo | CPU justa y disco/backups delicados.          |
| 4 hosting normal basico (`$3.23`)              |        `$12.92` |            ~23% | Rentable si son sitios livianos.              |
| 2 hosting normal pro (`$5.37`)                 |        `$10.74` |             ~8% | Rentabilidad baja.                            |

Conclusion comercial: con 4 vCPU/8 GB, los precios WordPress actuales son demasiado baratos para margen estable si se respetan limites conservadores y backups locales. Hosting normal basico si puede sostener el margen por consumo real bajo.

## Backups y disco

El sidecar nuevo crea backups para todos los planes nuevos:

- Basico: backup semanal.
- Pro/Avanzado: backup diario + copia semanal.
- WordPress: archivos + MariaDB.
- Hosting normal: archivos del sitio.

Riesgo: esos backups viven en `backup-data` del mismo VPS. Si un plan avanzado usa 50 GB y retiene varios backups locales, un solo cliente puede consumir gran parte del disco. Para vender planes grandes con seguridad hace falta una de estas medidas antes de escalar:

1. backup remoto a otro VPS/storage externo;
2. cuota de backups por plan;
3. retencion menor para planes grandes mientras no haya storage externo;
4. calculo de capacidad que reserve multiplicador de backups.

## Recomendaciones operativas

- No llenar este VPS2 con WordPress barato; usarlo para pruebas, hosting normal liviano y pocos WordPress basicos.
- Umbral para contratar otro nodo: CPU load sostenido > 2.5, RAM disponible < 2 GB, disco usado > 65%, o 3 WordPress activos aunque el uso parezca bajo.
- Para WordPress Pro/Avanzado, mover a un nodo de 6 vCPU/16 GB o superior si van a ser clientes reales.
- Activar `ufw`/`fail2ban` o documentar por que Coolify/Traefik cubren ese riesgo; hoy la auditoria los marca inactivos.
- Implementar backup remoto antes de prometer planes avanzados con retencion diaria seria.
- Refrescar hostings existentes tras deploy si se quiere que reciban sidecar `backup-data`; los stacks actuales no lo tienen.
