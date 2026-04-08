/* [084A-7] Añadir username para URLs públicas de perfil tipo /usuario/:username.
 * Se genera automáticamente desde display_name o email para usuarios existentes. */

ALTER TABLE users ADD COLUMN IF NOT EXISTS username VARCHAR(50);

/* Generar username para usuarios existentes: usar display_name (slug) o parte antes del @ del email */
UPDATE users
SET username = COALESCE(
    LOWER(REGEXP_REPLACE(TRIM(display_name), '[^a-zA-Z0-9]+', '-', 'g')),
    SPLIT_PART(email, '@', 1)
)
WHERE username IS NULL;

/* Limpiar guiones al inicio/final */
UPDATE users SET username = TRIM(BOTH '-' FROM username) WHERE username LIKE '-%' OR username LIKE '%-';

/* Hacer UNIQUE después de popular */
ALTER TABLE users ALTER COLUMN username SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);
