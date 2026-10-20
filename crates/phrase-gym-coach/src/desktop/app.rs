use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetupCreateNew, wgpu};

use super::intake::{Event, IntakeRunner};
use super::smoke::{self, RendererInfo, SmokeRun};
use super::startup::StartupChecks;
use super::state::{StartupEvent, StatusSnapshot, ThemeChoice, ViewState};
use super::view::{self, TITLE};
use super::{appearance, failure};
use crate::platform::instance::{InstanceError, InstanceGuard};
use crate::platform::paths::CoachPaths;

pub fn main() -> ExitCode {
    let options = match super::launch::parse(std::env::args_os().skip(1)) {
        Ok(options) => options,
        Err(problem) => return refuse(None, &problem, smoke::EXIT_STARTUP),
    };
    let report = options.smoke_report;
    let smoke_input = options.smoke_input;
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
        Box::new(move |creation| {
            Ok(Box::new(CoachApp::new(
                creation,
                smoke_report,
                outcome,
                smoke_input,
            )))
        }),
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
    intake: IntakeRunner,
}

impl CoachApp {
    fn new(
        creation: &eframe::CreationContext<'_>,
        smoke_report: Option<PathBuf>,
        passed: Arc<AtomicBool>,
        smoke_input: Option<super::smoke_intake::SmokeInput>,
    ) -> Self {
        let context = creation.egui_ctx.clone();
        appearance::install(&context);
        appearance::apply(&context, ThemeChoice::System, 1.0);
        let waker = context.clone();
        let events = StartupChecks::native().spawn(move || waker.request_repaint());
        let waker = context.clone();
        let intake_expected = smoke_input.is_some();
        let intake = IntakeRunner::spawn(
            move || waker.request_repaint(),
            smoke_report.is_none(),
            smoke_input,
        );
        let renderer = creation
            .wgpu_render_state
            .as_ref()
            .map(|state| RendererInfo::from(&state.adapter.get_info()));
        let smoke = smoke_report
            .map(|path| SmokeRun::start(path, renderer, context, passed, intake_expected));
        Self {
            view: ViewState::new(StatusSnapshot::starting()),
            events,
            smoke,
            intake,
        }
    }
}

impl eframe::App for CoachApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !self.view.intake.busy {
            for file in ui.ctx().input(|input| input.raw.dropped_files.clone()) {
                let path = file.path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    self.view.intake.command = Some(super::intake::Command::Load(
                        path.to_string_lossy().into_owned(),
                    ));
                } else {
                    self.view
                        .intake
                        .accept_audio(path.to_string_lossy().into_owned(), &self.view.draft);
                }
            }
        }
        while let Ok(event) = self.events.try_recv() {
            self.view.snapshot.apply(event);
        }
        while let Ok(event) = self.intake.events.try_recv() {
            match event {
                Event::Status(status) => self.view.intake.status = status,
                Event::SelectedClaude(path) => self.view.snapshot.apply(StartupEvent::Tool(
                    crate::adapter::CliAdapter::Claude,
                    super::state::ToolReadiness::Found(path),
                )),
                Event::Connected(server) => {
                    self.view.intake.server = server;
                    self.view.intake.connected = true;
                    self.view.snapshot.server_configured = true;
                    self.view.snapshot.processing_enabled = true;
                }
                Event::Draft(json) => {
                    self.view.intake.preview = super::preview::LessonPreview::parse(&json).ok();
                    self.view.intake.prepared = Some(json);
                }
                Event::Audio(path) => self.view.intake.accept_audio(path, &self.view.draft),
                Event::Published(url) => self.view.intake.practice = Some(url),
                Event::Finished(result) => {
                    if let Some(smoke) = &mut self.smoke {
                        smoke.intake_finished(result.clone());
                    }
                    self.view.intake.busy = false;
                    if let Err(problem) = result {
                        self.view.intake.status = problem;
                    }
                }
            }
        }
        view::show(ui, &mut self.view);
        if let Some(command) = self.view.intake.command.take() {
            match self.intake.commands.try_send(command) {
                Ok(()) => self.view.intake.busy = true,
                Err(_) => {
                    self.view.intake.status =
                        "Runner unavailable; pending Send remains retained".into();
                }
            }
        }
        if let Some(smoke) = &mut self.smoke {
            smoke.on_frame(ui.ctx(), &self.view.snapshot);
        }
    }
}
