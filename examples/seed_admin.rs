/* sentinel-disable-file sqlx-query-sin-macro: seed usa runtime query para INSERT de admin. */
/* [044A-34] Seed: crea usuario admin/admin para desarrollo local.
 * Ejecutar: cargo run --example seed_admin
 * Requiere DATABASE_URL en .env o como variable de entorno. */

use argon2::password_hash::rand_core::OsRng;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL debe estar definida en .env");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    let email = "admin@admin.com";
    let password = "admin";

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Error al hashear contraseña")
        .to_string();

    let id = Uuid::new_v4();

    /* [054A-19] Usa query runtime (no macro) para no depender de .sqlx/ offline cache.
     * Los seeds son scripts de desarrollo, no código de producción.
     * [064A-42] Incluye display_name para que aparezca en la UI. */
    let result = sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name) VALUES ($1, $2, $3, 'Admin') ON CONFLICT (email) DO UPDATE SET password_hash = $3, display_name = 'Admin'"
    )
    .bind(id)
    .bind(email)
    .bind(&password_hash)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => println!("Usuario admin creado: email={email}, password={password}"),
        Err(e) => eprintln!("Error al crear usuario admin: {e}"),
    }
}
