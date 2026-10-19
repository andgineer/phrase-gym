#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod app;
pub mod appearance;
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
