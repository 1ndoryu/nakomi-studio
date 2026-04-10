# Tarjetas guardadas con Stripe SetupIntent - 2026-04-10

## Objetivo

Reemplazar el placeholder roto de metodos de pago por un flujo real para agregar, listar y borrar tarjetas guardadas desde el panel del cliente.

## Cambios aplicados

- Se agrego persistencia local de tarjetas en `user_payment_methods` y `users.stripe_customer_id`.
- El backend crea SetupIntents, reutiliza o crea el customer de Stripe, guarda metadata local de la tarjeta y la desacopla de Stripe al eliminarla.
- `SeccionMetodosPago` ahora muestra estados reales: carga, error, vacio, lista de tarjetas y borrado.
- `AddPaymentMethodModal` usa Stripe.js + `confirmCardSetup` y luego confirma el guardado contra `/api/payment-methods`.
- La direccion de facturacion sigue fuera de alcance y la UI lo declara de forma honesta en vez de fingir un guardado inexistente.

## Flujo resultante

1. El cliente abre "Agregar tarjeta".
2. El frontend pide `/api/payment-methods/setup-intent`.
3. Stripe.js confirma la tarjeta con el `client_secret`.
4. El frontend envia el `setup_intent_id` confirmado al backend.
5. El backend valida que el SetupIntent quedo exitoso y pertenece al customer del usuario.
6. La tarjeta queda listada como metodo reusable en el panel.

## Validacion

- `cargo check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm --prefix frontend run type-check`
- `npm --prefix frontend run build`
- Backend levantado localmente con `target/debug/glory-backend.exe`
- Login local con `cliente@test.com`
- `POST /api/payment-methods/setup-intent` responde `200`
- `GET /api/payment-methods` responde `200`

## Limitacion local

La automatizacion completa del alta de tarjeta por API no es viable en este entorno porque Stripe esta configurado con llaves live y bloquea el envio directo de numeros de tarjeta por REST. El modal nuevo usa Stripe.js, que si es el flujo soportado para confirmar la tarjeta en navegador.