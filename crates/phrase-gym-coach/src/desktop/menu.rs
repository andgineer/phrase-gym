use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

pub(super) struct ApplicationMenu {
    _menu: Menu,
    settings: Arc<AtomicBool>,
}

impl ApplicationMenu {
    pub(super) fn install(context: egui::Context) -> Result<Self, String> {
        let menu = Menu::new();
        let settings = MenuItem::new(
            "Settings…",
            true,
            Some(Accelerator::new(Modifiers::META, Code::Comma)),
        );
        let application = Submenu::with_items(
            "Phrase Gym Coach",
            true,
            &[
                &settings,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ],
        )
        .map_err(|error| error.to_string())?;
        let cut = MenuItem::new(
            "Cut",
            true,
            Some(Accelerator::new(Modifiers::META, Code::KeyX)),
        );
        let copy = MenuItem::new(
            "Copy",
            true,
            Some(Accelerator::new(Modifiers::META, Code::KeyC)),
        );
        let paste = MenuItem::new(
            "Paste",
            true,
            Some(Accelerator::new(Modifiers::META, Code::KeyV)),
        );
        let clipboard = [
            (cut.id().clone(), egui::ViewportCommand::RequestCut),
            (copy.id().clone(), egui::ViewportCommand::RequestCopy),
            (paste.id().clone(), egui::ViewportCommand::RequestPaste),
        ];
        let edit = Submenu::with_items("Edit", true, &[&cut, &copy, &paste])
            .map_err(|error| error.to_string())?;
        menu.append_items(&[&application, &edit])
            .map_err(|error| error.to_string())?;
        menu.init_for_nsapp();
        let requested = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&requested);
        let id = settings.id().clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if event.id == id {
                flag.store(true, Ordering::SeqCst);
                context.request_repaint();
            } else if let Some((_, command)) = clipboard.iter().find(|(id, _)| *id == event.id) {
                forward_clipboard(&context, command.clone());
            }
        }));
        Ok(Self {
            _menu: menu,
            settings: requested,
        })
    }

    pub(super) fn take_settings(&self) -> bool {
        self.settings.swap(false, Ordering::SeqCst)
    }
}

fn forward_clipboard(context: &egui::Context, command: egui::ViewportCommand) {
    let focused = context
        .input(|input| {
            input
                .raw
                .viewports
                .iter()
                .find_map(|(id, viewport)| (viewport.focused == Some(true)).then_some(*id))
        })
        .unwrap_or(egui::ViewportId::ROOT);
    // Native AppKit selectors do not reach egui's TextEdit clipboard event path.
    context.send_viewport_cmd_to(focused, command);
    context.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_clipboard_commands_target_the_focused_viewport() {
        let context = egui::Context::default();
        context.set_embed_viewports(false);
        let settings = egui::ViewportId::from_hash_of("settings");
        let mut input = egui::RawInput::default();
        input.viewports.entry(settings).or_default().focused = Some(true);
        context.begin_pass(input);
        context.show_viewport_deferred(settings, egui::ViewportBuilder::default(), |_, _| {});
        forward_clipboard(&context, egui::ViewportCommand::RequestPaste);
        let mut output = context.end_pass();
        output.textures_delta.clear();
        assert!(
            output.viewport_output[&settings]
                .commands
                .iter()
                .any(|command| { matches!(command, egui::ViewportCommand::RequestPaste) })
        );
    }
}
