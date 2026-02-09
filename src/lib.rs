pub mod gdextension_config;
pub mod godot_commands;

use crate::gdextension_config::GdExtensionConfig;
use crate::godot_commands::{godot_binary_path, run_godot_import_if_needed};
use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct GodotRunner {
    crate_name: String,
    godot_project_path: PathBuf,
    cargo_manifest_path: PathBuf,
    gdextension_config: Option<GdExtensionConfig>,
    write_gdextension_config: bool,
    pre_import: bool,
    godot_cli_arguments: Vec<String>,
}

impl GodotRunner {
    /// Example usage:
    /// ```rust,ignore
    /// let runner = cargo_godot_lib::GodotRunner::create(
    ///     env!("CARGO_PKG_NAME"),
    ///     &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    /// );
    /// if let Err(e) = runner.execute() {
    ///     eprintln!("{e:?}");
    ///     std::process::exit(1);
    /// }
    /// ```
    pub fn create(crate_name: &str, godot_project_path: &Path) -> Self {
        Self {
            crate_name: crate_name.to_string(),
            godot_project_path: godot_project_path.into(),
            cargo_manifest_path: Path::new("./Cargo.toml").into(),
            gdextension_config: None,
            write_gdextension_config: true,
            pre_import: true,
            godot_cli_arguments: vec![],
        }
    }

    /// Run Godot with the current configuration.
    pub fn execute(&self) -> Result<()> {
        let godot_project_path = self.godot_project_path.canonicalize().with_context(|| {
            format!(
                "Failed to canonicalize godot project path: {:?}",
                self.godot_project_path
            )
        })?;

        let godot_binary_path = godot_binary_path()?;

        if self.write_gdextension_config {
            let metadata = cargo_metadata::MetadataCommand::new()
                .manifest_path(&self.cargo_manifest_path)
                .exec()?;
            let default_config = GdExtensionConfig::start(
                &self.crate_name,
                &self.godot_project_path,
                metadata.target_directory.as_std_path(),
            );
            self.gdextension_config
                .clone()
                .unwrap_or(default_config)
                .build()
                .context("Failed to build .gdextension config")?
                .write()
                .context("Failed to write .gdextension file")?;
        }

        if self.pre_import {
            run_godot_import_if_needed(&godot_project_path)?;
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
            let code = status.code().context("Godot process exited")?;
            Err(anyhow!("Godot process exited with exit code {}", code))
        } else {
            Ok(())
        }
    }

    /// Specify the path to the cargo manifest. Default: `./Cargo.toml`.
    pub fn cargo_manifest_path(self, cargo_manifest_path: &Path) -> Self {
        Self {
            cargo_manifest_path: cargo_manifest_path.to_path_buf(),
            ..self
        }
    }

    /// Write the `.gdextension` config file before launching Godot. Default: true.
    /// See also: `gdextension_config`.
    pub fn write_gdextension_config(self, write_gdextension_config: bool) -> Self {
        Self {
            write_gdextension_config,
            ..self
        }
    }

    /// Replace the default configuration for the `.gdextension` file which is generated before Godot launch.
    /// See also: `write_gdextension_config`.
    pub fn gdextension_config(self, config: GdExtensionConfig) -> Self {
        Self {
            gdextension_config: Some(config),
            ..self
        }
    }

    /// Run `godot --import --headless` before launching Godot to create a `.godot` folder
    /// if it doesn't exist. Default: true.
    pub fn pre_import(self, pre_import: bool) -> Self {
        Self { pre_import, ..self }
    }

    /// Set additional arguments to the Godot CLI.
    /// See https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html
    /// for a list of available arguments.
    pub fn godot_cli_arguments(self, args: Vec<impl Into<String>>) -> Self {
        Self {
            godot_cli_arguments: args.into_iter().map(Into::into).collect(),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create() {
        let crate_name = "my_crate";
        let godot_project_path = PathBuf::from("godot_project");
        let runner = GodotRunner::create(crate_name, &godot_project_path);

        assert_eq!(runner.crate_name, crate_name);
        assert_eq!(runner.godot_project_path, godot_project_path);
        assert_eq!(runner.cargo_manifest_path, PathBuf::from("./Cargo.toml"));
        assert!(runner.gdextension_config.is_none());
        assert!(runner.write_gdextension_config);
        assert!(runner.pre_import);
        assert!(runner.godot_cli_arguments.is_empty());
    }

    #[test]
    fn test_builder_methods() {
        let runner = GodotRunner::create("a", Path::new("b"))
            .cargo_manifest_path(Path::new("custom/Cargo.toml"))
            .write_gdextension_config(false)
            .gdextension_config(GdExtensionConfig::default())
            .pre_import(false)
            .godot_cli_arguments(vec!["--hello", "world"]);

        assert_eq!(
            runner.cargo_manifest_path,
            PathBuf::from("custom/Cargo.toml")
        );
        assert!(!runner.write_gdextension_config);
        assert!(runner.gdextension_config.is_some());
        assert!(!runner.pre_import);
        assert_eq!(runner.godot_cli_arguments, vec!["--hello", "world"]);
    }

    #[test]
    fn test_gdextension_config_builder() {
        let dir = tempdir().unwrap();
        let godot_project_path = dir.path().join("godot");
        fs::create_dir(&godot_project_path).unwrap();

        let config = GdExtensionConfig::start("my_crate", &godot_project_path, Path::new("target"));
        let runner =
            GodotRunner::create("my_crate", &godot_project_path).gdextension_config(config);

        assert!(runner.gdextension_config.is_some());
    }

    #[test]
    fn test_execute_failure_invalid_project_path() {
        let runner = GodotRunner::create("my_crate", Path::new("non_existent_path"));
        let result = runner.execute();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to canonicalize godot project path")
        );
    }

    #[test]
    fn test_execute() {
        let dir = tempdir().unwrap();
        let godot_project_path = dir.path().join("godot");
        fs::create_dir(&godot_project_path).unwrap();
        copy_dir_all("mock_godot_project", &godot_project_path).unwrap();

        let runner = GodotRunner::create("my_crate", &godot_project_path)
            .godot_cli_arguments(vec!["--quit-after", "1", "--headless"]);

        // Godot will fail to find the gdextension file which is expected for this test's mock crate.
        runner.execute().unwrap();

        assert!(
            Path::new(&godot_project_path)
                .join("rust.gdextension")
                .exists()
        );
    }

    fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
