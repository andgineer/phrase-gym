use std::ffi::OsString;
use std::future::pending;
use std::path::Path;
use std::time::Duration;

use crate::platform::files::read_selected;
use crate::process::{self, Ending, Limits, ProcessSpec};

// Inspect package metadata without importing the synthesis engine or making a speech request.
const INSPECT: &str = "import importlib.metadata as m,sys; print(m.version('piper-tts')+'|'+m.version('onnxruntime')+'|'+sys.version)";

pub(super) async fn verify(
    interpreter: &Path,
    directory: &Path,
    env: Vec<(OsString, OsString)>,
) -> Result<String, String> {
    let interpreter_hash = crate::platform::files::sha256(&read_selected(
        &interpreter
            .canonicalize()
            .map_err(|_| "Cannot resolve Python interpreter")?,
        128 * 1024 * 1024,
    )?);
    let stdout = directory.join("runtime.txt");
    let report = process::run(
        ProcessSpec {
            program: interpreter.into(),
            args: ["-I", "-c", INSPECT].map(Into::into).to_vec(),
            cwd: directory.into(),
            env,
            stdout: stdout.clone(),
            stderr: directory.join("runtime-errors.log"),
            limits: Limits {
                output_bytes: 16_384,
                deadline: Duration::from_secs(15),
                grace: Duration::from_secs(1),
            },
        },
        pending(),
    )
    .await
    .map_err(|_| "Cannot inspect external Piper runtime")?;
    if report.ending != Ending::Exited(Some(0))
        || report.stdout.truncated
        || report.stderr.truncated
        || !report.stdout.closed
        || !report.stderr.closed
    {
        return Err(
            "External Python must provide Piper 1.8.0 and ONNX Runtime; no synthesis was started"
                .into(),
        );
    }
    let bytes = read_selected(&stdout, 16_384)?;
    let observed = std::str::from_utf8(&bytes)
        .map_err(|_| "Invalid runtime identity")?
        .trim();
    if !observed.starts_with("1.8.0|1.") || observed.split('|').count() != 3 {
        return Err("External interpreter does not report Piper 1.8.0 and ONNX Runtime 1.x".into());
    }
    Ok(format!("{interpreter_hash}|{observed}"))
}
