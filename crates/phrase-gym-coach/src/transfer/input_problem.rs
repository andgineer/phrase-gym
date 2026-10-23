// Problems the learner can cause and fix; each message says what happened and what to do, and the
// main window shows it verbatim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputProblem {
    Busy,
    SeveralFiles,
    Folder,
    Symlink,
    RichText,
    Binary,
    NotUtf8,
    FileTooLong,
    TextTooLong,
    TooManyItems,
    ItemTooLong,
    Empty,
    Link,
    HiddenCharacters,
    Unreadable,
    PreviousPending,
}

impl InputProblem {
    pub const ALL: [Self; 16] = [
        Self::Busy,
        Self::SeveralFiles,
        Self::Folder,
        Self::Symlink,
        Self::RichText,
        Self::Binary,
        Self::NotUtf8,
        Self::FileTooLong,
        Self::TextTooLong,
        Self::TooManyItems,
        Self::ItemTooLong,
        Self::Empty,
        Self::Link,
        Self::HiddenCharacters,
        Self::Unreadable,
        Self::PreviousPending,
    ];

    pub fn message(self) -> &'static str {
        match self {
            Self::Busy => {
                "Coach is still taking in your previous material. Wait until it says it is preparing, then paste or drop this again."
            }
            Self::SeveralFiles => "Drop one text file at a time.",
            Self::Folder => "That’s a folder. Drop one plain text file instead.",
            Self::Symlink => "That’s a link or alias to a file. Drop the text file itself.",
            Self::RichText => {
                "This is a rich-text file. Save it as plain text, then drop it again."
            }
            Self::Binary => {
                "This file isn’t plain text. Save it as a plain text file, then drop it again."
            }
            Self::NotUtf8 => {
                "This file isn’t saved as UTF-8 text. Save it as UTF-8 plain text, then drop it again."
            }
            Self::FileTooLong => {
                "This file is longer than 4,000 bytes. Keep it to up to five short words or phrases."
            }
            Self::TextTooLong => {
                "This text is longer than 4,000 bytes. Add up to five short words or phrases at a time."
            }
            Self::TooManyItems => {
                "Coach adds up to five items at a time, one per line. Split longer lists into batches of five or fewer."
            }
            Self::ItemTooLong => {
                "One line is longer than 800 bytes. Keep each line to a short word or phrase."
            }
            Self::Empty => "There’s no text to add. Paste, type or drop a word or phrase.",
            Self::Link => "Paste the words themselves, not a link. Coach doesn’t open web pages.",
            Self::HiddenCharacters => {
                "A line contains a tab or another hidden character. Remove it, then try again."
            }
            Self::Unreadable => {
                "Coach couldn’t read that file. Make sure it isn’t locked or still downloading, then drop it again."
            }
            Self::PreviousPending => {
                "Your previous material hasn’t reached your library yet; Coach keeps retrying it. Add new material once it is accepted."
            }
        }
    }

    pub fn recognize(status: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|problem| problem.message() == status)
    }
}

impl From<InputProblem> for String {
    fn from(problem: InputProblem) -> Self {
        problem.message().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_problem_has_a_distinct_actionable_message_that_round_trips() {
        let mut seen = std::collections::BTreeSet::new();
        for problem in InputProblem::ALL {
            let message = problem.message();
            assert!(seen.insert(message), "{problem:?}");
            assert!(message.ends_with('.'), "{problem:?}");
            assert_eq!(InputProblem::recognize(message), Some(problem));
            assert!(!message.contains("Server") && !message.contains("CLI"));
        }
        assert_eq!(InputProblem::recognize("Coach couldn’t finish"), None);
    }
}
