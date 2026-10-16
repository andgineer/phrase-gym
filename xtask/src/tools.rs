use std::fmt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Failure(String);

impl Failure {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn locate() -> Self {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self {
            root: manifest.parent().unwrap_or(&manifest).to_path_buf(),
        }
    }

    pub fn frontend(&self) -> PathBuf {
        self.root.join("frontend")
    }

    pub fn cargo(&self, arguments: &[&str]) -> Result<(), Failure> {
        run(Command::new(cargo())
            .args(arguments)
            .current_dir(&self.root))
    }

    pub fn cargo_output(&self, arguments: &[&str]) -> Result<String, Failure> {
        output(
            Command::new(cargo())
                .args(arguments)
                .current_dir(&self.root),
        )
    }

    pub fn npm(&self, arguments: &[&str]) -> Result<(), Failure> {
        run(Command::new(npm())
            .args(arguments)
            .current_dir(self.frontend()))
    }

    pub fn npx(&self, arguments: &[&str]) -> Result<(), Failure> {
        run(Command::new(npx())
            .args(arguments)
            .current_dir(self.frontend()))
    }

    pub fn require_rust_components(&self) -> Result<(), Failure> {
        for component in ["fmt", "clippy"] {
            self.cargo_output(&[component, "--version"]).map_err(|failure| {
                Failure::new(format!("cargo {component} is unavailable ({failure}); install the pinned toolchain components"))
            })?;
        }
        Ok(())
    }

    pub fn require_client_tooling(&self) -> Result<(), Failure> {
        let installed = output(Command::new("node").arg("--version"))
            .map_err(|failure| Failure::new(format!("Node.js is required: {failure}")))?;
        let package = std::fs::read_to_string(self.frontend().join("package.json"))
            .map_err(|error| Failure::new(format!("reading frontend/package.json: {error}")))?;
        let manifest: serde_json::Value = serde_json::from_str(&package)
            .map_err(|error| Failure::new(format!("parsing frontend/package.json: {error}")))?;
        let required = manifest["engines"]["node"].as_str().unwrap_or(">=0.0.0");
        check_node_version(installed.trim(), required)?;
        output(Command::new(npm()).arg("--version"))
            .map_err(|failure| Failure::new(format!("npm is required: {failure}")))?;
        if !self.frontend().join("node_modules").is_dir() {
            return Err(Failure::new(
                "client dependencies are missing; run `npm --prefix frontend ci`",
            ));
        }
        Ok(())
    }
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

// Windows exposes npm and npx as command shims rather than executables.
fn npm() -> &'static str {
    if cfg!(windows) { "npm.cmd" } else { "npm" }
}

fn npx() -> &'static str {
    if cfg!(windows) { "npx.cmd" } else { "npx" }
}

fn describe(command: &Command) -> String {
    let arguments: Vec<_> = command
        .get_args()
        .map(|argument| argument.to_string_lossy())
        .collect();
    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        arguments.join(" ")
    )
}

fn run(command: &mut Command) -> Result<(), Failure> {
    let description = describe(command);
    println!("xtask: {description}");
    let status = command
        .status()
        .map_err(|error| Failure::new(format!("cannot start `{description}`: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "`{description}` failed with {status}"
        )))
    }
}

fn output(command: &mut Command) -> Result<String, Failure> {
    let description = describe(command);
    let result = command
        .output()
        .map_err(|error| Failure::new(format!("cannot start `{description}`: {error}")))?;
    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
    } else {
        Err(Failure::new(format!(
            "`{description}` failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )))
    }
}

pub fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text
        .trim()
        .trim_start_matches(['v', '>', '=', '^', '~'])
        .split('.');
    let mut next = || parts.next().and_then(|part| part.parse().ok());
    Some((next()?, next().unwrap_or(0), next().unwrap_or(0)))
}

pub fn check_node_version(installed: &str, required: &str) -> Result<(), Failure> {
    let unparsable = || {
        Failure::new(format!(
            "cannot compare Node.js {installed} with {required}"
        ))
    };
    if !required.starts_with(">=") {
        return Err(unparsable());
    }
    let have = parse_version(installed).ok_or_else(unparsable)?;
    let need = parse_version(required).ok_or_else(unparsable)?;
    if have < need {
        return Err(Failure::new(format!(
            "Node.js {required} is required by frontend/package.json; found {installed}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_versions_meet_the_declared_engine_floor() {
        assert!(check_node_version("v22.23.3", ">=22.12.0").is_ok());
        assert!(check_node_version("v24.0.0", ">=22.12.0").is_ok());
        assert!(check_node_version("v20.15.1", ">=22.12.0").is_err());
        assert!(check_node_version("v22.11.9", ">=22.12.0").is_err());
        assert!(check_node_version("v22.23.3", "^22").is_err());
    }
}
