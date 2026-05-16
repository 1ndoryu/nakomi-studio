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

## Verificación

- `npm run self-check -- -TareaId 165A-5+165A-6+165A-7+165A-8+165A-9+165A-10+165A-11`
- `cargo test` con nuevas pruebas sobre labels Traefik y dominio custom en `src/services/coolify.rs`