#![windows_subsystem = "windows"]

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn main() -> std::process::ExitCode {
    phrase_gym_coach::desktop::app::main()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn main() -> std::process::ExitCode {
    eprintln!("Phrase Gym Coach supports macOS and Windows only.");
    std::process::ExitCode::FAILURE
}
