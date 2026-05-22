# Buzones de correo para clientes de hosting (2026-05-21)

## Estado actual

El proyecto solo tiene SMTP transaccional para enviar emails de la app:

- `src/services/email.rs` usa `lettre` con `SMTP_*` / `GLORY_SMTP_*`.
- Sirve para confirmaciones, escalaciones, notificaciones y emails propios de Nakomi.
- No existe modelo de buzones, aliases, dominios de correo, IMAP/POP3, webmail, cuotas ni reset de contrasena de mailbox.
- La gestion de dominios ya contempla DNS/MX como idea futura, pero no esta implementada como producto.

Conclusion: hoy no se pueden vender buzones tipo hosting tradicional desde la plataforma sin integrar un proveedor o montar infraestructura de mail.

## No recomendado: self-host mail en VPS2

No conviene montar Postfix/Dovecot/Rspamd en el mismo VPS2 de hosting:

- Entregabilidad fragil: IP nueva o compartida con hosting web suele caer en spam.
- Requiere PTR/rDNS, SPF, DKIM, DMARC, reputacion IP, queue management y monitoreo 24/7.
- Aumenta superficie de abuso: spam, credenciales robadas, malware, backscatter.
- Consume tiempo operativo desproporcionado frente al margen de planes de hosting baratos.
- Mezclar correo y hosting web en el mismo nodo complica backups, firewall y reputacion.

## Camino viable recomendado

Vender correo como add-on gestionado usando proveedor externo de mailboxes con API o panel reseller:

1. **Proveedor principal**: Migadu, MXroute, Zoho Mail, Forward Email, Mailbox.org o similar.
2. **Nakomi gestiona DNS**: MX, SPF, DKIM y DMARC sobre el dominio verificado del cliente.
3. **Panel Nakomi muestra estado**: dominio verificado, registros DNS pendientes, buzones contratados, cuota y enlaces de acceso.
4. **Provisioning externo**: crear mailbox/alias via API si el proveedor lo permite; si no, flujo asistido con evento admin.
5. **Billing**: Stripe add-on mensual/anual por dominio o por mailbox.

## Producto sugerido

| Producto | Uso | Precio sugerido | Implementacion |
| --- | --- | ---: | --- |
| Alias de correo | `info@dominio` reenvia a Gmail/Outlook del cliente | bajo / incluido en pro | Cloudflare Email Routing o proveedor externo |
| 1 mailbox real | IMAP/SMTP/webmail para una cuenta | add-on mensual | proveedor mailbox externo |
| Pack 5 mailboxes | equipos pequenos | add-on mensual | proveedor externo |
| Migracion correo | mover buzones existentes | pago unico | operacion manual o herramienta IMAP sync |

## Cambios necesarios en Nakomi

### Datos

Tablas futuras:

- `mail_domains`: dominio, hosting_subscription_id, proveedor, estado DNS, verificado_at.
- `mailboxes`: mail_domain_id, local_part, display_name, status, provider_mailbox_id, quota_mb, billing_status.
- `mail_events`: provisioning, DNS pending, password reset, suspend, delete.

Nunca guardar password de mailbox en texto plano. Para alta inicial, generar password temporal, mostrarlo una sola vez o enviar flujo de reset del proveedor.

### Backend

- `GET /api/hosting/{id}/mail` para listar dominio/buzones.
- `POST /api/hosting/{id}/mailboxes` para solicitar/crear buzon.
- `POST /api/hosting/{id}/mailboxes/{mailbox_id}/reset-password`.
- `DELETE /api/hosting/{id}/mailboxes/{mailbox_id}` con grace period.
- Integracion DNS para MX/SPF/DKIM/DMARC cuando el dominio esta verificado.
- Webhook/billing para suspender add-ons impagados.

### Frontend

- Tab `Correo` dentro del detalle de hosting.
- Estado claro: `DNS pendiente`, `Activo`, `Suspendido`, `Requiere accion`.
- Registros DNS copiables si el dominio no esta bajo DNS gestionado por Nakomi.
- CTA para contratar alias/mailbox sin prometerlo como incluido hasta que exista provisioning.

## Decision recomendada

Para corto plazo: ofrecer solo **aliases/reenvios** o correo asistido con proveedor externo. Para buzones reales: elegir proveedor antes de tocar codigo de provisioning. No anunciar mailboxes incluidos en los planes actuales hasta tener API, billing, DNS y soporte listos.
