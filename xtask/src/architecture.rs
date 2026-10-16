use serde_json::Value;

use crate::modules::oversized_modules;
use crate::tools::{Failure, Workspace};

const DEPENDENCY_DIRECTION: &[(&str, &[&str])] = &[
    ("phrase-gym-contracts", &[]),
    ("phrase-gym-core", &["phrase-gym-contracts"]),
    (
        "phrase-gym-server",
        &["phrase-gym-contracts", "phrase-gym-core"],
    ),
    ("phrase-gym-coach", &["phrase-gym-contracts"]),
    ("xtask", &[]),
];
// GUI toolkits, agent/provider clients and model or speech engines belong to Coach, never to Server.
const FORBIDDEN_SERVER_CRATES: &[&str] = &[
    "phrase-gym-coach",
    "eframe",
    "egui*",
    "epaint",
    "emath",
    "winit",
    "wgpu*",
    "glow",
    "glutin*",
    "accesskit*",
    "rfd",
    "tray-icon",
    "muda",
    "reqwest",
    "ureq",
    "isahc",
    "async-openai",
    "openai*",
    "anthropic*",
    "ollama*",
    "ort",
    "ort-sys",
    "onnxruntime*",
    "candle*",
    "tch",
    "torch*",
    "tract*",
    "burn*",
    "whisper*",
    "llama*",
    "piper*",
    "tts",
    "rodio",
    "cpal",
    "vosk",
    "sherpa*",
    "coqui*",
];
const FORBIDDEN_SERVER_FEATURES: &[(&str, &str)] = &[("tokio", "process")];

pub fn check(workspace: &Workspace) -> Result<(), Failure> {
    let metadata =
        workspace.cargo_output(&["metadata", "--format-version", "1", "--no-deps", "--locked"])?;
    let metadata: Value = serde_json::from_str(&metadata)
        .map_err(|error| Failure::new(format!("parsing cargo metadata: {error}")))?;
    let tree = workspace.cargo_output(&[
        "tree",
        "--package",
        "phrase-gym-server",
        "--edges",
        "normal,build",
        "--target",
        "all",
        "--prefix",
        "none",
        "--format",
        "{p} [{f}]",
        "--locked",
    ])?;
    let mut problems = manifest_problems(&metadata);
    problems.extend(server_graph_problems(&tree));
    problems.extend(oversized_modules(&workspace.root)?);
    if problems.is_empty() {
        println!("xtask: architecture checks passed");
        Ok(())
    } else {
        Err(Failure::new(problems.join("\n")))
    }
}

pub fn manifest_problems(metadata: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let name = package["name"].as_str().unwrap_or_default();
        let Some((_, allowed)) = DEPENDENCY_DIRECTION
            .iter()
            .find(|(member, _)| *member == name)
        else {
            problems.push(format!(
                "workspace member {name} has no declared dependency direction"
            ));
            continue;
        };
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            let target = dependency["name"].as_str().unwrap_or_default();
            if dependency["path"].is_string() {
                if !allowed.contains(&target) {
                    problems.push(format!("{name} must not depend on {target}"));
                }
            } else if dependency["uses_default_features"].as_bool() != Some(false) {
                problems.push(format!(
                    "{name}: {target} must set default-features = false and list the features it needs"
                ));
            }
        }
    }
    problems
}

fn matches(pattern: &str, name: &str) -> bool {
    pattern
        .strip_suffix('*')
        .map_or(pattern == name, |prefix| name.starts_with(prefix))
}

pub fn server_graph_problems(tree: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for line in tree.lines() {
        let Some(name) = line.split_whitespace().next() else {
            continue;
        };
        if FORBIDDEN_SERVER_CRATES
            .iter()
            .any(|pattern| matches(pattern, name))
        {
            problems.push(format!("server dependency graph contains {name}"));
        }
        let features = line
            .split_once('[')
            .and_then(|(_, rest)| rest.split_once(']'))
            .map_or("", |(features, _)| features);
        for (krate, feature) in FORBIDDEN_SERVER_FEATURES {
            if name == *krate && features.split(',').any(|item| item == *feature) {
                problems.push(format!("server enables {krate} feature {feature}"));
            }
        }
    }
    problems.sort();
    problems.dedup();
    problems
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn package(name: &str, dependencies: &Value) -> Value {
        json!({"name": name, "dependencies": dependencies})
    }

    #[test]
    fn reports_reverse_dependencies_and_implicit_default_features() {
        let metadata = json!({"packages": [
            package("phrase-gym-core", &json!([
                {"name": "phrase-gym-server", "path": "/w/crates/phrase-gym-server", "uses_default_features": true},
                {"name": "rusqlite", "uses_default_features": true},
            ])),
            package("phrase-gym-coach", &json!([
                {"name": "phrase-gym-core", "path": "/w/crates/phrase-gym-core", "uses_default_features": true},
            ])),
            package("phrase-gym-server", &json!([
                {"name": "phrase-gym-core", "path": "/w/crates/phrase-gym-core", "uses_default_features": true},
                {"name": "axum", "uses_default_features": false},
            ])),
            package("phrase-gym-helpers", &json!([])),
        ]});
        assert_eq!(
            manifest_problems(&metadata),
            [
                "phrase-gym-core must not depend on phrase-gym-server",
                "phrase-gym-core: rusqlite must set default-features = false and list the features it needs",
                "phrase-gym-coach must not depend on phrase-gym-core",
                "workspace member phrase-gym-helpers has no declared dependency direction",
            ]
        );
    }

    #[test]
    fn rejects_gui_provider_and_subprocess_dependencies_in_the_server_graph() {
        let tree = "phrase-gym-server v0.0.0 [default]\n\
                    egui-wgpu v0.33.0 []\n\
                    reqwest v0.12.0 [json]\n\
                    tokio v1.53.3 [net,process,rt]\n\
                    ordered-float v5.0.0 []\n\
                    tokio v1.53.3 [net,process,rt] (*)\n";
        assert_eq!(
            server_graph_problems(tree),
            [
                "server dependency graph contains egui-wgpu",
                "server dependency graph contains reqwest",
                "server enables tokio feature process",
            ]
        );
        assert!(
            server_graph_problems("tokio v1.53.3 [net,rt,signal]\nhyper v1.12.0 [server]")
                .is_empty()
        );
    }
}
