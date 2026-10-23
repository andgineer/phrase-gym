use std::path::PathBuf;

use phrase_gym_coach::platform::paths::CoachPaths;
use phrase_gym_coach::preparation::{
    attempt,
    readiness::{ClaudeTool, MODEL},
};
use phrase_gym_coach::transfer::material::{Material, RawLesson};
use phrase_gym_contracts::inbox::JobClaim;
use phrase_gym_contracts::media::MediaAsset;
use serde_json::json;
use uuid::Uuid;

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [destination] = arguments.as_slice() else {
        return Err(
            "Usage: prepare_dry_run ABSOLUTE_FRESH_DIRECTORY (never starts a model)".into(),
        );
    };
    let destination = PathBuf::from(destination);
    if !destination.is_absolute() || destination.exists() {
        return Err("Choose a fresh absolute directory outside any repository".into());
    }
    std::fs::create_dir(&destination).map_err(|_| "Cannot create dry-run directory")?;
    let paths = CoachPaths {
        config: destination.join("config"),
        cache: destination.join("cache"),
        logs: destination.join("logs"),
        state: destination.join("state"),
    };
    paths.ensure().map_err(|e| e.to_string())?;
    let readiness = attempt::private_directory(&paths)?;
    let tool = ClaudeTool::check(&readiness)?;
    let material = Material::Raw(
        serde_json::from_str::<RawLesson>(include_str!("../../../fixtures/coach/raw-english.json"))
            .map_err(|_| "Invalid fixture")?,
    );
    let media = MediaAsset {
        id: Uuid::from_u128(1),
        sha256: "ab".repeat(32),
        byte_length: 3244,
        mime_type: "audio/wav".into(),
        metadata: material.metadata(),
        available: true,
    };
    let input = material.submission(Uuid::from_u128(2), &media)?;
    let claim: JobClaim = serde_json::from_value(json!({
        "fence": {"generation_id": Uuid::from_u128(3), "claim_id": Uuid::from_u128(4), "token": Uuid::from_u128(5), "fence": 1},
        "lease_until": "2026-10-20T12:02:00Z", "deadline": (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339(),
        "receipt": {"generation_id": Uuid::from_u128(3), "submission_id": input.submission_id, "generation": 1, "namespace": "synthetic-dry-run", "profile_revision": 1, "guide_version": "english-claude-demo-v1", "received_at": "2026-10-20T12:00:00Z"},
        "input": input,
        "profile": {"revision": 1, "profile": {"native_language": "ru", "learning_timezone": "Europe/Belgrade", "languages": {"en": {"proficiency": "B2", "card_language": "en", "help_language": "en", "preparation_guidance": ""}}}, "mutation_id": null, "actor": "synthetic", "reason": "dry-run", "created_at": "2026-10-20T12:00:00Z", "restored_from": null},
        "guide": include_str!("../../phrase-gym-core/migrations/0005_preparation_guide.sql").split("', '").nth(1).ok_or("Missing guide")?.trim_end().trim_end_matches("');")
    })).map_err(|_| "Invalid synthetic pinned claim")?;
    let staged = attempt::stage(&paths, &claim, &tool)?;
    std::fs::write(
        destination.join("reference.wav"),
        include_bytes!("../../../fixtures/preservation/media/silence-100ms.wav"),
    )
    .map_err(|_| "Cannot stage synthetic clip")?;
    phrase_gym_coach::platform::files::save(&destination.join("claim.json"), &claim)?;
    println!(
        "Claude subscription login ready; version {}; model {MODEL}; no model started",
        tool.version
    );
    println!("Executable: {}", tool.executable.display());
    println!("Attempt: {}", staged.directory.display());
    println!("Arguments: {:?}", staged.spec.args);
    println!(
        "Limits: one start for the proposed measurement, 90 seconds, 64 KiB per stream; lease renewal every 30 seconds; null stdin; supplied silent clip only"
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            std::process::ExitCode::FAILURE
        }
    }
}
