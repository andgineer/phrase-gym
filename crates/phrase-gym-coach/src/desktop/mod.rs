#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod app;
pub mod appearance;
mod dropped_text;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod failure;
pub mod intake;
mod intake_view;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod smoke;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod smoke_checks;
pub mod startup;
pub mod state;
pub mod text;
pub mod view;

mod connection;
mod intake_actions;
mod intake_runtime;
pub mod launch;
pub mod smoke_intake;

mod selected;
