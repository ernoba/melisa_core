// ============================================================================
// src/core/metadata.rs
//
// MELISA container metadata management.
//
// Metadata is stored as a plain-text file at:
//   <LXC_BASE_PATH>/<container>/rootfs/etc/melisa-info
//
// Writes are performed atomically: the data is first written to a `.tmp` file
// which is then renamed into place.  This prevents partial reads if the
// process is interrupted mid-write.
// ============================================================================

use std::path::PathBuf;
use tokio::fs;

use crate::cli::color::{BOLD, RESET};
use crate::core::container::types::LXC_BASE_PATH;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors produced by metadata operations.
#[derive(Debug, thiserror::Error)]
pub enum MelisaError {
    /// The metadata file was not found in the container rootfs.
    #[error("No MELISA metadata found for container '{0}'")]
    MetadataNotFound(String),
    /// A filesystem I/O error occurred while reading or writing metadata.
    #[error("IO error while accessing metadata: {0}")]
    Io(#[from] std::io::Error),
}

// ── Paths ─────────────────────────────────────────────────────────────────────

/// Returns the path to the metadata file inside the container rootfs.
fn metadata_file_path(container_name: &str) -> PathBuf {
    PathBuf::from(LXC_BASE_PATH)
        .join(container_name)
        .join("rootfs")
        .join("etc")
        .join("melisa-info")
}

/// Returns the path to the temporary metadata file used during atomic writes.
fn metadata_temp_path(container_name: &str) -> PathBuf {
    PathBuf::from(LXC_BASE_PATH)
        .join(container_name)
        .join("rootfs")
        .join("etc")
        .join("melisa-info.tmp")
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Reads the MELISA metadata for a container.
///
/// # Arguments
/// * `container_name` - The LXC container name.
///
/// # Errors
/// Returns [`MelisaError::MetadataNotFound`] if the metadata file does not exist.
/// Returns [`MelisaError::Io`] for any other filesystem error.
pub async fn inspect_container_metadata(container_name: &str) -> Result<String, MelisaError> {
    let path = metadata_file_path(container_name);
    if !path.exists() {
        return Err(MelisaError::MetadataNotFound(container_name.to_string()));
    }
    let content = fs::read_to_string(&path).await?;
    Ok(content)
}

/// Atomically writes metadata for a container.
///
/// The content is written to a `.tmp` file first, then renamed to the final
/// path.  If the write fails mid-way, the original metadata file is preserved.
///
/// # Arguments
/// * `container_name` - The LXC container name.
/// * `content`        - Raw metadata content to persist.
///
/// # Errors
/// Returns [`MelisaError::Io`] if any filesystem operation fails.
pub async fn write_container_metadata(
    container_name: &str,
    content: &str,
) -> Result<(), MelisaError> {
    let final_path = metadata_file_path(container_name);
    let temp_path = metadata_temp_path(container_name);

    // Ensure the /etc directory exists inside the rootfs.
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write to the temporary file.
    fs::write(&temp_path, content).await?;

    // Atomically rename into the final position.
    fs::rename(&temp_path, &final_path).await?;

    Ok(())
}

/// Removes the MELISA metadata file (and its `.tmp` counterpart) for a container.
///
/// Called during container deletion to leave no orphaned files.
///
/// # Arguments
/// * `container_name` - The LXC container name.
pub async fn cleanup_container_metadata(container_name: &str) {
    let final_path = metadata_file_path(container_name);
    let temp_path = metadata_temp_path(container_name);

    if fs::try_exists(&final_path).await.unwrap_or(false) {
        let _ = fs::remove_file(&final_path).await;
    }
    if fs::try_exists(&temp_path).await.unwrap_or(false) {
        let _ = fs::remove_file(&temp_path).await;
    }
}

// ── Version display ───────────────────────────────────────────────────────────

/// Current MELISA server version string.
const MELISA_VERSION: &str = "0.1.3";

/// Prints the MELISA server version to stdout.
pub async fn print_version() {
    println!("{}MELISA SERVER VERSION{} {}", BOLD, RESET, MELISA_VERSION);
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_file_path_is_inside_rootfs() {
        let path = metadata_file_path("mybox");
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("rootfs"),
            "Metadata file path must be inside the container rootfs"
        );
        assert!(
            path_str.contains("melisa-info"),
            "Metadata file path must end with 'melisa-info'"
        );
        assert!(
            path_str.contains("mybox"),
            "Metadata file path must include the container name"
        );
    }

    #[test]
    fn test_temp_path_differs_from_final_path() {
        let final_path = metadata_file_path("box");
        let temp_path = metadata_temp_path("box");
        assert_ne!(
            final_path, temp_path,
            "Temp path and final path must be distinct to enable atomic rename"
        );
    }

    #[test]
    fn test_temp_path_has_tmp_suffix() {
        let temp_path = metadata_temp_path("box");
        let path_str = temp_path.to_string_lossy();
        assert!(
            path_str.ends_with(".tmp"),
            "Temp path must have a '.tmp' suffix to distinguish it from the final path"
        );
    }

    #[test]
    fn test_melisa_version_constant_is_semver_like() {
        // Verify the version string has at least two dots (X.Y.Z format).
        let dot_count = MELISA_VERSION.chars().filter(|&c| c == '.').count();
        assert!(
            dot_count >= 2,
            "MELISA_VERSION must follow semantic versioning (X.Y.Z) — got '{}'",
            MELISA_VERSION
        );
    }

    /// Verifies that `inspect_container_metadata` returns the correct error
    /// type when the metadata file does not exist.
    #[tokio::test]
    async fn test_inspect_metadata_returns_not_found_for_missing_container() {
        let result = inspect_container_metadata("THIS_CONTAINER_DOES_NOT_EXIST_12345").await;
        assert!(result.is_err(), "Must return an error for a non-existent container");
        match result.unwrap_err() {
            MelisaError::MetadataNotFound(name) => {
                assert!(
                    name.contains("THIS_CONTAINER_DOES_NOT_EXIST_12345"),
                    "Error must reference the container name"
                );
            }
            other => panic!("Expected MetadataNotFound, got: {:?}", other),
        }
    }
}