#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unable to read cargo manifest: {0}")]
    Manifest(#[from] cargo_metadata::Error),

    #[error("Invalid Godot run configuration: {0}")]
    InvalidGodotRunConfig(String),

    #[error("Invalid GDExtension configuration: {0}")]
    InvalidGdExtensionConfig(String),

    #[error("Unable to find Godot binary: {0}")]
    GodotBinaryNotFound(String),

    #[error("Failed to import Godot project: {0}")]
    GodotImportFailed(String),

    #[error("Executing Godot failed: {0}")]
    GodotExecFailed(String),
}
