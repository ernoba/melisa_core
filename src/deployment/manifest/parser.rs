// ============================================================================
// src/deployment/manifest/parser.rs
//
// MELISA `.mel` manifest file loader and validator.
//
// Reads a TOML file from disk, deserializes it into `MelManifest`, then
// validates required fields and structural constraints before returning.
// ============================================================================

use tokio::fs;

use crate::deployment::manifest::types::{MelManifest, PortSection, VolumeSection};

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors produced during `.mel` file loading and validation.
#[derive(Debug, thiserror::Error)]
pub enum MelParseError {
    /// The specified manifest file does not exist on the filesystem.
    #[error("Manifest file not found: '{0}'")]
    NotFound(String),
    /// The file content could not be parsed as valid TOML.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// A filesystem I/O error occurred while reading the file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// The manifest is syntactically valid TOML but fails semantic validation.
    #[error("Manifest validation error: {0}")]
    Invalid(String),
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Reads, deserializes, and validates a `.mel` manifest file.
///
/// # Arguments
/// * `path` - Filesystem path to the `.mel` file.
///
/// # Errors
/// * [`MelParseError::NotFound`]  — file does not exist.
/// * [`MelParseError::TomlParse`] — file is not valid TOML.
/// * [`MelParseError::Io`]        — filesystem read error.
/// * [`MelParseError::Invalid`]   — semantic validation failure.
pub async fn load_mel_file(path: &str) -> Result<MelManifest, MelParseError> {
    if !std::path::Path::new(path).exists() {
        return Err(MelParseError::NotFound(path.to_string()));
    }

    let raw_content = fs::read_to_string(path).await?;
    let manifest: MelManifest = toml::from_str(&raw_content)?;
    validate_manifest(&manifest)?;

    Ok(manifest)
}

/// Validates the semantic constraints of a parsed `MelManifest`.
///
/// This function is re-exported as `pub` for use in unit tests that
/// construct manifests directly without going through file I/O.
///
/// # Errors
/// Returns [`MelParseError::Invalid`] describing the first violated constraint.
pub fn validate_manifest(manifest: &MelManifest) -> Result<(), MelParseError> {
    validate_project_name(&manifest.project.name)?;
    validate_distro_string(&manifest.container.distro)?;
    validate_port_format(&manifest.ports)?;
    validate_volume_format(&manifest.volumes)?;
    Ok(())
}

// ── Validation helpers ────────────────────────────────────────────────────────

/// Ensures `project.name` is not blank.
fn validate_project_name(name: &str) -> Result<(), MelParseError> {
    if name.trim().is_empty() {
        return Err(MelParseError::Invalid(
            "[project].name is required and must not be empty".into(),
        ));
    }
    Ok(())
}

/// Ensures `container.distro` is not blank.
fn validate_distro_string(distro: &str) -> Result<(), MelParseError> {
    if distro.trim().is_empty() {
        return Err(MelParseError::Invalid(
            "[container].distro is required (run 'melisa --search' to list valid codes)".into(),
        ));
    }
    Ok(())
}

/// Ensures every entry in `ports.expose` follows the `"host:container"` format.
fn validate_port_format(ports: &PortSection) -> Result<(), MelParseError> {
    for port_entry in &ports.expose {
        if port_entry.split(':').count() != 2 {
            return Err(MelParseError::Invalid(format!(
                "Invalid port format '{}': expected 'host_port:container_port'",
                port_entry
            )));
        }
    }
    Ok(())
}

/// Ensures every entry in `volumes.mounts` follows the `"host:container"` format.
fn validate_volume_format(volumes: &VolumeSection) -> Result<(), MelParseError> {
    for mount_entry in &volumes.mounts {
        if mount_entry.split(':').count() != 2 {
            return Err(MelParseError::Invalid(format!(
                "Invalid volume format '{}': expected 'host_path:container_path'",
                mount_entry
            )));
        }
    }
    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::deployment::manifest::types::{
        ContainerSection, DependencySection, LifecycleSection,
        MelManifest, PortSection, ProjectSection, VolumeSection,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Writes content to a temporary file and returns the handle (keeps file alive).
    fn write_temp_manifest(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temporary file");
        file.write_all(content.as_bytes())
            .expect("Failed to write temporary manifest");
        file
    }

    /// Constructs a minimal valid `MelManifest` for validation tests.
    fn make_valid_manifest() -> MelManifest {
        MelManifest {
            project: ProjectSection {
                name: "test-app".into(),
                version: None,
                description: None,
                author: None,
            },
            container: ContainerSection {
                distro: "ubuntu/jammy/amd64".into(),
                name: None,
                auto_start: true,
            },
            env: HashMap::new(),
            dependencies: DependencySection::default(),
            ports: PortSection::default(),
            volumes: VolumeSection::default(),
            lifecycle: LifecycleSection::default(),
            services: HashMap::new(),
            health: None,
        }
    }

    fn make_manifest_with_name(name: &str) -> MelManifest {
        let mut m = make_valid_manifest();
        m.project.name = name.into();
        m
    }

    fn make_manifest_with_distro(distro: &str) -> MelManifest {
        let mut m = make_valid_manifest();
        m.container.distro = distro.into();
        m
    }

    // ── load_mel_file ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_load_mel_file_parses_minimal_manifest_successfully() {
        let content = r#"
[project]
name = "hello-app"
[container]
distro = "ubuntu/jammy/amd64"
"#;
        let file = write_temp_manifest(content);
        let result = load_mel_file(file.path().to_str().unwrap()).await;

        assert!(result.is_ok(), "Minimal valid manifest must parse without error");
        let manifest = result.unwrap();
        assert_eq!(manifest.project.name, "hello-app");
        assert_eq!(manifest.container.distro, "ubuntu/jammy/amd64");
        assert!(
            manifest.container.auto_start,
            "auto_start must default to true when not specified"
        );
    }

    #[tokio::test]
    async fn test_load_mel_file_parses_full_manifest_successfully() {
        let content = r#"
[project]
name        = "full-app"
version     = "2.0.0"
description = "Complete manifest test"
author      = "dev@example.com"
[container]
distro     = "debian/bookworm/amd64"
name       = "my-container"
auto_start = false
[env]
APP_PORT = "8080"
DEBUG    = "false"
[dependencies]
apt = ["curl", "git", "build-essential"]
pip = ["flask", "gunicorn"]
npm = ["typescript"]
[ports]
expose = ["8080:8080", "443:443"]
[volumes]
mounts = ["./src:/app/src", "./data:/var/data"]
[lifecycle]
on_create = ["mkdir -p /app/logs", "chmod 755 /app"]
on_start  = ["echo starting"]
on_stop   = ["echo stopping"]
[health]
command  = "curl -sf http://localhost:8080/health"
interval = 30
retries  = 3
timeout  = 10
"#;
        let file = write_temp_manifest(content);
        let manifest = load_mel_file(file.path().to_str().unwrap())
            .await
            .expect("Full manifest must parse without error");

        assert_eq!(manifest.project.version.as_deref(), Some("2.0.0"));
        assert_eq!(manifest.container.name.as_deref(), Some("my-container"));
        assert!(!manifest.container.auto_start);
        assert_eq!(manifest.env.get("APP_PORT").map(|s| s.as_str()), Some("8080"));
        assert_eq!(manifest.dependencies.apt.len(), 3);
        assert_eq!(manifest.dependencies.pip.len(), 2);
        assert_eq!(manifest.ports.expose.len(), 2);
        assert_eq!(manifest.volumes.mounts.len(), 2);
        assert_eq!(manifest.lifecycle.on_create.len(), 2);
        assert!(manifest.health.is_some(), "health section must be parsed");
        assert_eq!(manifest.health.unwrap().retries, Some(3));
    }

    #[tokio::test]
    async fn test_load_mel_file_returns_not_found_for_missing_path() {
        let result = load_mel_file("/tmp/MELISA_TEST_NONEXISTENT_12345.mel").await;
        assert!(result.is_err(), "Missing file must produce an error");
        assert!(
            matches!(result.unwrap_err(), MelParseError::NotFound(_)),
            "Error kind must be NotFound"
        );
    }

    #[tokio::test]
    async fn test_load_mel_file_returns_toml_parse_error_for_invalid_syntax() {
        let content = r#"
[project
name = "broken"
"#;
        let file = write_temp_manifest(content);
        let result = load_mel_file(file.path().to_str().unwrap()).await;
        assert!(result.is_err(), "Invalid TOML must produce an error");
        assert!(
            matches!(result.unwrap_err(), MelParseError::TomlParse(_)),
            "Error kind must be TomlParse"
        );
    }

    #[tokio::test]
    async fn test_load_mel_file_empty_dependencies_is_valid() {
        let content = r#"
[project]
name = "no-deps"
[container]
distro = "alpine/3.18/amd64"
"#;
        let file = write_temp_manifest(content);
        let manifest = load_mel_file(file.path().to_str().unwrap())
            .await
            .expect("Manifest with no dependencies must be valid");

        assert!(manifest.dependencies.apt.is_empty());
        assert!(manifest.dependencies.pip.is_empty());
        assert!(manifest.dependencies.npm.is_empty());
    }

    #[tokio::test]
    async fn test_load_mel_file_health_section_is_optional() {
        let content = r#"
[project]
name = "no-health"
[container]
distro = "ubuntu/jammy/amd64"
"#;
        let file = write_temp_manifest(content);
        let manifest = load_mel_file(file.path().to_str().unwrap())
            .await
            .expect("Manifest without health section must be valid");
        assert!(
            manifest.health.is_none(),
            "health section must be None when not specified"
        );
    }

    #[tokio::test]
    async fn test_load_mel_file_lifecycle_hook_order_is_preserved() {
        let content = r#"
[project]
name = "hook-order-test"
[container]
distro = "alpine/3.18/amd64"
[lifecycle]
on_create = ["step-1", "step-2", "step-3"]
on_stop   = ["cleanup"]
"#;
        let file = write_temp_manifest(content);
        let manifest = load_mel_file(file.path().to_str().unwrap())
            .await
            .expect("Lifecycle hooks must parse without error");

        assert_eq!(manifest.lifecycle.on_create[0], "step-1");
        assert_eq!(manifest.lifecycle.on_create[1], "step-2");
        assert_eq!(manifest.lifecycle.on_create[2], "step-3");
        assert_eq!(manifest.lifecycle.on_stop[0], "cleanup");
        assert!(
            manifest.lifecycle.on_start.is_empty(),
            "on_start must be empty when not specified"
        );
    }

    #[tokio::test]
    async fn test_load_mel_file_services_parsed_correctly() {
        let content = r#"
[project]
name = "svc-test"
[container]
distro = "ubuntu/jammy/amd64"
[services]
web    = { command = "node server.js", working_dir = "/app", enabled = true  }
worker = { command = "node worker.js", working_dir = "/app", enabled = false }
"#;
        let file = write_temp_manifest(content);
        let manifest = load_mel_file(file.path().to_str().unwrap())
            .await
            .expect("Services section must parse without error");

        assert_eq!(manifest.services.len(), 2);
        let web_svc = manifest.services.get("web").expect("'web' service must be present");
        assert!(web_svc.enabled, "web service must be enabled");
        assert_eq!(web_svc.command, "node server.js");
        let worker_svc = manifest.services.get("worker").expect("'worker' service must be present");
        assert!(!worker_svc.enabled, "worker service must be disabled");
    }

    // ── validate_manifest ─────────────────────────────────────────────────────

    #[test]
    fn test_validate_manifest_empty_project_name_returns_invalid_error() {
        let manifest = make_manifest_with_name("");
        let result = validate_manifest(&manifest);
        assert!(result.is_err(), "Empty project name must fail validation");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("name"),
            "Validation error must mention 'name' — got: '{}'", error_msg
        );
    }

    #[test]
    fn test_validate_manifest_blank_project_name_returns_invalid_error() {
        let manifest = make_manifest_with_name("   ");
        let result = validate_manifest(&manifest);
        assert!(
            result.is_err(),
            "Whitespace-only project name must fail validation"
        );
    }

    #[test]
    fn test_validate_manifest_empty_distro_returns_invalid_error() {
        let manifest = make_manifest_with_distro("");
        let result = validate_manifest(&manifest);
        assert!(result.is_err(), "Empty distro must fail validation");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("distro"),
            "Validation error must mention 'distro' — got: '{}'", error_msg
        );
    }

