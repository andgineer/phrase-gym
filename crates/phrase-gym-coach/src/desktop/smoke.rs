use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, sync_channel};
use std::time::{Duration, Instant};

use eframe::egui_wgpu::wgpu;
use serde::Serialize;

use super::smoke_checks::{self, ProcessChecks, TERMINAL_CANARY};
use super::state::{FolderHealth, StatusSnapshot};
use super::text;
use crate::platform::paths::CoachPaths;
use crate::platform::resources;

pub const EXIT_FAILED: u8 = 1;
pub const EXIT_STARTUP: u8 = 2;
pub const EXIT_INSTANCE: u8 = 3;
const IDLE_WINDOW: Duration = Duration::from_secs(3);
// The wake-up frame that ends the idle window, plus one for a layout pass egui may request.
const IDLE_FRAME_ALLOWANCE: u64 = 2;

#[derive(Clone, Debug, Serialize)]
pub struct RendererInfo {
    pub backend: String,
    pub device_type: String,
    pub name: String,
    pub driver: String,
}

impl From<&wgpu::AdapterInfo> for RendererInfo {
    fn from(info: &wgpu::AdapterInfo) -> Self {
        Self {
            backend: format!("{:?}", info.backend),
            device_type: format!("{:?}", info.device_type),
            name: info.name.clone(),
            driver: format!("{} {}", info.driver, info.driver_info)
                .trim()
                .to_owned(),
        }
    }
}

#[derive(Serialize)]
struct Report<'a> {
    passed: bool,
    version: &'static str,
    os: &'static str,
    arch: &'static str,
    executable: Option<PathBuf>,
    resources: Option<PathBuf>,
    terminal_canary_inherited: bool,
    launch_variables: Vec<&'static str>,
    renderer: Option<&'a RendererInfo>,
    idle_window_ms: u128,
    idle_frames: u64,
    total_frames: u64,
    folders: String,
    instance_lock: Option<PathBuf>,
    checks: Option<&'a ProcessChecks>,
    startup_error: Option<&'a str>,
    intake: Option<&'a Result<(), String>>,
}

// Drives the real window through startup, an idle period and the bundled process checks, then
// writes a JSON report and closes; used to verify packaged launches without a person watching.
pub struct SmokeRun {
    report: PathBuf,
    renderer: Option<RendererInfo>,
    passed: Arc<AtomicBool>,
    checks: Receiver<ProcessChecks>,
    finished_checks: Option<ProcessChecks>,
    frames: u64,
    idle_start: Option<(Instant, u64)>,
    done: bool,
    intake_expected: bool,
    intake: Option<Result<(), String>>,
}

impl SmokeRun {
    pub fn start(
        report: PathBuf,
        renderer: Option<RendererInfo>,
        context: egui::Context,
        passed: Arc<AtomicBool>,
        intake_expected: bool,
    ) -> Self {
        let (sender, checks) = sync_channel(1);
        std::thread::spawn(move || {
            let executable = std::env::current_exe().unwrap_or_default();
            let _ = sender.send(smoke_checks::run_all(&executable));
            context.request_repaint();
        });
        Self {
            report,
            renderer,
            passed,
            checks,
            finished_checks: None,
            frames: 0,
            idle_start: None,
            done: false,
            intake_expected,
            intake: None,
        }
    }

    pub fn intake_finished(&mut self, result: Result<(), String>) {
        if self.intake_expected {
            self.intake = Some(result);
        }
    }

    pub fn on_frame(&mut self, context: &egui::Context, snapshot: &StatusSnapshot) {
        self.frames += 1;
        if let Ok(checks) = self.checks.try_recv() {
            self.finished_checks = Some(checks);
        }
        if self.done
            || self.finished_checks.is_none()
            || !snapshot.settled()
            || (self.intake_expected && self.intake.is_none())
        {
            return;
        }
        let Some((started, frames_at_start)) = self.idle_start else {
            self.idle_start = Some((Instant::now(), self.frames));
            context.request_repaint_after(IDLE_WINDOW);
            return;
        };
        let elapsed = started.elapsed();
        if elapsed < IDLE_WINDOW {
            context.request_repaint_after(IDLE_WINDOW.saturating_sub(elapsed));
            return;
        }
        let idle_frames = self.frames - frames_at_start;
        self.finish(snapshot, idle_frames, elapsed);
        context.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn finish(&mut self, snapshot: &StatusSnapshot, idle_frames: u64, elapsed: Duration) {
        self.done = true;
        let checks = self.finished_checks.as_ref();
        let folders_ready = matches!(snapshot.folders, FolderHealth::Ready(_));
        let passed = self.renderer.is_some()
            && checks.is_some_and(ProcessChecks::passed)
            && idle_frames <= IDLE_FRAME_ALLOWANCE
            && folders_ready
            && self.intake.as_ref().is_none_or(Result::is_ok);
        let executable = std::env::current_exe().ok();
        let report = Report {
            passed,
            version: env!("CARGO_PKG_VERSION"),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            resources: executable.as_deref().map(resources::resource_directory),
            executable,
            terminal_canary_inherited: std::env::var_os(TERMINAL_CANARY).is_some(),
            launch_variables: smoke_checks::launch_variables_received(),
            renderer: self.renderer.as_ref(),
            idle_window_ms: elapsed.as_millis(),
            idle_frames,
            total_frames: self.frames,
            folders: text::folders(&snapshot.folders)
                .into_iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("; "),
            instance_lock: CoachPaths::for_current_user()
                .ok()
                .map(|paths| paths.instance_lock()),
            checks,
            startup_error: None,
            intake: self.intake.as_ref(),
        };
        write(&self.report, &report);
        self.passed.store(passed, Ordering::SeqCst);
    }
}

pub fn write_failure(path: &Path, problem: &str) {
    let report = Report {
        passed: false,
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        executable: std::env::current_exe().ok(),
        resources: None,
        terminal_canary_inherited: std::env::var_os(TERMINAL_CANARY).is_some(),
        launch_variables: smoke_checks::launch_variables_received(),
        renderer: None,
        idle_window_ms: 0,
        idle_frames: 0,
        total_frames: 0,
        folders: String::new(),
        instance_lock: None,
        checks: None,
        startup_error: Some(problem),
        intake: None,
    };
    write(path, &report);
}

fn write(path: &Path, report: &Report<'_>) {
    if let Ok(text) = serde_json::to_string_pretty(report) {
        let _ = std::fs::write(path, text);
    }
}
