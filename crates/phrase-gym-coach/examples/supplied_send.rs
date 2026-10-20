use std::path::PathBuf;

use phrase_gym_coach::transfer::{client::ServerClient, pending, runner};

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let [server, lesson, audio, retained] = arguments.as_slice() else {
        return Err("Usage: supplied_send SERVER LESSON_JSON ORIGINAL_AUDIO PENDING_JSON".into());
    };
    let client = ServerClient::new(server)?;
    let path = PathBuf::from(retained);
    let mut send = if let Some(retained) = pending::load(&path)? {
        retained
    } else {
        let json = String::from_utf8(phrase_gym_coach::platform::files::read_selected(
            &PathBuf::from(lesson),
            8_000,
        )?)
        .map_err(|_| "Lesson must be UTF-8 JSON")?;
        runner::create(&client.server, &json, &PathBuf::from(audio))?
    };
    let receipt = runner::deliver(&client, &path, &mut send, |phase| eprintln!("{phase}"))?;
    println!(
        "{}",
        serde_json::to_string(&receipt).map_err(|_| "Cannot encode receipt")?
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
