# Plan: Glory-RS Content Fixture System

**Fecha:** 2026-04-07
**Problema raíz:** Datos de prueba frágiles, hardcodeados en seed.rs, difíciles de mantener y extender. El usuario ha pedido datos de hosting 4x y no aparecen correctamente.

## Visión

Un sistema declarativo en glory-rs que:
1. Lee fixtures desde archivos (TOML) en un directorio `content/`
2. Sincroniza con la base de datos al arrancar (insert/update/delete)
3. Rastrea qué registros son "gestionados" para no tocar datos de usuario
4. Es agnóstico del proyecto — funciona con cualquier tabla

## Arquitectura

### Componentes

1. **`glory_fixtures` module** (en `glory-rs/backend/src/fixtures/`)
   - `mod.rs` — ContentManager principal
   - `parser.rs` — Parser de archivos TOML
   - `sync.rs` — Lógica de diff y sincronización
   - `registry.rs` — Registro de definiciones de tablas

2. **Tracking table** (`_glory_fixtures`)
   ```sql
   CREATE TABLE _glory_fixtures (
       id UUID PRIMARY KEY,
       fixture_file TEXT NOT NULL,
       table_name TEXT NOT NULL,
       record_id TEXT NOT NULL,
       content_hash TEXT NOT NULL,
       created_at TIMESTAMPTZ DEFAULT NOW(),
       updated_at TIMESTAMPTZ DEFAULT NOW(),
       UNIQUE(table_name, record_id)
   );
   ```

3. **Formato de fixtures** (`content/`)
   ```toml
   # content/users.toml
   [meta]
   table = "users"
   id_field = "email"  # campo usado como identificador natural
   depends_on = []  # orden de FK

   [[records]]
   email = "cliente@test.com"
   password_hash = "$argon2..." # o "plain:cliente" para auto-hash
   role = "client"
   display_name = "Cliente de Prueba"
   ```

4. **ContentManager API**
   ```rust
   pub struct ContentManager {
       pool: PgPool,
       content_dir: PathBuf,
   }

   impl ContentManager {
       pub async fn sync_all(&self) -> Result<SyncReport>;
       pub async fn sync_file(&self, path: &Path) -> Result<SyncReport>;
       pub async fn clean_orphans(&self) -> Result<u64>; // borra registros cuyo fixture desapareció
   }
   ```

### Flujo de sincronización

1. Escanear `content/*.toml`
2. Para cada archivo, parsear registros
3. Comparar content_hash con `_glory_fixtures`
4. Si hash cambió → UPDATE
5. Si registro nuevo → INSERT + registrar en _glory_fixtures
6. Si registro en _glory_fixtures pero no en archivo → DELETE
7. Respetar orden de FK (`depends_on` en meta)

### Fases de implementación

- **Fase 1:** Parser TOML + modelo de fixtures + tracking table migration
- **Fase 2:** ContentManager con sync_all (insert/update)
- **Fase 3:** Detección de eliminaciones (orphan cleanup)
- **Fase 4:** Soporte para relaciones FK (depends_on + resolución de IDs)
- **Fase 5:** Migrar seed.rs actual de nakomi a fixtures TOML

## Decisiones

- **TOML sobre JSON/YAML:** Legible, bien soportado en Rust (toml crate), sin ambigüedades
- **content_hash:** SHA-256 del registro serializado. Evita UPDATE innecesarios
- **plain:password:** Sintaxis especial para auto-hashear passwords con Argon2
- **Tabla tracking genérica:** Un solo lugar para saber qué es fixture y qué es dato real

## Impacto en nakomi

Una vez implementado, el seed.rs de 700+ líneas se reemplaza por archivos TOML en `content/`:
- content/users.toml
- content/orders.toml
- content/hosting.toml
- content/projects.toml
- content/team.toml
- content/services.toml
- content/blog.toml

## Pendientes

- [x] Fase 1: Parser + modelo + migración
- [x] Fase 2: sync_all (insert/update) 
- [x] Fase 3: orphan cleanup
- [x] Fase 4: FK dependencies (@table:record_id → db_id)
- [ ] Fase 5: Migrar nakomi seed.rs (parcial: users, projects, team_members, hosting migrados a TOML. Falta: orders, chat, reviews, activity log, notifications)
