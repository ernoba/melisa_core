// ============================================================================
// src/distros/builtin_distros.rs
//
// Built-in concrete implementations of HostDistroFamily and ContainerDistro traits.
// This centralizes all distro-specific configuration in one place.
// ============================================================================

use crate::distros::abstraction::{ContainerDistro, FirewallKind, HostDistroFamily};

// ── Host Distros ──────────────────────────────────────────────────────────────

/// Debian family implementation (Ubuntu, Debian, Linux Mint, etc.)
pub struct DebianFamily;
impl HostDistroFamily for DebianFamily {
    fn name(&self) -> &str { "Debian / Ubuntu" }
    fn pkg_manager(&self) -> &str { "apt-get" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Ufw }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "lxc-templates", "uidmap", "bridge-utils", "dnsmasq"]
    }
    fn codename(&self) -> &str { "debian" }
}

/// Fedora family implementation (Fedora, RHEL, CentOS, Rocky, AlmaLinux)
pub struct FedoraFamily;
impl HostDistroFamily for FedoraFamily {
    fn name(&self) -> &str { "Fedora / RHEL" }
    fn pkg_manager(&self) -> &str { "dnf" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Firewalld }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "lxc-templates", "lxc-extra", "dnsmasq"]
    }
    fn codename(&self) -> &str { "fedora" }
}

/// Arch Linux family implementation
pub struct ArchFamily;
impl HostDistroFamily for ArchFamily {
    fn name(&self) -> &str { "Arch Linux" }
    fn pkg_manager(&self) -> &str { "pacman" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Iptables }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "bridge-utils"]
    }
    fn codename(&self) -> &str { "arch" }
}

/// Alpine Linux implementation
pub struct AlpineFamily;
impl HostDistroFamily for AlpineFamily {
    fn name(&self) -> &str { "Alpine Linux" }
    fn pkg_manager(&self) -> &str { "apk" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Iptables }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "lxc-templates"]
    }
    fn codename(&self) -> &str { "alpine" }
}

/// openSUSE family implementation
pub struct SuseFamily;
impl HostDistroFamily for SuseFamily {
    fn name(&self) -> &str { "openSUSE" }
    fn pkg_manager(&self) -> &str { "zypper" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Firewalld }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "lxc-templates", "bridge-utils"]
    }
    fn codename(&self) -> &str { "suse" }
}

/// OrbStack virtual machine implementation
pub struct OrbStackFamily;
impl HostDistroFamily for OrbStackFamily {
    fn name(&self) -> &str { "OrbStack Virtual Machine" }
    fn pkg_manager(&self) -> &str { "apt-get" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Iptables }
    fn lxc_packages(&self) -> Vec<&str> {
        vec![
            "lxc", "bridge-utils", "dnsmasq", "ufw", 
            "iptables", "uidmap", "openssh-server"
        ]
    }
    fn codename(&self) -> &str { "orbstack" }
}

/// Fallback/Unknown distribution
pub struct UnknownFamily;
impl HostDistroFamily for UnknownFamily {
    fn name(&self) -> &str { "Unknown / Generic" }
    fn pkg_manager(&self) -> &str { "apt-get" }
    fn firewall_tool(&self) -> FirewallKind { FirewallKind::Iptables }
    fn lxc_packages(&self) -> Vec<&str> {
        vec!["lxc", "lxc-templates"]
    }
    fn codename(&self) -> &str { "unknown" }
}

// ── Container Distros ──────────────────────────────────────────────────────────

/// Debian container distro
pub struct DebianContainer;
impl ContainerDistro for DebianContainer {
    fn name(&self) -> &str { "debian" }
    fn pkg_manager(&self) -> &str { "apt" }
    fn pkg_update_cmd(&self) -> &str { "apt-get update -y" }
}

/// Ubuntu container distro
pub struct UbuntuContainer;
impl ContainerDistro for UbuntuContainer {
    fn name(&self) -> &str { "ubuntu" }
    fn pkg_manager(&self) -> &str { "apt" }
    fn pkg_update_cmd(&self) -> &str { "apt-get update -y" }
}

/// Fedora container distro
pub struct FedoraContainer;
impl ContainerDistro for FedoraContainer {
    fn name(&self) -> &str { "fedora" }
    fn pkg_manager(&self) -> &str { "dnf" }
    fn pkg_update_cmd(&self) -> &str { "dnf makecache" }
}

/// Alpine Linux container distro
pub struct AlpineContainer;
impl ContainerDistro for AlpineContainer {
    fn name(&self) -> &str { "alpine" }
    fn pkg_manager(&self) -> &str { "apk" }
    fn pkg_update_cmd(&self) -> &str { "apk update" }
}

/// Arch Linux container distro
pub struct ArchContainer;
impl ContainerDistro for ArchContainer {
    fn name(&self) -> &str { "arch" }
    fn pkg_manager(&self) -> &str { "pacman" }
    fn pkg_update_cmd(&self) -> &str { "pacman -Sy --noconfirm" }
}

/// Manjaro container distro
pub struct ManjaroContainer;
impl ContainerDistro for ManjaroContainer {
    fn name(&self) -> &str { "manjaro" }
    fn pkg_manager(&self) -> &str { "pacman" }
    fn pkg_update_cmd(&self) -> &str { "pacman -Sy --noconfirm" }
}

/// openSUSE container distro
pub struct SuseContainer;
impl ContainerDistro for SuseContainer {
    fn name(&self) -> &str { "opensuse" }
    fn pkg_manager(&self) -> &str { "zypper" }
    fn pkg_update_cmd(&self) -> &str { "zypper --non-interactive refresh" }
}

/// CentOS container distro
pub struct CentOSContainer;
impl ContainerDistro for CentOSContainer {
    fn name(&self) -> &str { "centos" }
    fn pkg_manager(&self) -> &str { "dnf" }
    fn pkg_update_cmd(&self) -> &str { "dnf makecache" }
}

/// Rocky Linux container distro
pub struct RockyLinuxContainer;
impl ContainerDistro for RockyLinuxContainer {
    fn name(&self) -> &str { "rocky" }
    fn pkg_manager(&self) -> &str { "dnf" }
    fn pkg_update_cmd(&self) -> &str { "dnf makecache" }
}

/// AlmaLinux container distro
pub struct AlmaLinuxContainer;
impl ContainerDistro for AlmaLinuxContainer {
    fn name(&self) -> &str { "almalinux" }
    fn pkg_manager(&self) -> &str { "dnf" }
    fn pkg_update_cmd(&self) -> &str { "dnf makecache" }
}

/// Fallback for unknown container distros
pub struct UnknownContainer;
impl ContainerDistro for UnknownContainer {
    fn name(&self) -> &str { "unknown" }
    fn pkg_manager(&self) -> &str { "apt" }
    fn pkg_update_cmd(&self) -> &str { "true" }
    fn is_known(&self) -> bool { false }
}
