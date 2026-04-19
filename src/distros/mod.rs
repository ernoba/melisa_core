// ============================================================================
// src/distros/mod.rs
// ============================================================================

pub mod abstraction;
pub mod builtin_distros;
pub mod registry;
pub mod container;
pub mod host_distro;
pub mod lxc_distro;

// Re-export commonly used types for convenience
pub use abstraction::{ContainerDistro, DistroConfig, FirewallKind, HostDistroFamily};
pub use registry::DistroRegistry;
pub use host_distro::{detect_host_distro, get_distro_config, HostDistro};
pub use container::{get_container_pkg_manager, get_pkg_update_cmd, get_pkg_manager_from_distro_name};