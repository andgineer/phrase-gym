use std::future::pending;
use std::path::Path;
use std::time::Duration;

use phrase_gym_coach::platform::files::sha256;
use phrase_gym_coach::process::{Ending, ProcessReport, StreamSummary};
use phrase_gym_coach::speech::{PiperVoice, SpeechRequest, stage};
use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::media::MediaRole;
use tempfile::TempDir;

fn fixture(root: &Path) -> (PiperVoice, SpeechRequest) {
    let model = b"fake model, never inference";
    let config = br#"{"language":{"code":"en_US"},"num_speakers":1,"audio":{"sample_rate":22050}}"#;
    let model_path = root.join("fake voice.onnx");
    let config_path = root.join("fake voice.json");
    std::fs::write(&model_path, model).unwrap();
    std::fs::write(&config_path, config).unwrap();
    (
        PiperVoice {
            executable: env!("CARGO_BIN_EXE_phrase-gym-fake-child").into(),
            runtime_version: "1.8.0".into(),
            model: model_path,
            model_sha256: sha256(model),
            config: config_path,
            config_sha256: sha256(config),
            voice: "synthetic-en".into(),
            locale: "en-US".into(),
        },
        SpeechRequest {
            language: Language::En,
            text: "Hello, world.".into(),
            speech_text: "Hello, world.".into(),
            source: "synthetic".into(),
            role: MediaRole::Reference,
        },
    )
}

#[tokio::test]
async fn stages_verified_bytes_and_returns_exact_pcm_as_wav() {
    let root = TempDir::new().unwrap();
    let (voice, request) = fixture(root.path());
    let staged = stage(root.path(), &voice, &request).await.unwrap();
    std::fs::write(&voice.model, "changed after staging").unwrap();
    assert_eq!(
        std::fs::read(staged.spec.cwd.join("voice.onnx")).unwrap(),
        b"fake model, never inference"
    );
    assert_eq!(
        std::fs::read(staged.spec.cwd.join("input.txt")).unwrap(),
        request.speech_text.as_bytes()
    );
    assert!(staged.spec.args.iter().any(|a| a == "--output-raw"));
    assert!(
        !staged
            .spec
            .env
            .iter()
            .any(|(key, _)| key == "OPENAI_API_KEY" || key == "PYTHONPATH")
    );
    let wav = staged.execute(pending()).await.unwrap();
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(&wav[24..28], &22050_u32.to_le_bytes());
    assert_eq!(&wav[44..], &[1, 0, 2, 0].repeat(2205));
}

#[tokio::test]
async fn validates_before_start_and_identity_changes_with_every_speech_condition() {
    let root = TempDir::new().unwrap();
    let (mut voice, mut request) = fixture(root.path());
    let original = stage(root.path(), &voice, &request)
        .await
        .unwrap()
        .cache_key;
    request.speech_text = "Hello again.".into();
    assert_ne!(
        original,
        stage(root.path(), &voice, &request)
            .await
            .unwrap()
            .cache_key
    );
    request.speech_text = request.text.clone();
    request.role = MediaRole::Partner;
    assert_ne!(
        original,
        stage(root.path(), &voice, &request)
            .await
            .unwrap()
            .cache_key
    );
    voice.locale = "de-DE".into();
    assert!(stage(root.path(), &voice, &request).await.is_err());
    voice.locale = "en-US".into();
    voice.model_sha256 = "0".repeat(64);
    assert!(stage(root.path(), &voice, &request).await.is_err());
    let (mut voice, _) = fixture(root.path());
    voice.config_sha256 = "0".repeat(64);
    assert!(stage(root.path(), &voice, &request).await.is_err());
    voice.voice = "serbski_institut".into();
    assert!(request.metadata(&voice).is_err());
}

#[tokio::test]
async fn rejects_process_failure_incomplete_silence_overflow_timeout_and_cancel() {
    let root = TempDir::new().unwrap();
    for input in ["fail", "odd", "silence", "flood", "wait"] {
        let (voice, mut request) = fixture(root.path());
        request.speech_text = input.into();
        let mut staged = stage(root.path(), &voice, &request).await.unwrap();
        if input == "wait" {
            staged.spec.limits.deadline = Duration::from_millis(50);
        }
        assert!(staged.execute(pending()).await.is_err(), "{input}");
    }
    let (voice, request) = fixture(root.path());
    let staged = stage(root.path(), &voice, &request).await.unwrap();
    assert!(staged.execute(async {}).await.is_err());
}

#[tokio::test]
async fn does_not_trust_success_with_open_or_truncated_streams_or_missing_pcm() {
    let root = TempDir::new().unwrap();
    let (voice, request) = fixture(root.path());
    let staged = stage(root.path(), &voice, &request).await.unwrap();
    let stream = StreamSummary {
        bytes: 4,
        truncated: false,
        closed: true,
    };
    let mut report = ProcessReport {
        ending: Ending::Exited(Some(0)),
        stdout: stream,
        stderr: stream,
        elapsed: Duration::ZERO,
    };
    assert!(staged.finish(report).is_err());
    std::fs::write(&staged.spec.stdout, [1, 0, 2, 0]).unwrap();
    report.stdout.closed = false;
    assert!(staged.finish(report).is_err());
    report.stdout.closed = true;
    report.stderr.truncated = true;
    assert!(staged.finish(report).is_err());
}

#[tokio::test]
async fn rejects_actual_runtime_mismatch_and_configuration_mismatch() {
    let root = TempDir::new().unwrap();
    let (mut voice, request) = fixture(root.path());
    let wrong = root
        .path()
        .join(format!("wrong-runtime{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(&voice.executable, &wrong).unwrap();
    voice.executable = wrong;
    assert!(
        stage(root.path(), &voice, &request)
            .await
            .err()
            .unwrap()
            .contains("does not report Piper")
    );
    let (mut voice, _) = fixture(root.path());
    for config in [
        r#"{"language":{"code":"de_DE"},"num_speakers":1,"audio":{"sample_rate":22050}}"#,
        r#"{"language":{"code":"en_US"},"num_speakers":2,"audio":{"sample_rate":22050}}"#,
        r#"{"language":{"code":"en_US"},"num_speakers":1,"audio":{"sample_rate":0}}"#,
    ] {
        std::fs::write(&voice.config, config).unwrap();
        voice.config_sha256 = sha256(config.as_bytes());
        assert!(stage(root.path(), &voice, &request).await.is_err());
    }
}

#[tokio::test]
async fn rejects_audio_beyond_duration_even_when_within_stream_byte_limit() {
    let root = TempDir::new().unwrap();
    let (voice, request) = fixture(root.path());
    let staged = stage(root.path(), &voice, &request).await.unwrap();
    let bytes = vec![1; 22_050 * 2 * 30 + 2];
    std::fs::write(&staged.spec.stdout, &bytes).unwrap();
    let stream = StreamSummary {
        bytes: bytes.len() as u64,
        truncated: false,
        closed: true,
    };
    assert!(
        staged
            .finish(ProcessReport {
                ending: Ending::Exited(Some(0)),
                stdout: stream,
                stderr: stream,
                elapsed: Duration::ZERO
            })
            .is_err()
    );
}

#[cfg(unix)]
#[tokio::test]
async fn rejects_symlinked_model_input() {
    let root = TempDir::new().unwrap();
    let (mut voice, request) = fixture(root.path());
    let alias = root.path().join("alias.onnx");
    std::os::unix::fs::symlink(&voice.model, &alias).unwrap();
    voice.model = alias;
    assert!(stage(root.path(), &voice, &request).await.is_err());
}
