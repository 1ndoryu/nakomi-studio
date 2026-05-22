# Limpieza de temporales de compilacion Cargo (2026-05-21)

## Estado medido

- `CARGO_TARGET_DIR=C:\tmp\glory-target`.
- `C:\tmp\glory-target` ocupa ~6422.79 MB.
- `target\` del workspace no existe.
- `frontend\node_modules\.vite` existe pero esta practicamente vacio.
- Hay un `cargo run --bin glory-backend` activo usando el target global; por eso la limpieza segura se pospone.

## Por que no se limpiaba automaticamente

Cargo no borra `target` automaticamente: conserva artefactos, incremental, deps y fingerprints para acelerar recompilaciones. En este proyecto se saco el target de OneDrive hacia `C:\tmp\glory-target` para evitar locks y sincronizacion lenta.

Ya existia un limpiador compartido en `glory-rs/scripts/clean-cargo-target.ps1` y un watcher en `glory-rs/scripts/watch-cargo-target.ps1`, pero habia dos problemas operativos:

1. El workspace raiz no tenia `scripts/clean-cargo-target.ps1`, aunque algunos scripts lo esperaban.
2. Si hay un `cargo run` activo usando directamente el target global, la limpieza segura debe posponerse para no borrar artefactos en uso.

## Correccion aplicada

- Se agrego wrapper raiz `scripts/clean-cargo-target.ps1`.
- Se agrego `npm run clean:cargo`.
- El wrapper usa `powershell -NoProfile` para evitar perfiles locales rotos.
- La limpieza manual ya esta disponible desde el workspace raiz.

## Uso

```powershell
npm run clean:cargo
```

Si hay un `cargo run` usando directamente `C:\tmp\glory-target`, la limpieza se pospone por seguridad. Para limpiar realmente ese caso, primero hay que detener el backend Rust activo y volver a ejecutar el comando.

Para automatizar limpieza mientras `npm run dev` esta activo hace falta aplicar el mismo criterio de exclusiones en el repo compartido `glory-rs`, que vive como clone anidado e ignorado por este repo.
