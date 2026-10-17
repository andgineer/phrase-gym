use rfd::{MessageButtons, MessageDialog, MessageLevel};

use super::view::TITLE;

pub fn startup_message(error: &eframe::Error) -> String {
    match error {
        eframe::Error::Wgpu(cause) => format!(
            "Phrase Gym Coach could not start its graphics renderer ({cause}).\n\n\
             Coach draws its window with Metal on macOS and Direct3D 12 on Windows. Update the \
             graphics driver and restart Coach. In a Windows Remote Desktop or virtual machine \
             session, enable graphics acceleration for that session or start Coach locally."
        ),
        other => format!(
            "Phrase Gym Coach could not open its window ({other}).\n\n\
             Restart Coach. If the problem continues, sign out and back in to the desktop session."
        ),
    }
}

pub fn already_running() -> &'static str {
    "Phrase Gym Coach is already running for this user. Switch to its open window instead of \
     starting another copy."
}

// A Finder or Start-menu launch has no terminal, so failures before the window exists need a
// native dialog to be seen at all.
pub fn show(message: &str) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(TITLE)
        .set_description(message)
        .set_buttons(MessageButtons::Ok)
        .show();
}

#[cfg(test)]
mod tests {
    use eframe::egui_wgpu::WgpuError;

    use super::*;

    #[test]
    fn renderer_failures_name_the_cause_and_the_next_step() {
        let error = eframe::Error::Wgpu(WgpuError::NoSurfaceFormatsAvailable);
        let message = startup_message(&error);
        assert!(
            message.contains("no valid format for the surface"),
            "{message}"
        );
        assert!(message.contains("Update the graphics driver"));
        assert!(message.contains("Remote Desktop"));
        let other = eframe::Error::AppCreation("bad state".into());
        assert!(startup_message(&other).contains("bad state"));
    }
}
