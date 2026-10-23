use serde_json::json;

use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant};

// Claude answers with a result envelope around the schema-validated object; Codex prints it bare.
pub(super) fn prepare(envelope: bool) -> ExitCode {
    let input: serde_json::Value =
        serde_json::from_slice(&std::fs::read("input.json").unwrap()).unwrap();
    let context = input["input"]["context"].as_str().unwrap_or("");
    if context == "fake-lease-loss" {
        std::fs::write("running.marker", b"started").unwrap();
        let started = Instant::now();
        while !Path::new("release.signal").exists() && started.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    if context == "fake-auth" {
        println!("Not logged in · Please run /login");
        return ExitCode::from(1);
    }
    if context == "fake-quota" {
        eprintln!("Individual quota reached");
        return ExitCode::from(1);
    }
    if context == "fake-malformed" {
        println!("not JSON");
        return ExitCode::SUCCESS;
    }
    if context == "fake-flood" {
        return super::flood(200_000);
    }
    if context == "fake-wait" {
        return super::pause("20000");
    }
    let mut output: serde_json::Value =
        serde_json::from_slice(&std::fs::read("output-shape.json").unwrap()).unwrap();
    if let Some(phrases) = output
        .get_mut("phrases")
        .and_then(serde_json::Value::as_array_mut)
    {
        for (index, phrase) in phrases.iter_mut().enumerate() {
            prepare_item(phrase, index);
        }
        // Real models do not always keep the submitted item order.
        phrases.reverse();
    } else {
        output["meaning_key"] = "begin-activity".into();
        output["intention_key"] = "open-meeting".into();
        output["situations"][0]["text"] = "Everyone has arrived. Open the meeting.".into();
        output["hint"] = if context == "fake-invalid-hint" {
            "unrelated"
        } else {
            "started"
        }
        .into();
        output["help"]["meaning"] = "Begin a shared activity.".into();
    }
    if envelope {
        println!(
            "{}",
            json!({"type": "result", "subtype": "success", "is_error": false, "num_turns": 3,
                   "result": output.to_string(), "structured_output": output,
                   "usage": {"input_tokens": 12, "output_tokens": 345}})
        );
    } else {
        println!("{output}");
    }
    if context == "fake-prefix-flood" {
        let _ = std::io::stdout().write_all(&vec![b' '; 200_000]);
    }
    ExitCode::SUCCESS
}

fn prepare_item(output: &mut serde_json::Value, index: usize) {
    output["meaning_key"] = format!("working-set-{index}").into();
    output["intention_key"] = format!("working-set-{index}").into();
    let (text, prompts, hint, help) = match index {
        0 => (
            "Let's get started.",
            vec![
                "Everyone has arrived. Open the meeting.",
                "Your group is ready. Begin the activity.",
            ],
            "started",
            "Begin a shared activity.",
        ),
        1 => (
            "Could you repeat that, please?",
            vec![
                "You did not hear your colleague. Ask them to say it again.",
                "The line broke up. Ask the speaker to say their last point again.",
            ],
            "repeat",
            "Politely ask to hear the same words again.",
        ),
        2 => (
            "I'll send the notes.",
            vec![
                "The meeting has ended. Offer to share the written summary.",
                "Your colleague needs the meeting summary. Promise to email it.",
            ],
            "notes",
            "Promise to share a written record of the meeting.",
        ),
        3 => (
            "Could we move the call to Friday?",
            vec![
                "The current meeting time is inconvenient. Suggest another day.",
                "Your appointment conflicts with other work. Ask to change its day.",
            ],
            "move",
            "Suggest changing an event to another time.",
        ),
        _ => (
            "Please clarify the deadline.",
            vec![
                "Please _____ the deadline.",
                "Can you _____ this point?",
                "We need to _____ the plan.",
            ],
            "cl",
            "Make something clearer or easier to understand.",
        ),
    };
    output["text"] = text.into();
    output["hint"] = hint.into();
    output["help"]["meaning"] = help.into();
    output["situations"] = json!(prompts.iter().enumerate().map(|(i, text)| json!({"id": format!("context-{i}"), "text": if index == 4 {format!("{text} | Make it clear.")} else {(*text).into()}, "language":"en", "audio_id": null, "partner": null, "exposed_target_source_ids": []})).collect::<Vec<_>>());
    if index == 3 {
        if let Some(source) = output["source_text"]
            .as_str()
            .filter(|source| !source.contains('{'))
        {
            output["text"] = source.to_owned().into();
        }
        output["task"] = json!({"kind":"frame", "pattern":"Could we move {item} to {day}?", "slots":[{"name":"item","values":["the call", "the meeting"]},{"name":"day","values":["Friday", "Monday"]}]});
    }
    if index == 4 {
        output["task"] = json!({"kind":"word_sense", "word":"clarify", "sense":help, "contexts":prompts.iter().enumerate().map(|(i, prompt)| json!({"situation_id":format!("context-{i}"),"answer":prompt.replace("_____", "clarify"),"reference_audio_id":"00000000-0000-0000-0000-000000000000"})).collect::<Vec<_>>()});
    }
}
