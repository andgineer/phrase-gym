use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::intake::Event;

#[derive(Clone, Default)]
pub struct Events(Arc<Mutex<VecDeque<Event>>>);

impl Events {
    pub fn push(&self, event: Event) {
        let mut pending = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // A sleeping UI needs the latest state of each kind, including final outcomes; a full
        // progress channel must neither lose them nor block worker lease cleanup and shutdown.
        pending
            .retain(|previous| std::mem::discriminant(previous) != std::mem::discriminant(&event));
        pending.push_back(event);
    }

    pub fn pop(&self) -> Option<Event> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleeping_ui_keeps_failure_and_completion_after_prolonged_progress() {
        let events = Events::default();
        events.push(Event::Status("Preparing".into()));
        events.push(Event::Finished(Ok(())));
        for count in 0..1000 {
            events.push(Event::Summary(format!("{count} phrases")));
        }
        events.push(Event::Status("Selected CLI frame example failed".into()));
        events.push(Event::Finished(Err("Preparation failed".into())));
        for count in 1000..2000 {
            events.push(Event::Summary(format!("{count} phrases")));
        }
        assert_eq!(events.0.lock().unwrap().len(), 3);
        assert!(matches!(events.pop(), Some(Event::Status(message)) if message.contains("failed")));
        assert!(
            matches!(events.pop(), Some(Event::Finished(Err(message))) if message == "Preparation failed")
        );
        assert!(matches!(events.pop(), Some(Event::Summary(message)) if message == "1999 phrases"));
        assert!(events.pop().is_none());
    }

    #[test]
    fn latest_publication_and_input_survive_without_blocking_an_unread_ui() {
        let events = Events::default();
        for count in 0..1000 {
            events.push(Event::Draft(format!("input {count}")));
            events.push(Event::Published(format!("lesson {count}")));
            events.push(Event::Status(format!("Added {count} targets")));
        }
        assert_eq!(events.0.lock().unwrap().len(), 3);
        assert!(matches!(events.pop(), Some(Event::Draft(text)) if text == "input 999"));
        assert!(matches!(events.pop(), Some(Event::Published(url)) if url == "lesson 999"));
        assert!(matches!(events.pop(), Some(Event::Status(text)) if text == "Added 999 targets"));
    }
}
