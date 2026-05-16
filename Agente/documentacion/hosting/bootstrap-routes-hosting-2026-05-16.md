# Bootstrap Routes de Hosting

> Fecha: 2026-05-16

## Problema

Los hostings compose creados en Coolify dependían del FQDN implícito que devolvía la API al crear el servicio. En la práctica, el panel construía la URL bootstrap con `server_uuid` y no había labels Traefik explícitas en el compose, así que el `sslip.io` inicial podía responder `404 page not found` aunque el contenedor existiera.

## Decisión aplicada

- La URL bootstrap del panel ahora se deriva de `coolify_site_name`, que es el nombre estable del servicio persistido en la suscripción.
- Los compose de hosting normal y WordPress generan labels Traefik explícitas para el host bootstrap `site|wordpress-{coolify_site_name}.{server_ip}.sslip.io`.
- Cuando el cliente guarda o cambia su dominio, el backend regenera el compose con el dominio real adicional para que conviva con el host bootstrap.
- Los endpoints de refresh y rotación de credenciales regeneran el compose completo preservando esas rutas.

## Implicaciones operativas

- Los hostings ya provisionados antes de este cambio necesitan un refresh para recibir las labels nuevas.
- El dominio real sigue dependiendo de que el cliente apunte DNS al `server_ip` correcto.
- El host bootstrap queda disponible como fallback técnico aunque el dominio final todavía no propague.

## Incidente 165A-12: recompra caída en provisioning manualmente rescatado

- El hosting recomprado `hosting-2dad31af` no estaba roto por la URL bootstrap sino por dos fallos del compose generado: `pids_limit` entraba en conflicto con `deploy.resources.limits.pids` inyectado por Coolify y el Dockerfile inline del sidecar SSH seguía pineado a `lscr.io/linuxserver/openssh-server:9.9_p2-r0-ls190`, una tag ya retirada.
- Para rescatar el stack ya creado sin esperar una nueva provisión hubo que editar el `docker-compose.yml` on-disk en `/data/coolify/services/{uuid}`: quitar `pids_limit`, cambiar la tag a `version-9.9_p2-r0` y volver a ejecutar `docker compose up -d`.
- Al levantar un servicio manualmente desde el compose on-disk, Coolify no vuelve a cablear automáticamente el proxy. Si la URL pública queda en timeout pero el contenedor responde 200 internamente, conectar `coolify-proxy` a la red `caddy_ingress_network` indicada por las labels del servicio (`docker network connect {stack_uuid} coolify-proxy`).
- El rescate dejó operativo el hosting normal ya comprado, pero la expectativa del usuario era WordPress. Por eso el CTA del panel se redirigió desde la landing genérica a `/soluciones/hosting-wordpress/` para futuras compras.

## Actualización 165A-13+165A-14: WordPress realmente preinstalado

- El incidente siguiente mostró una segunda brecha: aunque el compose y la URL bootstrap ya funcionaban, el producto seguía entregando solo el wizard de WordPress. Eso no cumple la promesa comercial de "WordPress preinstalado".
- El provisioning ahora espera el formulario real de `wp-admin/install.php?step=1` y completa `install.php?step=2` con el nombre del cliente, su email y las credenciales iniciales del hosting. Si el wizard no confirma el login final, el alta del stack queda como creada pero `wordpress_ready = false` y `wordpress_install_error` se expone en eventos para auditoría.
- La tab Acceso del panel muestra esas credenciales iniciales de WordPress reutilizando las del SFTP para que soporte y cliente tengan una ruta consistente de bootstrap hasta que el usuario cambie la contraseña desde el propio WordPress.
- El helper de auto-install quedó separado del create/start de Coolify para mantener `provision_hosting()` por debajo del límite de líneas del proyecto y evitar que el endurecimiento vuelva a caer en un bloque monolítico difícil de validar.

## Verificación

- `npm run self-check -- -TareaId 165A-5+165A-6+165A-7+165A-8+165A-9+165A-10+165A-11`
- `cargo test` con nuevas pruebas sobre labels Traefik y dominio custom en `src/services/coolify.rs`
- `cargo test services::coolify::tests --lib`
- `npm --prefix frontend run type-check`
- `cargo check`
- Probes reales a `site-hosting-2dad31af.173.249.50.44.sslip.io` y `173.249.50.44:12132` tras el rescate operativo