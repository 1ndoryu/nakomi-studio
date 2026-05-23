# Recursos hosting — sampler, bandwidth y capacidad

## Estado

Implementado en `225A-4`.

## Arquitectura

- `infrastructure_servers` guarda inventario no sensible de servidores Coolify configurados. Tokens y SSH keys quedan como `secret_ref`/env, no como secretos en BD.
- `infrastructure_metrics_loop` corre cada 10 minutos y hace una sola SSH por servidor activo para capturar CPU/RAM/disco VPS y `docker stats` de todos los contenedores.
- `infrastructure_resource_samples` retiene muestras por 7 dias; los endpoints de dashboard leen esta tabla y no ejecutan SSH en cada render.
- El disco por despliegue usa `du -sm /var/lib/docker/volumes/*/_data` solo en ventana horaria para evitar coste innecesario.
- `bandwidth_snapshots` guarda el ultimo contador Docker por suscripcion/despliegue. `bandwidth_usage` acumula deltas mensuales por `subscription_id + month_start`.
- `bandwidth_enforcement_loop` corre cada hora. Si una suscripcion activa supera `bandwidth_limit_gb`, detiene el servicio Coolify y marca `suspended_bandwidth`; si vuelve a estar bajo limite o se reinicia el ciclo mensual, restaura `active`.
- `server_capacity` guarda specs y asignacion. Provisioning reserva capacidad con update atomico; si no hay specs conocidas aun, no bloquea el alta.
- `vps_monitor_loop` consulta Contabo cada hora para estado proveedor de VPS vendidas, sin usarlo para metricas de CPU/RAM/disco usados.

## Endpoints admin

- `GET /api/infrastructure/servers`
- `GET /api/infrastructure/deployments`
- `GET /api/infrastructure/deployments/:uuid/metrics?range=24h`
- `POST /api/infrastructure/metrics/refresh`
- `GET /api/infrastructure/resource-report`

Los endpoints legacy de hosting siguen disponibles como alias donde aplica.

## Validaciones

- `cargo fmt --check`
- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm run check:front`
- `npm --prefix frontend run build`

## Gotchas

- El primer sample de CPU VPS no tiene delta valido; se guarda referencia y el siguiente ciclo ya puede calcular porcentaje.
- Docker `NetIO` es acumulado desde arranque del contenedor. La primera muestra o un delta negativo por restart no se suma como uso mensual.
- El panel puede mostrar guiones hasta que el sampler tenga la primera muestra; eso es preferible a abrir SSH en render.
- `bandwidth_limit_gb = -1` significa ilimitado y el enforcement lo ignora.