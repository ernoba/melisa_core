// ============================================================================
// src/deployment/manifest/mod.rs
// ============================================================================

pub mod parser;
pub mod types;
pub mod validator;

pub use parser::{load_mel_file, validate_manifest, MelParseError};
pub use types::{
    ContainerSection, DependencySection, HealthSection, LifecycleSection,
    MelManifest, PortSection, ProjectSection, ServiceDefinition, VolumeSection,
};
pub use validator::{validate_manifest_distro, validate_manifest_deployment};