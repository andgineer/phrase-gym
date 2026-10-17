use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetupCreateNew, wgpu};

use super::smoke::{self, RendererInfo, SmokeRun};
use super::startup::StartupChecks;
use super::state::{StartupEvent, StatusSnapshot, ThemeChoice, ViewState};
use super::view::{self, TITLE};
use super::{appearance, failure};
use crate::platform::instance::{InstanceError, InstanceGuard};
use crate::platform::paths::CoachPaths;

#[derive(Debug, PartialEq, Eq)]
pub struct LaunchOptions {
    pub smoke_report: Option<PathBuf>,
}

pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<LaunchOptions, String> {
    let mut arguments = arguments.into_iter();
    let mut options = LaunchOptions { smoke_report: None };
    while let Some(argument) = arguments.next() {
        // Older macOS versions append a process serial number when Finder launches an app.
        if argument.to_string_lossy().starts_with("-psn_") {
            continue;
        }
        if argument == "--smoke-test" {
            let report = arguments.next().ok_or("--smoke-test needs a report path")?;
            options.smoke_report = Some(PathBuf::from(report));
        } else {
            return Err(format!("unknown argument {}", argument.to_string_lossy()));
        }
    }
    Ok(options)
}

pub fn main() -> ExitCode {
    let options = match parse(std::env::args_os().skip(1)) {
        Ok(options) => options,
        Err(problem) => return refuse(None, &problem, smoke::EXIT_STARTUP),
    };
    let report = options.smoke_report;
    let guard = CoachPaths::for_current_user()
        .map_err(|error| error.to_string())
        .and_then(|paths| {
            paths.ensure().map_err(|error| error.to_string())?;
            InstanceGuard::acquire(&paths.instance_lock()).map_err(|error| match error {
                InstanceError::AlreadyRunning => failure::already_running().to_owned(),
                lock @ InstanceError::Lock { .. } => lock.to_string(),
            })
        });
    let _guard = match guard {
        Ok(guard) => guard,
        Err(problem) => return refuse(report.as_deref(), &problem, smoke::EXIT_INSTANCE),
    };
    let passed = Arc::new(AtomicBool::new(false));
    let outcome = Arc::clone(&passed);
    let smoke_report = report.clone();
    let result = eframe::run_native(
        TITLE,
        native_options(),
        Box::new(move |creation| Ok(Box::new(CoachApp::new(creation, smoke_report, outcome)))),
    );
    match (result, report) {
        (Err(error), report) => refuse(
            report.as_deref(),
            &failure::startup_message(&error),
            smoke::EXIT_STARTUP,
        ),
        (Ok(()), Some(_)) if !passed.load(Ordering::SeqCst) => ExitCode::from(smoke::EXIT_FAILED),
        (Ok(()), _) => ExitCode::SUCCESS,
    }
}

fn refuse(report: Option<&std::path::Path>, problem: &str, code: u8) -> ExitCode {
    match report {
        Some(path) => smoke::write_failure(path, problem),
        None => failure::show(problem),
    }
    ExitCode::from(code)
}

fn native_options() -> eframe::NativeOptions {
    let mut setup = WgpuSetupCreateNew::without_display_handle();
    // A status window has no need for a discrete GPU; the low-power adapter saves laptop battery.
    setup.power_preference =
        wgpu::PowerPreference::from_env().unwrap_or(wgpu::PowerPreference::LowPower);
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_inner_size([560.0, 560.0])
            .with_min_inner_size([380.0, 320.0])
            .with_resizable(true),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
            wgpu_setup: setup.into(),
            ..WgpuConfiguration::default()
        },
        ..eframe::NativeOptions::default()
    }
}

struct CoachApp {
    view: ViewState,
    events: Receiver<StartupEvent>,
    smoke: Option<SmokeRun>,
}

impl CoachApp {
    fn new(
        creation: &eframe::CreationContext<'_>,
        smoke_report: Option<PathBuf>,
        passed: Arc<AtomicBool>,
    ) -> Self {
        let context = creation.egui_ctx.clone();
        appearance::install(&context);
        appearance::apply(&context, ThemeChoice::System, 1.0);
        let waker = context.clone();
        let events = StartupChecks::native().spawn(move || waker.request_repaint());
        let renderer = creation
            .wgpu_render_state
            .as_ref()
            .map(|state| RendererInfo::from(&state.adapter.get_info()));
        let smoke = smoke_report.map(|path| SmokeRun::start(path, renderer, context, passed));
        Self {
            view: ViewState::new(StatusSnapshot::starting()),
            events,
            smoke,
        }
    }
}

impl eframe::App for CoachApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.events.try_recv() {
            self.view.snapshot.apply(event);
        }
        view::show(ui, &mut self.view);
        if let Some(smoke) = &mut self.smoke {
            smoke.on_frame(ui.ctx(), &self.view.snapshot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_the_smoke_test_option() {
        assert_eq!(parse([]), Ok(LaunchOptions { smoke_report: None }));
        let parsed = parse(["--smoke-test".into(), "/tmp/Ünï report.json".into()]).unwrap();
        assert_eq!(
            parsed.smoke_report,
            Some(PathBuf::from("/tmp/Ünï report.json"))
        );
        assert!(parse(["--smoke-test".into()]).is_err());
        assert_eq!(
            parse(["-psn_0_12345".into()]),
            Ok(LaunchOptions { smoke_report: None })
        );
        assert!(parse(["--verbose".into()]).is_err());
    }
}
