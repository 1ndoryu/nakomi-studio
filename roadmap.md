Objetivo: Nakomi Studio — sitio web de agencia creativa. Migrado de WordPress a Rust (Axum) + React SPA.
Rama: glory-rust-nakomi

## Stack

| Capa | Herramienta |
|------|-------------|
| Framework web | Axum 0.7 |
| OpenAPI | utoipa 4 + utoipa-swagger-ui 7 |
| Base de datos | SQLx 0.8 (PostgreSQL) |
| Validación | validator 0.18 |
| Auth | jsonwebtoken + argon2 |
| Frontend | React 18 + TypeScript + Vite |
| State | React Query + Zustand |
| Codegen | Orval 8 |
| Deploy | coolify-manager-rs |

# Nakomi Studio — Roadmap

## Notas de infraestructura

- **nakomi.studio**: VPS1 (66.94.100.241), Coolify service `do8k4w8swccwwogoc0os0ck0`
- **VPS2 Coolify**: Configurado en settings.json
- **Deploy**: Siempre via coolify-manager-rs, nunca desde Coolify UI (ver doc de persistencia volúmenes)
- **Volúmenes**: Documentado en `Agente/documentacion/hosting/coolify-volumenes-persistencia-2026-04-12.md`

## Contexto

Proyecto migrado de WordPress a Rust (Axum) + React SPA. El frontend React se integra en frontend/src/. El backend Rust sirve API + SPA.

---

## Tareas pendientes

- **065A-4 — Resolver bloqueo BDP `[300035]` fuera de horario (sin escrituras reales).**
	- Estado actual: `sync-dry-run` ya valida lecturas reales; `CreateOrder` se ejecuta en `OnlyCheck` (`OrderOperationType=1`) con `escritura_real=false` y sin `Payments`.
	- Bloqueo vigente: BDP devuelve `[300035]-NO SE HA DEFINIDO UNA SERIE DE FACTURACION VALIDA`.
	- Nota clave: no es un bug del backend Rust; es configuracion interna de BDP-Net (serie/destino para pedidos WebLink).
	- Pendiente operativo para el usuario (cuando el restaurante no este en horas de trabajo):
		1. Conectarse por RDP a `100.83.196.35`.
		2. Abrir BDP-Net y entrar a `Utilidades -> Configuracion Servicios Web`.
		3. Localizar el ajuste de serie de destino/serie para pedidos externos WebLink (o nombre equivalente).
		4. Guardar evidencia de valores actuales antes de cambiar (captura/foto).
		5. Asignar una serie valida de facturacion simplificada para pedidos WebLink (segun configuracion del local).
		6. Guardar cambios y volver a la app.
		7. Ejecutar `Probar sincronizacion segura` en produccion.
	- Criterio de cierre: `listo_para_sincronizar=true` manteniendo `escritura_real=false`.
	- Restriccion: no crear/modificar ventas, comandas, clientes, articulos ni pagos reales en el restaurante.

- **215A-2 — Reemplazar modales de compra de hosting por pagina de configuracion reutilizable.** Hecho: rutas `/soluciones/hosting(-wordpress)/configurar/:plan`, hook/componente reutilizable, plan primero, dominio, wp-admin, idioma, SFTP opcional, cuenta y checkout.
- **215A-3 — Investigar Gateway Timeout en WordPress hosting comprado por usuario test.** Revisar por que `http://wordpress-hosting-de17015b.173.249.50.44.sslip.io/` dejo de funcionar tras compra con bypass. Este error no debe ocurrir. Diagnostico/fix: Traefik elegia una red Docker interna al no existir `traefik.docker.network`; se rescato el servicio conectando `coolify-proxy` a las redes del stack y se corrigio el generador de compose para fijar la red del servicio Coolify.
- **215A-4 — Normalizar beneficios visibles de planes hosting/VPS.** Hecho: hosting/WordPress muestran trafico ilimitado, dominio temporal, SSL; WordPress agrega Free CDN; VPS usa trafico ilimitado.
- **215A-5 — Analisis profundo de capacidad y rentabilidad VPS2.** Hecho: evidencia real documentada en `Agente/documentacion/hosting/vps2-capacidad-rentabilidad-2026-05-21.md`, recalibrada con pruebas manuales del usuario y coste real de `$7.44/mes`. El target actual tiene 4 vCPU/7.9 GB RAM/290 GB, no el supuesto viejo de 6 vCPU/16 GB.
- **215A-6 — Verificar backups diarios/semanales y uso real de espacio.** Hecho: compose nuevo agrega sidecar de backup para todos los planes que lo anuncian; basico semanal, pro/avanzado diario + semanal. Nota: hostings existentes necesitan refresh para recibir `backup-data`.
- **215A-7 — Corregir calculo de espacio para hosting normal y WordPress.** Hecho: storage via SSH ahora resuelve contenedores Coolify reales (`wordpress/site/mariadb-{uuid}`) y fallback legacy; WordPress suma archivos+DB, hosting normal mide `site-data`.
- **215A-8 — Permitir pagar hosting por 1 mes, 6 meses o 1 año con descuentos.** Hecho: frontend/backend/Stripe aceptan 1/6/12 meses con descuentos de $10/$20.
- **215A-9 — Renombrar planes E-commerce/WooCommerce a nombres genericos.** Hecho: `ecommerce` visible como Avanzado y sin prometer WooCommerce.
- **215A-10 — Reducir fuente de `tarjetaPlanDescripcion`.** Hecho: descripcion usa `var(--text-sm)`.
- **215A-11 — Analizar soporte de buzones de correo para clientes.** Hecho: documentado en `Agente/documentacion/hosting/buzones-correo-clientes-2026-05-21.md`; hoy solo hay SMTP transaccional, se recomienda proveedor externo antes de vender mailboxes.
- **215A-12 — Tooltips explicativos en caracteristicas de planes hosting/VPS.** Hecho: componente `PlanFeatureTooltip` en planes de hosting/VPS; WordPress dice `WP-CLI + SSH`; features no obvias explican alcance en hover/foco.
- **215A-13 — Investigar temporales que llegan y no se limpian automaticamente.** Hecho: documentado en `Agente/documentacion/tooling/cargo-target-cleanup-2026-05-21.md`; wrapper `npm run clean:cargo` agregado. El target global no se limpia mientras un `cargo run` activo lo usa.
- **215A-14 — Infraestructura: detalles VPS + tabla minimalista de despliegues.** Hecho: backend enriquece `/api/hosting/deployments` con CPU, RAM, disco y nombre del cliente por despliegue (via cache Docker stats). Frontend reemplaza tarjetas por tabla minimalista con resumen VPS (CPU, RAM, disco, conteos WP/Normal), iconos de tipo, columna de usuario, menu contextual de 3 puntos y fila expandible para detalles.
