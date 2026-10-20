// Exercises ordinary text intake with native fake provider/speech processes; never discovers a CLI.
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_coach::adapter::{codex_readiness::CodexReady, environment::HostEnvironment};
use phrase_gym_coach::platform::{files, paths::CoachPaths};
use phrase_gym_coach::preparation::{self, selection::SelectedTool};
use phrase_gym_coach::speech::PiperVoice;
use phrase_gym_coach::transfer::{client::ServerClient, text, worker};

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [server, directory, mode @ ..] = arguments.as_slice() else {
        return Err("Usage: fake_text_worker LOOPBACK_SERVER TEMP_DIRECTORY".into());
    };
    let directory = PathBuf::from(directory)
        .canonicalize()
        .map_err(|_| "Missing test directory")?;
    let paths = CoachPaths {
        config: directory.join("config"),
        cache: directory.join("cache"),
        logs: directory.join("logs"),
        state: directory.join("state"),
    };
    paths.ensure().map_err(|e| e.to_string())?;
    let workspace = directory.join("workspace");
    std::fs::create_dir(&workspace).map_err(|_| "Cannot create test workspace")?;
    let fake = std::env::current_exe()
        .map_err(|_| "Cannot locate test executable")?
        .parent()
        .and_then(|p| p.parent())
        .ok_or("Missing test binaries")?
        .join(format!(
            "phrase-gym-fake-child{}",
            std::env::consts::EXE_SUFFIX
        ));
    let model = b"fake model, never inference";
    let config = br#"{"language":{"code":"en_US"},"num_speakers":1,"audio":{"sample_rate":22050}}"#;
    std::fs::write(directory.join("voice.onnx"), model)
        .map_err(|_| "Cannot stage synthetic model")?;
    std::fs::write(directory.join("voice.json"), config)
        .map_err(|_| "Cannot stage synthetic configuration")?;
    let voice = PiperVoice {
        executable: fake.clone(),
        runtime_version: "1.8.0".into(),
        model: directory.join("voice.onnx"),
        model_sha256: files::sha256(model),
        config: directory.join("voice.json"),
        config_sha256: files::sha256(config),
        voice: "synthetic".into(),
        locale: "en-US".into(),
    };
    files::save(&paths.config.join("piper-voice.json"), &voice)?;
    let stop = Arc::new(AtomicBool::new(false));
    let client = ServerClient::new(server)?;
    if mode.is_empty() {
        text::submit_raw(&paths, server, "Let's get started.", &client)?;
    }
    let tool = CodexReady {
        executable: fake,
        home: directory.join("normal-home"),
        host: HostEnvironment::default(),
    };
    let receipt = worker::tick(&client, &paths.state.join("worker.json"), |claim| {
        preparation::prepare_selected(
            &client,
            &paths,
            claim,
            &SelectedTool::codex(&tool, &workspace),
            &stop,
        )
    })?
    .ok_or("Expected an accepted text input")?;
    let manifest: serde_json::Value = client.get(&format!(
        "/api/v1/phrases/{}?language=en",
        receipt.target_ids[0]
    ))?;
    println!(
        "{}",
        serde_json::json!({"publication": receipt, "reference": manifest["content"]["reference_audio_id"]})
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
