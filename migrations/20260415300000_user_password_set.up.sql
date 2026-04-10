/* [154A-5] Columna para rastrear si el usuario estableció contraseña propia.
 * Usuarios existentes = true (ya tienen contraseña real).
 * quick_register = false (contraseña aleatoria, el usuario no la conoce). */
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_set BOOLEAN NOT NULL DEFAULT true;
