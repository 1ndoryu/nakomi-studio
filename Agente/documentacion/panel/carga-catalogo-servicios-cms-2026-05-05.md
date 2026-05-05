# Carga de catalogo de servicios por API admin

- **Fecha:** 2026-05-05
- **Objetivo:** publicar el catalogo de 5 servicios y 3 planes por servicio en el CMS real de Nakomi sin fixtures ni escrituras directas a BD.
- **Script:** `scripts/push-services-cms.ps1`
- **Fuente:** `Agente/documentacion/panel/propuesta-servicios-planes-2026-05-05.json`

## Reglas operativas

- El script firma un JWT admin con `JWT_SECRET` y consume `POST /api/admin/services`, `PUT /api/admin/services/:id` y `PUT /api/admin/services/:id/plans`.
- Si existe mas de un slug candidato para el mismo servicio, prioriza el registro `is_active = true` antes que aliases legacy publicados pero inactivos.
- Si la propuesta no trae `image_url` o `gallery`, preserva la media existente y omite campos vacios para no borrar activos que el usuario gestiona aparte.
- Los planes se suben con slugs canonicos `basico`, `medio`, `avanzado`; si el servicio ya tiene esos planes, se preservan sus `id` para no romper ordenes historicas.
- Si el guardado de 3 planes falla y el servicio tiene planes legacy extra, el script reintenta preservando esos planes existentes.

## Validacion ejecutada

- `-DryRun` contra `https://nakomi.studio` para validar token, aliases, payloads y preservacion de media.
- Carga real por API admin.
- Verificacion posterior por API publica y admin: 5 servicios visibles (`diseno-de-sitios-web`, `desarrollo-apps`, `agentes-ia`, `branding`, `ecommerce`) y 3 planes por servicio.

## Resultado

- Se actualizo el servicio web activo `diseno-de-sitios-web` sin tocar su imagen existente.
- Se mantuvieron y actualizaron `desarrollo-apps`, `agentes-ia` y `branding`.
- Se creo `ecommerce` con su contenido, SEO, skills y 3 planes.