pub mod gdextension_config;
pub mod godot_commands;

use crate::gdextension_config::GdExtensionConfig;
use crate::godot_commands::{godot_binary_path, run_godot_import_if_needed};
use anyhow::{Context, Result, anyhow};
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
    /// ```rust,ignore
    /// cargo_godot_lib::GodotRunner::create(
    ///     env!("CARGO_PKG_NAME"),
    ///     &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    /// )
    /// .expect("Failed to create Godot run configuration")
    /// .execute()
    /// .expect("Failed to execute Godot");
    /// ```
    pub fn create(crate_name: &str, godot_project_path: &Path) -> Result<Self> {
        let manifest_path = Path::new("./Cargo.toml");
        Self::create_with_manifest(crate_name, godot_project_path, manifest_path)
    }

    /// Like `create`, but allows specifying a custom cargo manifest path (i.e., path to `Cargo.toml`).
    pub fn create_with_manifest(
        crate_name: &str,
        godot_project_path: &Path,
        cargo_manifest_path: &Path,
    ) -> Result<Self> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(cargo_manifest_path)
            .exec()?;
        Ok(Self {
            godot_project_path: Some(godot_project_path.canonicalize().with_context(|| {
                format!(
                    "Failed to canonicalize godot project path: {:?}",
                    godot_project_path
                )
            })?),
            gdextension_config: Some(GdExtensionConfig::start(
                crate_name,
                godot_project_path,
                metadata.target_directory.as_std_path(),
            )),
            pre_import: true,
            godot_cli_arguments: vec!["--debug".to_string()], // Launch Godot with the local stdout debugger enabled
        })
    }

    /// Run Godot with the current configuration.
    pub fn execute(&self) -> Result<()> {
        let godot_project_path = self
            .godot_project_path
            .as_ref()
            .context("Godot project path not set.")?;
        let godot_binary_path = godot_binary_path()?;

        if let Some(gdextension_config) = &self.gdextension_config {
            let valid_gdextension_config = gdextension_config
                .build()
                .context("Failed to build .gdextension config")?;
            valid_gdextension_config
                .write()
                .context("Failed to write .gdextension file")?;
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
            .spawn()
            .context("Failed to spawn Godot process")?
            .wait()
            .context("Failed to wait for Godot process")?;

        if !status.success() {
            let code = status.code().context("Failed to get exit code")?;
            Err(anyhow!("Godot process failed with exit code {}", code))
        } else {
            Ok(())
        }
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
