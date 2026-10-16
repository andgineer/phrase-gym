mod architecture;
mod contracts;
mod modules;
mod tools;

use std::path::PathBuf;
use std::process::ExitCode;

use tools::{Failure, Workspace};

const USAGE: &str =
    "usage: cargo xtask <pre|test|api-types|app-build|run [--data-dir DIR] [--port PORT]>";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let workspace = Workspace::locate();
    let result = match arguments.first().map(String::as_str) {
        Some("pre") if arguments.len() == 1 => pre(&workspace),
        Some("test") if arguments.len() == 1 => test(&workspace),
        Some("api-types") if arguments.len() == 1 => api_types(&workspace),
        Some("app-build") if arguments.len() == 1 => app_build(&workspace),
        Some("run") => run(&workspace, &arguments[1..]),
        _ => Err(Failure::new(USAGE)),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("xtask: {failure}");
            ExitCode::FAILURE
        }
    }
}

fn pre(workspace: &Workspace) -> Result<(), Failure> {
    workspace.require_rust_components()?;
    workspace.require_client_tooling()?;
    workspace.cargo(&["fmt", "--all", "--check"])?;
    workspace.cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--locked",
        "--",
        "-D",
        "warnings",
    ])?;
    architecture::check(workspace)?;
    workspace.npm(&["run", "format:check"])?;
    contracts::check_drift(workspace)
}

fn test(workspace: &Workspace) -> Result<(), Failure> {
    workspace.require_client_tooling()?;
    workspace.cargo(&["test", "--workspace", "--all-features", "--locked"])?;
    for script in ["typecheck", "test", "build", "test:browser"] {
        workspace.npm(&["run", script])?;
    }
    Ok(())
}

fn api_types(workspace: &Workspace) -> Result<(), Failure> {
    workspace.require_client_tooling()?;
    contracts::generate(workspace, &contracts::committed(workspace))
}

fn app_build(workspace: &Workspace) -> Result<(), Failure> {
    workspace.require_client_tooling()?;
    workspace.npm(&["run", "typecheck"])?;
    workspace.npm(&["run", "build"])?;
    workspace.cargo(&[
        "build",
        "--release",
        "--locked",
        "--package",
        "phrase-gym-server",
    ])?;
    workspace.cargo(&[
        "build",
        "--release",
        "--locked",
        "--package",
        "phrase-gym-coach",
    ])
}

fn run(workspace: &Workspace, options: &[String]) -> Result<(), Failure> {
    let mut data_dir = workspace.root.join(".data/rust-server");
    let mut port = String::from("8010");
    let mut remaining = options.iter();
    while let Some(option) = remaining.next() {
        let value = remaining
            .next()
            .ok_or_else(|| Failure::new(format!("{option} needs a value; {USAGE}")))?;
        match option.as_str() {
            "--data-dir" => data_dir = PathBuf::from(value),
            "--port" => port.clone_from(value),
            _ => return Err(Failure::new(USAGE)),
        }
    }
    let data_dir = data_dir.to_string_lossy().into_owned();
    workspace.cargo(&[
        "run",
        "--locked",
        "--package",
        "phrase-gym-server",
        "--",
        "serve",
        "--data-dir",
        &data_dir,
        "--port",
        &port,
    ])
}
