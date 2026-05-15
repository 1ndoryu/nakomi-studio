/* [155A-1] Vinculación de cuentas Google OAuth.
 * Tabla separada para no tocar users ni invalidar el cache SQLx offline.
 * Un usuario puede tener máximo una cuenta Google (PK = user_id).
 * google_id UNIQUE garantiza que el mismo Google no se vincule a dos usuarios. */
CREATE TABLE user_google_accounts (
    user_id   UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    google_id VARCHAR(255) NOT NULL UNIQUE,
    email     VARCHAR(255),
    name      VARCHAR(255),
    picture   TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id)
);
