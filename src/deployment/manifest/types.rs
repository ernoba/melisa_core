// ============================================================================
// src/deployment/manifest/types.rs
//
// Typed data structures for the MELISA `.mel` manifest format.
//
// A `.mel` file is a TOML document.  Serde deserialises it into a
// `MelManifest` which is the single source of truth for the deployment engine.
// ============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Root manifest ─────────────────────────────────────────────────────────────

/// The root type representing a complete `.mel` deployment manifest.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MelManifest {
    /// Identifies the project being deployed.
    pub project: ProjectSection,
    /// Describes the target LXC container.
    pub container: ContainerSection,
    /// Environment variables injected into the container.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// System and language package dependencies.
    #[serde(default)]
    pub dependencies: DependencySection,
    /// Host-to-container port mappings.
    #[serde(default)]
    pub ports: PortSection,
    /// Host-to-container directory bind-mounts.
    #[serde(default)]
    pub volumes: VolumeSection,
    /// Commands executed at container lifecycle events.
    #[serde(default)]
    pub lifecycle: LifecycleSection,
    /// Long-running service definitions.
    #[serde(default)]
    pub services: HashMap<String, ServiceDefinition>,
    /// Optional health check configuration.
    pub health: Option<HealthSection>,
}

// ── Project section ───────────────────────────────────────────────────────────

/// Metadata that identifies the project.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProjectSection {
    /// Required: human-readable project name (must not be blank).
    pub name: String,
    /// Optional: semantic version string (e.g. `"1.2.3"`).
    pub version: Option<String>,
    /// Optional: short project description.
    pub description: Option<String>,
    /// Optional: author name or email.
    pub author: Option<String>,
}

// ── Container section ─────────────────────────────────────────────────────────

/// Describes the LXC container to create or reuse.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContainerSection {
    /// LXC distribution slug (e.g. `"ubuntu/jammy/amd64"`).
    pub distro: String,
    /// Optional override for the container name.
    /// Defaults to a slugified form of `project.name`.
    pub name: Option<String>,
    /// Whether the container should start automatically after creation.
    #[serde(default = "default_true")]
    pub auto_start: bool,
}

impl ContainerSection {
    /// Returns the effective container name.
    ///
    /// Uses the explicit `name` field when set; otherwise falls back to a
    /// lower-cased, hyphenated form of `project_name`.
    ///
    /// # Arguments
    /// * `project_name` - The `project.name` value from the manifest.
    pub fn effective_name(&self, project_name: &str) -> String {
        self.name.clone().unwrap_or_else(|| {
            project_name.replace(' ', "-").to_lowercase()
        })
    }
}

fn default_true() -> bool {
    true
}

// ── Dependency section ────────────────────────────────────────────────────────

/// System and language package dependencies.
///
/// Packages are grouped by their installer.  Only the groups relevant to
/// the container's detected package manager are installed.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DependencySection {
    /// Packages installed via `apt-get` (Debian/Ubuntu).
    #[serde(default)]
    pub apt: Vec<String>,
    /// Packages installed via `pacman` (Arch Linux).
    #[serde(default)]
    pub pacman: Vec<String>,
    /// Packages installed via `dnf`/`yum` (Fedora/RHEL).
    #[serde(default)]
    pub dnf: Vec<String>,
    /// Packages installed via `zypper` (openSUSE).
    #[serde(default)]
    pub zypper: Vec<String>,
    /// Packages installed via `apk` (Alpine Linux).
    #[serde(default)]
    pub apk: Vec<String>,
    /// Python packages installed via `pip3`.
    #[serde(default)]
    pub pip: Vec<String>,
    /// Node packages installed globally via `npm`.
    #[serde(default)]
    pub npm: Vec<String>,
    /// Rust crates installed via `cargo install`.
    #[serde(default)]
    pub cargo: Vec<String>,
    /// Ruby gems installed via `gem install`.
    #[serde(default)]
    pub gem: Vec<String>,
    /// PHP packages installed via `composer global require`.
    #[serde(default)]
    pub composer: Vec<String>,
}

// ── Port section ──────────────────────────────────────────────────────────────

/// Container port exposure configuration.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PortSection {
    /// List of `"host:container"` port pairs to expose.
    #[serde(default)]
    pub expose: Vec<String>,
}

// ── Volume section ────────────────────────────────────────────────────────────

/// Container bind-mount configuration.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct VolumeSection {
    /// List of `"host_path:container_path"` bind-mount pairs.
    #[serde(default)]
    pub mounts: Vec<String>,
}

// ── Lifecycle section ─────────────────────────────────────────────────────────

