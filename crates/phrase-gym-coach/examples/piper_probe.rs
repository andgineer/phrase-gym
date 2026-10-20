use std::future::pending;
use std::path::PathBuf;

use phrase_gym_coach::speech::{PiperVoice, SpeechRequest, stage};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Proof {
    voice: PiperVoice,
    request: SpeechRequest,
}

async fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [authorization, configuration, parent] = args.as_slice() else {
        return Err("Usage: piper_probe --authorized-local-synthesis CONFIG_JSON ABSOLUTE_STAGING_PARENT; starts one real synthesis, never plays audio".into());
    };
    if authorization != "--authorized-local-synthesis" || !PathBuf::from(parent).is_absolute() {
        return Err(
            "Explicit local-synthesis authorization and an absolute staging parent are required"
                .into(),
        );
    }
    let bytes =
        phrase_gym_coach::platform::files::read_selected(&PathBuf::from(configuration), 16_384)?;
    let proof: Proof = serde_json::from_slice(&bytes)
        .map_err(|_| "Invalid explicit voice/request configuration")?;
    let staged = stage(&PathBuf::from(parent), &proof.voice, &proof.request).await?;
    let wav = staged.execute(pending()).await?;
    let output = staged.spec.cwd.join("reference.wav");
    std::fs::write(&output, wav).map_err(|_| "Cannot save generated reference")?;
    phrase_gym_coach::platform::files::save(
        &staged.spec.cwd.join("metadata.json"),
        &staged.metadata,
    )?;
    println!(
        "One synthesis completed: {}\nCache identity: {}\nListening acceptance: pending; no audio was played or published.",
        output.display(),
        staged.cache_key
    );
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
