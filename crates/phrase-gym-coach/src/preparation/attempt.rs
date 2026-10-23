use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use phrase_gym_contracts::inbox::JobClaim;
use serde_json::json;
use uuid::Uuid;

use super::readiness::ClaudeTool;
use super::selection::SelectedTool;
use crate::adapter::invocation::{AttemptFiles, LaunchContext, ModelSelection};
use crate::platform::paths::CoachPaths;
use crate::process::{Limits, ProcessSpec};

pub struct StagedAttempt {
    pub directory: PathBuf,
    pub spec: ProcessSpec,
}

pub fn private_directory(paths: &CoachPaths) -> Result<PathBuf, String> {
    let directory = paths.attempts().join(Uuid::new_v4().to_string());
    fs::create_dir_all(&directory).map_err(|_| "Cannot create private preparation directory")?;
    #[cfg(unix)]
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|_| "Cannot protect preparation directory")?;
    Ok(directory)
}

pub fn stage(
    paths: &CoachPaths,
    claim: &JobClaim,
    tool: &ClaudeTool,
) -> Result<StagedAttempt, String> {
    stage_selected(paths, claim, &SelectedTool::claude(tool))
}

pub fn stage_selected(
    paths: &CoachPaths,
    claim: &JobClaim,
    tool: &SelectedTool<'_>,
) -> Result<StagedAttempt, String> {
    let deadline = chrono::DateTime::parse_from_rfc3339(&claim.deadline)
        .map_err(|_| "Server returned an invalid preparation deadline")?;
    if deadline
        .signed_duration_since(chrono::Utc::now())
        .num_seconds()
        < 110
    {
        return Err("Too little fenced attempt time remains for a safe CLI start; reconcile and explicitly retry preparation".into());
    }
    validate_input(claim)?;
    fs::create_dir_all(tool.attempts(paths)).map_err(|_| "Cannot create attempt cache")?;
    let directory = tool.attempts(paths).join(claim.fence.token.to_string());
    fs::create_dir(&directory).map_err(|_| "This attempt already started; reconcile its saved result before requesting another attempt")?;
    #[cfg(unix)]
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|_| "Cannot protect preparation directory")?;
    let files = AttemptFiles::new(directory.clone(), "prompt.md").map_err(|e| e.to_string())?;
    let schema = stage_inputs(&directory, &files, claim)?;
    let model = ModelSelection::new(tool.model, tool.effort).map_err(|e| e.to_string())?;
    let mut invocation = tool
        .adapter
        .invocation(&LaunchContext {
            executable: tool.executable,
            attempt: &files,
            model: &model,
            host: tool.host,
            codex_home: tool.home.unwrap_or(&paths.state),
        })
        .map_err(|e| e.to_string())?;
    tool.adapter
        .require_schema(&mut invocation, &directory.join("schema.json"), &schema);
    if let Some(workspace) = tool.workspace {
        invocation.cwd = workspace.to_path_buf();
    }
    let spec = ProcessSpec::from_invocation(
        invocation,
        directory.join("stdout.log"),
        directory.join("stderr.log"),
        Limits {
            output_bytes: 64 * 1024,
            deadline: Duration::from_secs(90),
            grace: Duration::from_secs(1),
        },
    );
    Ok(StagedAttempt { directory, spec })
}

fn stage_inputs(
    directory: &Path,
    files: &AttemptFiles,
    claim: &JobClaim,
) -> Result<String, String> {
    let data = json!({"input": claim.input, "profile": claim.profile, "guide_version": claim.receipt.guide_version, "guide": claim.guide});
    fs::write(
        directory.join("input.json"),
        serde_json::to_vec_pretty(&data).map_err(|_| "Cannot encode pinned input")?,
    )
    .map_err(|_| "Cannot stage pinned preparation input")?;
    let native = claim.profile.profile.native_language;
    let help = claim
        .profile
        .profile
        .languages
        .get(&claim.input.language)
        .and_then(|preferences| preferences.help_language)
        .ok_or("English help language is missing from the pinned profile")?;
    let mut schema = super::schema::batch_schema(&claim.input.phrases)?;
    super::schema::pin_help_languages(&mut schema, help, native);
    let schema = serde_json::to_string(&schema).map_err(|_| "Cannot encode pinned schema")?;
    fs::write(directory.join("schema.json"), &schema).map_err(|_| "Cannot stage output schema")?;
    let shapes: Vec<_> = claim
        .input
        .phrases
        .iter()
        .map(|input| super::prompt::example_shape(input, help, native))
        .collect();
    let shape = if shapes.len() == 1 {
        shapes[0].clone()
    } else {
        json!({"phrases": shapes})
    };
    fs::write(
        directory.join("output-shape.json"),
        serde_json::to_vec_pretty(&shape).map_err(|_| "Cannot encode output shape")?,
    )
    .map_err(|_| "Cannot stage output shape")?;
    fs::write(files.prompt(), super::prompt::text(directory))
        .map_err(|_| "Cannot stage preparation prompt")?;
    Ok(schema)
}

pub(super) fn validate_input(claim: &JobClaim) -> Result<(), String> {
    if !(1..=5).contains(&claim.input.phrases.len()) {
        return Err("Submit one to five English items".into());
    }
    if claim.input.language != phrase_gym_contracts::base::Language::En
        || claim.profile.revision != claim.receipt.profile_revision
        || claim.guide.trim().is_empty()
    {
        return Err("Preparation requires pinned English input, profile and guide".into());
    }
    let profile = claim
        .profile
        .profile
        .languages
        .get(&claim.input.language)
        .ok_or("English preparation preferences are missing")?;
    if profile.card_language != phrase_gym_contracts::base::Language::En
        || profile.help_language.is_none()
    {
        return Err("This bounded demo needs English card language and an explicit help language; update the personal profile through its authorized API".into());
    }
    Ok(())
}
