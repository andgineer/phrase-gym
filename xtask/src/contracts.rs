use std::path::PathBuf;

use crate::tools::{Failure, Workspace};

pub struct ContractFiles {
    openapi: PathBuf,
    typescript: PathBuf,
}

pub fn committed(workspace: &Workspace) -> ContractFiles {
    let directory = workspace.frontend().join("tests/rust-contract");
    ContractFiles {
        openapi: directory.join("openapi.json"),
        typescript: directory.join("api.generated.ts"),
    }
}

pub fn generate(workspace: &Workspace, files: &ContractFiles) -> Result<(), Failure> {
    let openapi = files.openapi.to_string_lossy();
    workspace.cargo(&[
        "run",
        "--quiet",
        "--locked",
        "--package",
        "phrase-gym-server",
        "--",
        "export-schema",
        &openapi,
    ])?;
    workspace.npx(&[
        "openapi-typescript",
        &openapi,
        "-o",
        &files.typescript.to_string_lossy(),
    ])
}

pub fn check_drift(workspace: &Workspace) -> Result<(), Failure> {
    let directory = workspace.root.join("target/xtask/contracts");
    let fresh = ContractFiles {
        openapi: directory.join("openapi.json"),
        typescript: directory.join("api.generated.ts"),
    };
    generate(workspace, &fresh)?;
    let committed = committed(workspace);
    for (expected, actual) in [
        (&committed.openapi, &fresh.openapi),
        (&committed.typescript, &fresh.typescript),
    ] {
        if read(expected)? != read(actual)? {
            return Err(Failure::new(format!(
                "{} is stale; run `cargo xtask api-types` and commit the result",
                expected.display()
            )));
        }
    }
    println!("xtask: generated contracts are current");
    Ok(())
}

fn read(path: &PathBuf) -> Result<String, Failure> {
    std::fs::read_to_string(path)
        .map(|text| text.replace("\r\n", "\n"))
        .map_err(|error| Failure::new(format!("reading {}: {error}", path.display())))
}
