// ============================================================================
// src/deployment/manifest/mod.rs
// ============================================================================

pub mod parser;
pub mod types;

pub use parser::{load_mel_file, validate_manifest, MelParseError};
pub use types::{
    ContainerSection, DependencySection, HealthSection, LifecycleSection,
    MelManifest, PortSection, ProjectSection, ServiceDefinition, VolumeSection,
};