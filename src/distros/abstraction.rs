// ============================================================================
// src/distros/abstraction.rs
//
// Abstract trait definitions for host and container distributions.
// This enables pluggable, extensible distro support without hardcoding.
// ============================================================================

/// Represents a host Linux distribution family with its associated tooling.
pub trait HostDistroFamily: Send + Sync {
    /// Human-readable name (e.g., "Debian / Ubuntu")
    fn name(&self) -> &str;

    /// System package manager command (e.g., "apt-get", "dnf", "pacman")
    fn pkg_manager(&self) -> &str;

    /// Which firewall tool to use
    fn firewall_tool(&self) -> FirewallKind;

    /// LXC-related packages required on this distro
    fn lxc_packages(&self) -> Vec<&str>;

    /// Distro family codename for matching (e.g., "debian", "fedora")
    fn codename(&self) -> &str;
}

/// Represents a container distribution (guest OS)
pub trait ContainerDistro: Send + Sync {
    /// Name of the container distro (e.g., "ubuntu", "alpine", "fedora")
    fn name(&self) -> &str;

    /// Primary package manager used in this distro (e.g., "apt", "apk", "dnf")
    fn pkg_manager(&self) -> &str;

    /// Command to update package repository (e.g., "apt-get update -y")
    fn pkg_update_cmd(&self) -> &str;

    /// Whether this distro is known/officially supported
    fn is_known(&self) -> bool {
        true
    }
}

/// Firewall management tool available on the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallKind {
    /// `firewalld` — used on Fedora/RHEL systems.
    Firewalld,
    /// `ufw` — used on Debian/Ubuntu systems.
    Ufw,
    /// Raw `iptables` — fallback for minimal systems.
    Iptables,
}

/// Per-distribution configuration used by the setup routine.
/// This is the concrete struct returned by trait implementations.
#[derive(Debug, Clone)]
pub struct DistroConfig {
    /// Human-readable name for display.
    pub name: String,
    /// System package manager executable name.
    pub pkg_manager: String,
    /// Package names required to install LXC on this distro.
    pub lxc_packages: Vec<String>,
    /// Firewall management tool available on this distro.
    pub firewall_tool: FirewallKind,
}

impl DistroConfig {
    /// Create a new distro config from a trait implementation
    pub fn from_trait(family: &dyn HostDistroFamily) -> Self {
        Self {
            name: family.name().to_string(),
            pkg_manager: family.pkg_manager().to_string(),
            lxc_packages: family.lxc_packages().into_iter().map(|s| s.to_string()).collect(),
            firewall_tool: family.firewall_tool(),
        }
    }
}
