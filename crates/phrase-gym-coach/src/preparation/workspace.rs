use std::path::{Path, PathBuf};

use crate::platform::{files, paths::CoachPaths};

pub fn save(paths: &CoachPaths, workspace: &Path) -> Result<PathBuf, String> {
    let workspace = workspace
        .canonicalize()
        .map_err(|_| "Cannot resolve Coach repository workspace")?;
    if !workspace.join(".git").exists() || !workspace.join("Cargo.toml").is_file() {
        return Err("Choose the Phrase Gym repository as the Coach workspace".into());
    }
    files::save(&paths.config.join("workspace.json"), &workspace)?;
    Ok(workspace)
}

pub fn load(paths: &CoachPaths) -> Result<PathBuf, String> {
    let workspace: PathBuf = serde_json::from_slice(&files::read_selected(&paths.config.join("workspace.json"), 4096)?)
        .map_err(|_| "Run phrase-gym-add-text once from the Phrase Gym repository to configure its workspace")?;
    if !workspace.is_absolute() || !workspace.join(".git").exists() {
        return Err("Configured Coach repository workspace is unavailable".into());
    }
    Ok(workspace)
}
