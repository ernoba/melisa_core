// ============================================================================
// src/distros/registry.rs
//
// Distro registry system that manages host and container distro mappings.
// This is the single source of truth for all distro configurations.
// ============================================================================

use std::collections::HashMap;
use crate::distros::abstraction::{ContainerDistro, DistroConfig, HostDistroFamily};
use crate::distros::builtin_distros::*;

/// Central registry for all supported host and container distros.
pub struct DistroRegistry {
    /// Maps distro ID (from /etc/os-release) to HostDistroFamily implementations
    host_distros: HashMap<String, Box<dyn HostDistroFamily>>,
    /// Maps container distro names to ContainerDistro implementations
    container_distros: HashMap<String, Box<dyn ContainerDistro>>,
}

impl DistroRegistry {
    /// Create a new registry with built-in distro definitions
    pub fn new() -> Self {
        let mut registry = Self {
            host_distros: HashMap::new(),
            container_distros: HashMap::new(),
        };
        registry.register_builtin_host_distros();
        registry.register_builtin_container_distros();
        registry
    }

    /// Register a custom host distro family
    pub fn register_host_distro(&mut self, id: &str, distro: Box<dyn HostDistroFamily>) {
        self.host_distros.insert(id.to_lowercase(), distro);
    }

    /// Register a custom container distro
    pub fn register_container_distro(&mut self, name: &str, distro: Box<dyn ContainerDistro>) {
        self.container_distros.insert(name.to_lowercase(), distro);
    }

    /// Get host distro configuration by ID (matches ID= or ID_LIKE= from /etc/os-release)
    pub fn get_host_distro(&self, id: &str) -> Option<DistroConfig> {
        let id_lower = id.to_lowercase();
        
        // Try exact match first
        if let Some(distro) = self.host_distros.get(&id_lower) {
            return Some(DistroConfig::from_trait(distro.as_ref()));
        }
        
        // Try to find a matching family by codename or contained string
        for (_, distro) in &self.host_distros {
            let codename = distro.codename();
            if id_lower.contains(codename) || codename.contains(&id_lower) {
                return Some(DistroConfig::from_trait(distro.as_ref()));
            }
        }
        
        None
    }

    /// Get container distro by name (e.g., "ubuntu", "alpine")
    pub fn get_container_distro(&self, name: &str) -> Option<&dyn ContainerDistro> {
        self.container_distros
            .get(&name.to_lowercase())
            .map(|b| b.as_ref())
    }

    /// Get or create a fallback container distro configuration
    pub fn get_container_distro_or_fallback(&self, name: &str) -> (String, &dyn ContainerDistro) {
        if let Some(distro) = self.get_container_distro(name) {
            (name.to_lowercase(), distro)
        } else {
            // Fallback: guess based on string matching
            let name_lower = name.to_lowercase();
            for (key, distro) in &self.container_distros {
                if name_lower.contains(key) || key.contains(&name_lower) {
                    return (name.to_lowercase(), distro.as_ref());
                }
            }
            // Ultimate fallback
            (name.to_lowercase(), &UnknownContainer)
        }
    }

    /// List all registered host distro codenames for debugging
    pub fn list_host_distro_codenames(&self) -> Vec<&str> {
        self.host_distros
            .values()
            .map(|d| d.codename())
            .collect()
    }

    /// List all registered container distro names
    pub fn list_container_distro_names(&self) -> Vec<&str> {
        self.container_distros
            .values()
            .map(|d| d.name())
            .collect()
    }

    // ── Built-in Registration Helpers ─────────────────────────────────────────

    fn register_builtin_host_distros(&mut self) {
        // Debian family - register all known Debian-like distros
        self.register_host_distro("ubuntu", Box::new(DebianFamily));
        self.register_host_distro("debian", Box::new(DebianFamily));
        self.register_host_distro("mint", Box::new(DebianFamily));
        self.register_host_distro("linuxmint", Box::new(DebianFamily));
        self.register_host_distro("raspbian", Box::new(DebianFamily));
        self.register_host_distro("kali", Box::new(DebianFamily));
        
        // Fedora family
        self.register_host_distro("fedora", Box::new(FedoraFamily));
        self.register_host_distro("rhel", Box::new(FedoraFamily));
        self.register_host_distro("centos", Box::new(FedoraFamily));
        self.register_host_distro("rocky", Box::new(FedoraFamily));
        self.register_host_distro("almalinux", Box::new(FedoraFamily));
        
        // Arch family
        self.register_host_distro("arch", Box::new(ArchFamily));
        self.register_host_distro("manjaro", Box::new(ArchFamily));
        self.register_host_distro("endeavouros", Box::new(ArchFamily));
        
        // Alpine
        self.register_host_distro("alpine", Box::new(AlpineFamily));
        
        // SUSE family
        self.register_host_distro("suse", Box::new(SuseFamily));
        self.register_host_distro("opensuse", Box::new(SuseFamily));
        self.register_host_distro("opensuse-leap", Box::new(SuseFamily));
        self.register_host_distro("opensuse-tumbleweed", Box::new(SuseFamily));
        
        // Special: OrbStack
        self.register_host_distro("orbstack", Box::new(OrbStackFamily));
    }

    fn register_builtin_container_distros(&mut self) {
        // Debian family
        self.register_container_distro("debian", Box::new(DebianContainer));
        self.register_container_distro("ubuntu", Box::new(UbuntuContainer));
        
        // Fedora family
        self.register_container_distro("fedora", Box::new(FedoraContainer));
        self.register_container_distro("centos", Box::new(CentOSContainer));
        self.register_container_distro("rocky", Box::new(RockyLinuxContainer));
        self.register_container_distro("almalinux", Box::new(AlmaLinuxContainer));
        
        // Alpine
        self.register_container_distro("alpine", Box::new(AlpineContainer));
        
        // Arch family
        self.register_container_distro("arch", Box::new(ArchContainer));
        self.register_container_distro("archlinux", Box::new(ArchContainer));
        self.register_container_distro("manjaro", Box::new(ManjaroContainer));
        
        // SUSE
        self.register_container_distro("opensuse", Box::new(SuseContainer));
        self.register_container_distro("suse", Box::new(SuseContainer));
    }
}

impl Default for DistroRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_finds_debian_family() {
        let registry = DistroRegistry::new();
        assert!(registry.get_host_distro("ubuntu").is_some());
        assert!(registry.get_host_distro("debian").is_some());
        assert!(registry.get_host_distro("mint").is_some());
    }

    #[test]
    fn test_registry_finds_fedora_family() {
        let registry = DistroRegistry::new();
        assert!(registry.get_host_distro("fedora").is_some());
        assert!(registry.get_host_distro("centos").is_some());
        assert!(registry.get_host_distro("rocky").is_some());
    }

    #[test]
    fn test_registry_finds_container_distros() {
        let registry = DistroRegistry::new();
        assert!(registry.get_container_distro("ubuntu").is_some());
        assert!(registry.get_container_distro("alpine").is_some());
        assert!(registry.get_container_distro("fedora").is_some());
    }

    #[test]
    fn test_registry_fallback_for_unknown() {
        let registry = DistroRegistry::new();
        let (_, distro) = registry.get_container_distro_or_fallback("unknown-distro");
        assert!(!distro.is_known());
    }
}
