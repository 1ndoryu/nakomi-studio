/* [104A-39] Sistema de lectura de chat.
 * last_viewed_at: cuándo el staff vio los mensajes de esta sesión por última vez.
 * Permite que el badge de ChatBell muestre sesiones con mensajes no leídos,
 * en vez de todas las sesiones activas (comportamiento previo). */

ALTER TABLE chat_sessions ADD COLUMN IF NOT EXISTS last_viewed_at TIMESTAMPTZ;
