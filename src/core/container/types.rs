// ============================================================================
// src/core/container/types.rs
//
// Shared types and constants for LXC container management.
// ============================================================================

/// Absolute path to the LXC container storage directory on the host.
pub const LXC_BASE_PATH: &str = "/var/lib/lxc";

/// Alias kept for backward compatibility with modules that still reference `LXC_PATH`.
pub const LXC_PATH: &str = LXC_BASE_PATH;

// ── Data structures ──────────────────────────────────────────────────────────

/// Metadata describing a single LXC distribution available for download.
#[derive(Debug, Clone)]
pub struct DistroMetadata {
    /// Unique slug used as the `--create` code (e.g. `"ubuntu/jammy/amd64"`).
    pub slug: String,
    /// Human-readable distribution name (e.g. `"Ubuntu"`).
    pub name: String,
    /// CPU architecture (e.g. `"amd64"`).
    pub arch: String,
}

/// Operational state of an LXC container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Container process is alive and accepting commands.
    Running,
    /// Container process is not running.
    Stopped,
    /// Container status could not be determined.
    Unknown,
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerStatus::Running => write!(f, "RUNNING"),
            ContainerStatus::Stopped => write!(f, "STOPPED"),
            ContainerStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lxc_base_path_constant_is_absolute() {
        assert!(
            LXC_BASE_PATH.starts_with('/'),
            "LXC_BASE_PATH must be an absolute filesystem path"
        );
    }

    #[test]
    fn test_lxc_path_alias_matches_base_path() {
        assert_eq!(
            LXC_PATH, LXC_BASE_PATH,
            "LXC_PATH alias must equal LXC_BASE_PATH to maintain backward compatibility"
        );
    }

    #[test]
    fn test_container_status_display_running() {
        assert_eq!(
            ContainerStatus::Running.to_string(),
            "RUNNING",
            "Running status must display as 'RUNNING'"
        );
    }

    #[test]
    fn test_container_status_display_stopped() {
        assert_eq!(
            ContainerStatus::Stopped.to_string(),
            "STOPPED",
            "Stopped status must display as 'STOPPED'"
        );
    }

    #[test]
    fn test_container_status_display_unknown() {
        assert_eq!(
            ContainerStatus::Unknown.to_string(),
            "UNKNOWN",
            "Unknown status must display as 'UNKNOWN'"
        );
    }

    #[test]
    fn test_distro_metadata_fields_are_accessible() {
        let meta = DistroMetadata {
            slug: "ubuntu/jammy/amd64".to_string(),
            name: "Ubuntu".to_string(),
            arch: "amd64".to_string(),
        };
        assert_eq!(meta.slug, "ubuntu/jammy/amd64");
        assert_eq!(meta.name, "Ubuntu");
        assert_eq!(meta.arch, "amd64");
    }

    #[test]
    fn test_container_status_equality() {
        assert_eq!(ContainerStatus::Running, ContainerStatus::Running);
        assert_ne!(ContainerStatus::Running, ContainerStatus::Stopped);
    }
}