/// Shell commands executed at specific points in the container lifecycle.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LifecycleSection {
    /// Commands run once after the container is created for the first time.
    #[serde(default)]
    pub on_create: Vec<String>,
    /// Commands run each time the container starts.
    #[serde(default)]
    pub on_start: Vec<String>,
    /// Commands run before the container is stopped.
    #[serde(default)]
    pub on_stop: Vec<String>,
}

// ── Service definition ────────────────────────────────────────────────────────

/// A single long-running service managed inside the container.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceDefinition {
    /// Shell command used to start the service.
    pub command: String,
    /// Optional working directory for the service process.
    pub working_dir: Option<String>,
    /// Whether this service is currently active.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

// ── Health check section ──────────────────────────────────────────────────────

/// Health check configuration for the deployed application.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthSection {
    /// Shell command used to test application readiness.
    pub command: String,
    /// Seconds between health check attempts (default: 5).
    pub interval: Option<u32>,
    /// Maximum number of retry attempts before reporting failure (default: 3).
    pub retries: Option<u32>,
    /// Seconds before a single check attempt times out (default: 30).
    pub timeout: Option<u32>,
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── ContainerSection::effective_name ─────────────────────────────────────

    #[test]
    fn test_effective_name_uses_explicit_container_name_when_set() {
        let section = ContainerSection {
            distro: "ubuntu/jammy/amd64".into(),
            name: Some("explicit-name".into()),
            auto_start: true,
        };
        assert_eq!(
            section.effective_name("My Project"),
            "explicit-name",
            "effective_name must return the explicit container name when it is set"
        );
    }

    #[test]
    fn test_effective_name_slugifies_project_name_when_container_name_is_absent() {
        let section = ContainerSection {
            distro: "ubuntu/jammy/amd64".into(),
            name: None,
            auto_start: true,
        };
        assert_eq!(
            section.effective_name("My Cool App"),
            "my-cool-app",
            "effective_name must convert spaces to hyphens and lowercase the project name"
        );
    }

    #[test]
    fn test_effective_name_preserves_lowercase_project_name() {
        let section = ContainerSection {
            distro: "alpine/3.18/amd64".into(),
            name: None,
            auto_start: false,
        };
        assert_eq!(
            section.effective_name("myapp"),
            "myapp",
            "effective_name must preserve a project name that is already lowercase without spaces"
        );
    }

    #[test]
    fn test_effective_name_handles_multiple_spaces() {
        let section = ContainerSection {
            distro: "debian/bookworm/amd64".into(),
            name: None,
            auto_start: true,
        };
        let name = section.effective_name("My Big Web App");
        assert_eq!(
            name, "my-big-web-app",
            "effective_name must replace every space with a hyphen"
        );
    }

    // ── default_true ──────────────────────────────────────────────────────────

    #[test]
    fn test_default_auto_start_is_true() {
        assert!(
            default_true(),
            "default_true helper must return true — it provides the default for auto_start"
        );
    }

    // ── DependencySection defaults ────────────────────────────────────────────

    #[test]
    fn test_dependency_section_default_all_fields_are_empty() {
        let deps = DependencySection::default();
        assert!(deps.apt.is_empty(), "DependencySection default apt must be empty");
        assert!(deps.pip.is_empty(), "DependencySection default pip must be empty");
        assert!(deps.npm.is_empty(), "DependencySection default npm must be empty");
        assert!(deps.cargo.is_empty(), "DependencySection default cargo must be empty");
        assert!(deps.gem.is_empty(), "DependencySection default gem must be empty");
        assert!(deps.composer.is_empty(), "DependencySection default composer must be empty");
    }

    // ── PortSection & VolumeSection defaults ──────────────────────────────────

    #[test]
    fn test_port_section_default_expose_is_empty() {
        let ports = PortSection::default();
        assert!(
            ports.expose.is_empty(),
            "PortSection default expose list must be empty"
        );
    }

    #[test]
    fn test_volume_section_default_mounts_is_empty() {
        let vols = VolumeSection::default();
        assert!(
            vols.mounts.is_empty(),
            "VolumeSection default mounts list must be empty"
        );
    }

    // ── LifecycleSection defaults ─────────────────────────────────────────────

    #[test]
    fn test_lifecycle_section_default_all_hooks_are_empty() {
        let lc = LifecycleSection::default();
        assert!(lc.on_create.is_empty(), "on_create must default to an empty list");
        assert!(lc.on_start.is_empty(), "on_start must default to an empty list");
        assert!(lc.on_stop.is_empty(), "on_stop must default to an empty list");
    }
}