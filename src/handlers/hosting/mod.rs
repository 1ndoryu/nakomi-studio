mod checkout;
mod control;
mod deployments;
mod domain;
mod infrastructure;
mod plans;
mod provisioning;
mod routes;
mod stats;
mod subscriptions;
mod vps;

/* [225A-1] El antiguo controlador monolítico de hosting se dividió por responsabilidad.
 * La raíz queda como fachada para que los límites de tamaño no vuelvan a ocultar
 * archivos enormes y para que cada área compile/testee aislada. */
pub use routes::hosting_routes;
