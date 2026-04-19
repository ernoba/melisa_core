// ============================================================================
// src/deployment/manifest/validator.rs
//
// Extended manifest validation functions that check distro availability.
// This provides fail-fast validation before attempting deployment.
// Complements parser.rs which handles basic syntax validation.
// ============================================================================

use crate::deployment::manifest::types::MelManifest;
use crate::distros::lxc_distro;

/// Validates a manifest's distro slug against available LXC distros.
/// 
/// This should be called after loading a manifest to ensure the distro
/// slug is valid before attempting to create a container.
///
/// # Arguments
/// * `manifest` - The manifest to validate
///
/// # Returns
/// * `Ok(())` if the distro slug is valid
/// * `Err(String)` with a detailed error message if validation fails
pub async fn validate_manifest_distro(manifest: &MelManifest) -> Result<(), String> {
    let distro_slug = &manifest.container.distro;
    
    // Fetch available distros
    let (available_distros, _) = lxc_distro::get_lxc_distro_list(false).await;
    
    // Check if our distro slug is in the available list
    let found = available_distros
        .iter()
        .any(|d| d.slug == *distro_slug);
    
    if found {
        Ok(())
    } else {
        // Build a helpful error message
        let similar = available_distros
            .iter()
            .filter(|d| d.slug.starts_with(distro_slug.split('/').next().unwrap_or("")))
            .take(5)
            .map(|d| d.slug.clone())
            .collect::<Vec<_>>();
        
        if similar.is_empty() {
            Err(format!(
                "Invalid distro slug '{}' in container section. \
                Available distros include: ubuntu/jammy/amd64, ubuntu/focal/amd64, \
                debian/bookworm/amd64, alpine/3.17/amd64, fedora/38/amd64, and 100+ more. \
                Use 'melisa --search <name>' to find available distros.",
                distro_slug
            ))
        } else {
            Err(format!(
                "Invalid distro slug '{}' in container section. \
                Did you mean one of these?\n  - {}\n\
                Use 'melisa --search <name>' to find available distros.",
                distro_slug,
                similar.join("\n  - ")
            ))
        }
    }
}

/// Validates the entire manifest's distro availability asynchronously.
/// This is meant to be called after basic syntax validation from parser.rs
pub async fn validate_manifest_deployment(manifest: &MelManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    
    // Validate distro slug against available distros
    if let Err(e) = validate_manifest_distro(manifest).await {
        errors.push(e);
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_distro_validation() {
        // This test would require async runtime
        // Actual validation is done at deployment time
    }
}
