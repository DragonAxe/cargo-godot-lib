use crate::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::{which, which_in_global};

pub fn run_godot_import_if_needed(godot_project_path: &Path) -> Result<(), Error> {
    if !godot_project_path.join(".godot").exists() {
        return run_godot_import(godot_project_path);
    }
    Ok(())
}

pub fn run_godot_import(godot_project_path: &Path) -> Result<(), Error> {
    println!("+-------- Running Godot import");
    let godot_binary_path = godot_binary_path()?;
    let mut child = Command::new(godot_binary_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(godot_project_path)
        .arg("--import") // Launch Godot with local stdout debugger
        .arg("--headless") // Launch Godot with local stdout debugger
        .spawn()?;

    let status = child.wait()?;
    println!("+-------- Import complete");

    if !status.success() {
        let message = format!(
            "Godot import process failed with exit code `{}`.\n\
            Possible cause: Known bug in Godot 4.5.1: \"Headless import of project with GDExtensions crashes\"\n\
            See: https://github.com/godotengine/godot/issues/111645\n\
            Try re-running if `.godot` folder was generated successfully.",
            status
                .code()
                .map(|e| e.to_string())
                .unwrap_or("unknown".to_string())
        );
        return Err(Error::GodotImportFailed(message));
    }

    Ok(())
}

/// Looks for a godot executable in the following places:
/// - `godot` environment variable.
/// - `GODOT` environment variable.
/// - `godot` executable in the PATH.
/// - `godot` executable in the following common paths for linux and osx: `/usr/local/bin:/usr/bin:/bin:/Applications/Godot.app/Contents/MacOS`.
pub fn godot_binary_path() -> Result<PathBuf, Error> {
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

    Err(Error::InvalidGodotRunConfig(format!(
        concat!(
            "Couldn't find the godot binary. Searched in the following locations:\n",
            "    - `godot or `GODOT` environment variables.\n",
            "    - `$PATH` locations.\n",
            "    - Default search locations ({godot_search_paths:?}).\n",
            "  Tip: Consider using `gdenv` to manage your godot installations",
            " (https://github.com/bytemeadow/gdenv)."
        ),
        godot_search_paths = godot_search_paths
    )))
}
