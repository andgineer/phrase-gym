use std::future::Future;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;
use std::time::Duration;

use phrase_gym_contracts::media::MediaMetadata;
use serde_json::Value;

use super::{MAX_AUDIO_BYTES, PiperVoice, SpeechRequest};
use crate::adapter::environment::{HostEnvironment, child_environment};
use crate::platform::files::read_selected;
use crate::platform::files::sha256 as digest;
use crate::process::{self, Ending, Limits, ProcessReport, ProcessSpec};

pub struct StagedSpeech {
    pub spec: ProcessSpec,
    pub metadata: MediaMetadata,
    pub cache_key: String,
    rate: u32,
}

pub async fn stage(
    parent: &Path,
    voice: &PiperVoice,
    request: &SpeechRequest,
) -> Result<StagedSpeech, String> {
    let metadata = request.metadata(voice)?;
    let executable = voice.executable.clone();
    if !executable.is_file() {
        return Err("Select a Piper executable file".into());
    }
    let parent = parent
        .canonicalize()
        .map_err(|_| "Speech staging parent must exist")?;
    if parent.ancestors().any(|p| p.join(".git").exists()) {
        return Err("Speech staging must be outside repositories".into());
    }
    let model = read_selected(&voice.model, 128 * 1024 * 1024)?;
    let config = read_selected(&voice.config, 65_536)?;
    if digest(&model) != voice.model_sha256 || digest(&config) != voice.config_sha256 {
        return Err("Piper model or configuration checksum mismatch".into());
    }
    let settings: Value =
        serde_json::from_slice(&config).map_err(|_| "Invalid Piper configuration")?;
    if settings["language"]["code"]
        .as_str()
        .map(|s| s.replace('_', "-"))
        != Some(voice.locale.clone())
        || settings["num_speakers"].as_u64() != Some(1)
    {
        return Err("Piper configuration must match the locale and contain one speaker".into());
    }
    let rate = settings["audio"]["sample_rate"]
        .as_u64()
        .filter(|r| (8_000..=48_000).contains(r))
        .and_then(|r| u32::try_from(r).ok())
        .ok_or("Unsupported Piper sample rate")?;
    let directory = parent.join(format!("speech-{}", uuid::Uuid::new_v4()));
    private_directory(&directory)?;
    let environment = child_environment(
        &HostEnvironment::current(),
        executable.parent().ok_or("Invalid interpreter path")?,
    );
    let runtime = super::runtime::verify(&executable, &directory, environment.clone()).await?;
    let mut metadata = metadata;
    metadata.settings.insert(
        "runtime_identity".into(),
        phrase_gym_contracts::media::SettingValue::Text(runtime),
    );
    // Stage the verified bytes so a later model/config edit cannot change this attempt's identity.
    for (name, bytes) in [
        ("voice.onnx", model.as_slice()),
        ("voice.onnx.json", config.as_slice()),
        ("input.txt", request.speech_text.as_bytes()),
    ] {
        std::fs::write(directory.join(name), bytes).map_err(|_| "Cannot stage speech inputs")?;
    }
    let cache_key =
        digest(&serde_json::to_vec(&metadata).map_err(|_| "Cannot encode speech identity")?);
    Ok(StagedSpeech {
        spec: ProcessSpec {
            program: executable.clone(),
            args: [
                "-I",
                "-m",
                "piper",
                "--model",
                "voice.onnx",
                "--config",
                "voice.onnx.json",
                "--input-file",
                "input.txt",
                "--output-raw",
                "--speaker",
                "0",
                "--length-scale",
                "1",
                "--noise-scale",
                "0.667",
                "--noise-w-scale",
                "0.8",
                "--volume",
                "1",
                "--sentence-silence",
                "0.2",
            ]
            .map(Into::into)
            .to_vec(),
            cwd: directory.clone(),
            env: environment,
            stdout: directory.join("reference.pcm"),
            stderr: directory.join("stderr.log"),
            limits: Limits {
                output_bytes: MAX_AUDIO_BYTES,
                deadline: Duration::from_mins(1),
                grace: Duration::from_secs(1),
            },
        },
        metadata,
        cache_key,
        rate,
    })
}

fn private_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    let created = std::fs::DirBuilder::new().mode(0o700).create(path);
    #[cfg(not(unix))]
    let created = std::fs::create_dir(path);
    created.map_err(|_| "Cannot create private speech staging".into())
}

impl StagedSpeech {
    pub async fn execute(&self, cancel: impl Future<Output = ()>) -> Result<Vec<u8>, String> {
        let report = process::run(self.spec.clone(), cancel)
            .await
            .map_err(|_| "Cannot supervise Piper")?;
        self.finish(report)
    }

    pub fn finish(&self, report: ProcessReport) -> Result<Vec<u8>, String> {
        if report.ending != Ending::Exited(Some(0))
            || report.stdout.truncated
            || report.stderr.truncated
            || !report.stdout.closed
            || !report.stderr.closed
        {
            return Err("Piper failed, was cancelled, exceeded bounds or left open streams; no reference accepted".into());
        }
        let pcm = read_selected(&self.spec.stdout, MAX_AUDIO_BYTES)?;
        super::wav::from_pcm(&pcm, self.rate)
    }
}
