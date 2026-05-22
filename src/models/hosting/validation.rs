use regex::Regex;
use std::sync::LazyLock;

/* [094A-9] Regex para validar nombres de dominio (RFC 1035/1123).
 * Acepta subdominios y TLDs estándar; rechaza IPs, rutas y esquemas. */
pub(crate) static DOMAIN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?i)(?:[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?\.)*[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?$",
    )
    .expect("regex de dominio válida")
});

pub(crate) static HOSTING_USERNAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9_-]{2,59}$").expect("regex de usuario hosting válida")
});
