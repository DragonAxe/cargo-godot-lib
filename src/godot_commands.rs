use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::{which, which_in_global};

pub fn run_godot_import_if_needed(
    godot_project_path: &Path,
    godot_version: Option<&str>,
) -> Result<()> {
    if !godot_project_path.join(".godot").exists() {
        run_godot_import(godot_project_path, godot_version)
    } else {
        Ok(())
    }
}

pub fn run_godot_import(godot_project_path: &Path, godot_version: Option<&str>) -> Result<()> {
    let mut command = godot_command(godot_version)?;

    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(godot_project_path)
        .arg("--import")
        .arg("--headless");
    let status = command
        .spawn()
        .with_context(|| format!("Failed to spawn Godot import process: {:?}", command))?
        .wait()
        .with_context(|| format!("Failed to wait for Godot import process: {:?}", command))?;

    if !status.success() {
        Err(anyhow!(
            "Godot import process failed with exit code `{}`.\n\
            Possible cause: Known bug in Godot 4.5.1: \"Headless import of project with GDExtensions crashes\"\n\
            See: https://github.com/godotengine/godot/issues/111645\n\
            Try re-running if `.godot` folder was generated successfully.",
            status
                .code()
                .map(|e| e.to_string())
                .unwrap_or("unknown".to_string())
        ))
    } else {
        Ok(())
    }
}

pub fn run_godot(
    godot_project_path: &Path,
    godot_version: Option<&str>,
    args: &[String],
) -> Result<()> {
    let mut command = godot_command(godot_version)?;

    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(godot_project_path)
        .args(args)
        .spawn()
        .context("Failed to spawn Godot process")?
        .wait()
        .context("Failed to wait for Godot process")?;

    if !status.success() {
        let code = status.code().context("Godot process exited")?;
        Err(anyhow!("Godot process exited with exit code {}\nCommand: {:?}", code, command))
    } else {
        Ok(())
    }
}

/// Returns a Command for running godot with the specified version (using `gdenv run <version>`),
/// or the default godot binary if no version is provided.
fn godot_command(godot_version: Option<&str>) -> Result<Command> {
    Ok(if let Some(version) = godot_version {
        let mut cmd = Command::new("gdenv");
        cmd.arg("run").arg(version);
        cmd
    } else {
        Command::new(godot_binary_path()?)
    })
}

/// Looks for a godot executable in the following places:
/// - `godot` environment variable.
/// - `GODOT` environment variable.
/// - `godot` executable in the PATH.
/// - `godot` executable in the following common paths for linux and osx: `/usr/local/bin:/usr/bin:/bin:/Applications/Godot.app/Contents/MacOS`.
fn godot_binary_path() -> Result<PathBuf> {
    if let Ok(godot_binary_path) = std::env::var("godot") {
        return Ok(PathBuf::from(godot_binary_path));
    }

    if let Ok(godot_binary_path) = std::env::var("GODOT") {
        return Ok(PathBuf::from(godot_binary_path));
    }

    if let Ok(godot_binary_path) = which("godot") {
        return Ok(godot_binary_path);
    }

    // Search in some reasonable locations across linux and osx for godot.
    // Windows is trickier, as I believe the binary name contains the version
    // of godot, e.g., C:\\Program Files\\Godot\\Godot_v3.4.2-stable_win64.exe
    let godot_search_paths = "/usr/local/bin:/usr/bin:/bin:/Applications/Godot.app/Contents/MacOS";

    if let Some(godot_binary_path) = which_in_global("godot", Some(godot_search_paths))
        .ok()
        .and_then(|it| it.into_iter().next())
    {
        return Ok(godot_binary_path);
    }

    Err(anyhow!(
        concat!(
            "Couldn't find the godot binary. Searched in the following locations:\n",
            "    - `godot or `GODOT` environment variables.\n",
            "    - `$PATH` locations.\n",
            "    - Default search locations ({godot_search_paths:?}).\n",
            "  Tip: Consider using `gdenv` to manage your godot installations",
            " (https://github.com/bytemeadow/gdenv)."
        ),
        godot_search_paths = godot_search_paths
    ))
}
