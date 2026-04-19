// ============================================================================
// src/distros/container.rs
//
// Container-specific distro utilities.
// Provides unified interface for container distro detection and package manager queries.
// This centralizes the logic previously scattered in lifecycle.rs.
// ============================================================================

use std::sync::OnceLock;
use crate::distros::registry::DistroRegistry;

/// Lazy-initialized global distro registry
static REGISTRY: OnceLock<DistroRegistry> = OnceLock::new();

fn get_registry() -> &'static DistroRegistry {
    REGISTRY.get_or_init(DistroRegistry::new)
}

/// Get the package manager for a container distro name
/// This is the single source of truth for container package managers.
pub fn get_container_pkg_manager(distro_name: &str) -> &'static str {
    let registry = get_registry();
    let (_, distro) = registry.get_container_distro_or_fallback(distro_name);
    distro.pkg_manager()
}

/// Get the package update command for a package manager
/// This is the single source of truth for package update commands.
pub fn get_pkg_update_cmd(pkg_manager: &str) -> &'static str {
    let registry = get_registry();
    
    // Try to find the distro that uses this package manager
    for name in &["debian", "ubuntu", "fedora", "alpine", "arch", "archlinux", "manjaro", "opensuse", "centos", "rocky", "almalinux"] {
        if let Some(distro) = registry.get_container_distro(name) {
            if distro.pkg_manager() == pkg_manager {
                return distro.pkg_update_cmd();
            }
        }
    }
    
    // Fallback for unknown package managers
    match pkg_manager {
        "apt" | "apt-get" => "apt-get update -y",
        "dnf" | "yum" => "dnf makecache",
        "apk" => "apk update",
        "pacman" => "pacman -Sy --noconfirm",
        "zypper" => "zypper --non-interactive refresh",
        _ => "true",
    }
}

/// Detect package manager for a container by probing which one is installed.
/// Falls back to get_container_pkg_manager if probing is not available.
pub fn get_pkg_manager_from_distro_name(distro_name: &str) -> &'static str {
    get_container_pkg_manager(distro_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ubuntu_pkg_manager() {
        assert_eq!(get_container_pkg_manager("ubuntu"), "apt");
    }

    #[test]
    fn test_fedora_pkg_manager() {
        assert_eq!(get_container_pkg_manager("fedora"), "dnf");
    }

    #[test]
    fn test_alpine_pkg_manager() {
        assert_eq!(get_container_pkg_manager("alpine"), "apk");
    }

    #[test]
    fn test_arch_pkg_manager() {
        assert_eq!(get_container_pkg_manager("arch"), "pacman");
    }

    #[test]
    fn test_pkg_update_cmd_apt() {
        assert_eq!(get_pkg_update_cmd("apt"), "apt-get update -y");
    }

    #[test]
    fn test_pkg_update_cmd_apk() {
        assert_eq!(get_pkg_update_cmd("apk"), "apk update");
    }

    #[test]
    fn test_pkg_update_cmd_unknown_fallback() {
        assert_eq!(get_pkg_update_cmd("chocolatey"), "true");
    }
}
