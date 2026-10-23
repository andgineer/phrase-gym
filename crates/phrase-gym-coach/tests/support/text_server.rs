use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_coach::adapter::{codex_readiness::CodexReady, environment::HostEnvironment};
use phrase_gym_coach::preparation::{self, selection::SelectedTool};
use phrase_gym_coach::transfer::text;

use super::*;

#[test]
fn text_only_fake_codex_and_speech_publish_atomically_and_survive_server_restart() {
    let (directory, binary, data) = setup();
    let (server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let paths = super::generated_server::paths(directory.path());
    let workspace = directory.path().join("workspace");
    std::fs::create_dir(&workspace).unwrap();
    let model = b"synthetic";
    let config = br#"{"language":{"code":"en_US"},"num_speakers":1,"audio":{"sample_rate":22050}}"#;
    std::fs::write(directory.path().join("voice.onnx"), model).unwrap();
    std::fs::write(directory.path().join("voice.json"), config).unwrap();
    let voice = phrase_gym_coach::speech::PiperVoice {
        executable: env!("CARGO_BIN_EXE_phrase-gym-fake-child").into(),
        runtime_version: "1.8.0".into(),
        model: directory.path().join("voice.onnx"),
        model_sha256: phrase_gym_coach::platform::files::sha256(model),
        config: directory.path().join("voice.json"),
        config_sha256: phrase_gym_coach::platform::files::sha256(config),
        voice: "synthetic".into(),
        locale: "en-US".into(),
    };
    phrase_gym_coach::platform::files::save(&paths.config.join("piper-voice.json"), &voice)
        .unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let draft_path = paths.state.join("pending-text.json");
    let draft = text::TextDraft::retain(&draft_path, &url, "Let's get started.").unwrap();
    let accepted = text::submit_raw(&paths, &url, &draft.phrase, &client).unwrap();
    assert_eq!(accepted.submission_id, draft.submission_id);
    assert_eq!(
        std::fs::read_dir(&paths.cache).unwrap().count(),
        0,
        "Server acknowledges original text before synthesis"
    );
    // Simulate a crash after acknowledgement but before local outbox removal.
    phrase_gym_coach::platform::files::save(&draft_path, &draft).unwrap();
    assert_eq!(
        text::submit_raw(&paths, &url, &draft.phrase, &client).unwrap(),
        accepted
    );
    assert!(!draft_path.exists());
    let tool = CodexReady {
        executable: env!("CARGO_BIN_EXE_phrase-gym-fake-child").into(),
        home: directory.path().join("normal-home"),
        host: HostEnvironment::default(),
    };
    let file = paths.state.join("worker.json");
    let mut committed_checkpoint = None;
    let publication = phrase_gym_coach::transfer::worker::tick(&client, &file, |claim| {
        assert!(claim.input.phrases[0].reference_audio_id.is_none());
        let output = preparation::prepare_selected(&client, &paths, claim, &SelectedTool::codex(&tool, &workspace), &stop)?;
        committed_checkpoint = Some(serde_json::json!({"identity":client.identity,"claim_id":claim.fence.claim_id,"output":output}));
        Ok(output)
    }).unwrap().unwrap();
    let attempts = std::fs::read_dir(workspace.join(".phrase-gym-coach/attempts"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        std::fs::canonicalize(std::fs::read_to_string(attempts.join("launch-cwd.txt")).unwrap(),)
            .unwrap(),
        workspace.canonicalize().unwrap()
    );
    let arguments: Vec<String> =
        serde_json::from_slice(&std::fs::read(attempts.join("launch-arguments.json")).unwrap())
            .unwrap();
    assert!(
        arguments.windows(2).any(|pair| pair[0] == "--output-schema"
            && Path::new(&pair[1]) == attempts.join("schema.json"))
    );
    let staged_schema: serde_json::Value =
        serde_json::from_slice(&std::fs::read(attempts.join("schema.json")).unwrap()).unwrap();
    assert_eq!(
        staged_schema["properties"]["hint"]["enum"],
        serde_json::json!(["Let's", "get", "started"])
    );
    assert!(!draft_path.exists());
    phrase_gym_coach::platform::files::save(&file, &committed_checkpoint.unwrap()).unwrap();
    drop(server);
    let (_server, _) = start(&binary, &data, url.rsplit(':').next().unwrap());
    let recovered = phrase_gym_coach::transfer::worker::tick(&client, &file, |_| {
        panic!("Committed result must not restart providers")
    })
    .unwrap()
    .unwrap();
    assert_eq!(recovered, publication);
    assert_eq!(publication.target_ids.len(), 1);
    assert_eq!(
        std::fs::read_dir(&paths.cache).unwrap().count(),
        1,
        "Restart reuses generated speech"
    );
    let reference: serde_json::Value = client
        .get(&format!(
            "/api/v1/phrases/{}?language=en",
            publication.target_ids[0]
        ))
        .unwrap();
    assert_eq!(reference["lessons"][0]["title"], "Let's get started.");
    assert_eq!(reference["english"]["details"]["language"], "ru");
    text::TextDraft::retain(&draft_path, &url, "Could you repeat that, please?").unwrap();
}

#[test]
fn exact_raw_text_with_whitespace_retries_its_failed_generation_without_rewriting_original_input() {
    let (directory, binary, data) = setup();
    let (_server, url) = start(&binary, &data, "0");
    let client = ServerClient::new(&url).unwrap();
    let paths = super::generated_server::paths(directory.path());
    let original = "  Let's get started. \n";
    let receipt = text::submit_raw(&paths, &url, original, &client).unwrap();
    let claim: JobClaim = client
        .post::<_, Option<JobClaim>>(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: Uuid::new_v4(),
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(claim.input.context, original);
    assert_eq!(claim.input.phrases[0].text, original.trim());
    client
        .post::<_, ()>(
            "/api/v1/preparation/fail",
            &phrase_gym_contracts::inbox::JobFailure {
                fence: claim.fence,
                reason: "Synthetic speech setup failure".into(),
            },
        )
        .unwrap();
    let retried = text::submit_raw(&paths, &url, original, &client).unwrap();
    assert_eq!(retried.generation_id, receipt.generation_id);
    assert_eq!(retried.submission_id, receipt.submission_id);
    let next: JobClaim = client
        .post::<_, Option<JobClaim>>(
            "/api/v1/preparation/claims",
            &ClaimRequest {
                claim_id: Uuid::new_v4(),
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!(next.receipt.generation_id, receipt.generation_id);
    assert_eq!(next.receipt.submission_id, receipt.submission_id);
    assert_eq!(next.input.context, original);
    assert!(
        text::retry_preparation(&paths, &client, Some(original.trim()))
            .unwrap_err()
            .contains("exact phrase")
    );
}