    #[test]
    fn test_validate_manifest_bad_port_format_returns_invalid_error() {
        let mut manifest = make_valid_manifest();
        manifest.ports.expose = vec!["8080".into()]; // missing colon
        let result = validate_manifest(&manifest);
        assert!(result.is_err(), "Port without colon separator must fail validation");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.to_lowercase().contains("port"),
            "Validation error must mention 'port'"
        );
    }

    #[test]
    fn test_validate_manifest_bad_volume_format_returns_invalid_error() {
        let mut manifest = make_valid_manifest();
        manifest.volumes.mounts = vec!["/only/one/path".into()]; // missing colon
        let result = validate_manifest(&manifest);
        assert!(result.is_err(), "Volume without colon separator must fail validation");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.to_lowercase().contains("volume"),
            "Validation error must mention 'volume'"
        );
    }

    #[test]
    fn test_validate_manifest_valid_manifest_passes() {
        let manifest = make_valid_manifest();
        assert!(
            validate_manifest(&manifest).is_ok(),
            "A fully valid manifest must pass all validation checks"
        );
    }

    #[test]
    fn test_validate_manifest_valid_ports_pass() {
        let mut manifest = make_valid_manifest();
        manifest.ports.expose = vec!["8080:8080".into(), "443:443".into()];
        assert!(
            validate_manifest(&manifest).is_ok(),
            "Valid 'host:container' port entries must pass validation"
        );
    }

    #[test]
    fn test_validate_manifest_valid_volumes_pass() {
        let mut manifest = make_valid_manifest();
        manifest.volumes.mounts = vec!["./src:/app/src".into()];
        assert!(
            validate_manifest(&manifest).is_ok(),
            "Valid 'host_path:container_path' volume entries must pass validation"
        );
    }
}