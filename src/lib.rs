pub mod error;
pub mod gdextension_config;
pub mod godot_commands;

use crate::error::Error;
use crate::gdextension_config::GdExtensionConfig;
use crate::godot_commands::{godot_binary_path, run_godot_import_if_needed};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct GodotRunner {
    godot_project_path: Option<PathBuf>,
    gdextension_config: Option<GdExtensionConfig>,
    pre_import: bool,
    godot_cli_arguments: Vec<String>,
}

impl GodotRunner {
    /// Example usage:
    /// ```rust
    /// cargo_godot::GodotRunner::create(
    ///     env!("CARGO_PKG_NAME"),
    ///     &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    /// )
    /// .expect("Failed to create Godot run configuration")
    /// .execute()
    /// .expect("Failed to execute Godot");
    /// ```
    pub fn create(crate_name: &str, godot_project_path: &Path) -> Result<Self, Error> {
        let manifest_path = Path::new("./Cargo.toml");
        Self::create_with_manifest(crate_name, godot_project_path, manifest_path)
    }

    /// Like `create`, but allows specifying a custom cargo manifest path (i.e., path to `Cargo.toml`).
    pub fn create_with_manifest(
        crate_name: &str,
        godot_project_path: &Path,
        cargo_manifest_path: &Path,
    ) -> Result<Self, Error> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(cargo_manifest_path)
            .exec()?;
        Ok(Self {
            godot_project_path: Some(godot_project_path.canonicalize()?),
            gdextension_config: Some(GdExtensionConfig::start(
                crate_name,
                godot_project_path,
                metadata.target_directory.as_std_path(),
            )),
            pre_import: true,
            godot_cli_arguments: vec!["--debug".to_string()], // Launch Godot with local stdout debugger
        })
    }

    /// Run Godot with the current configuration.
    pub fn execute(&self) -> Result<(), Error> {
        let godot_project_path =
            self.godot_project_path
                .as_ref()
                .ok_or(Error::InvalidGodotRunConfig(
                    "Godot project path not set.".to_string(),
                ))?;
        let godot_binary_path = godot_binary_path()?;

        if let Some(gdextension_config) = &self.gdextension_config {
            gdextension_config.build()?.write()?;
        }

        if self.pre_import {
            run_godot_import_if_needed(godot_project_path)?;
        }

        let status = Command::new(godot_binary_path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(godot_project_path)
            .args(&self.godot_cli_arguments)
            .spawn()?
            .wait()?;

        if !status.success() {
            return if let Some(code) = status.code() {
                Err(Error::GodotExecFailed(format!(
                    "Godot process failed with exit code {}",
                    code
                )))
            } else {
                Err(Error::GodotExecFailed(
                    "Godot process failed with unknown exit code".to_string(),
                ))
            };
        }
        Ok(())
    }

    /// Configure the `.gdextension` config file which is generated before launch by default.
    /// If `None` is provided, a `.gdextension` file will not be generated before launch.
    pub fn gdextension_config(self, config: Option<GdExtensionConfig>) -> Self {
        Self {
            gdextension_config: config,
            ..self
        }
    }

    /// Run `godot --import --headless` before launching Godot to create a `.godot` folder
    /// if it doesn't exist. Default: true.
    pub fn pre_import(self, pre_import: bool) -> Self {
        Self { pre_import, ..self }
    }

    /// Set additional arguments to the Godot CLI. Default: `--debug` for local stdout debugging.
    /// See https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html
    /// for a list of available arguments
    pub fn godot_cli_arguments(self, args: Vec<String>) -> Self {
        Self {
            godot_cli_arguments: args,
            ..self
        }
    }
}
