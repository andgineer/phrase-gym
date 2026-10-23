use super::input_problem::InputProblem;

pub const MAX_TEXT_BYTES: usize = 4000;
const MAX_ITEMS: usize = 5;
const MAX_ITEM_BYTES: usize = 800;
const TITLE_CHARS: usize = 60;
const SEPARATOR: &str = "·";

// One item per nonempty line; the original text itself is retained unchanged elsewhere.
pub fn entries(text: &str) -> Result<Vec<&str>, InputProblem> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(InputProblem::TextTooLong);
    }
    let entries: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if entries.is_empty() {
        return Err(InputProblem::Empty);
    }
    if entries.len() > MAX_ITEMS {
        return Err(InputProblem::TooManyItems);
    }
    if entries
        .iter()
        .any(|entry| entry.contains("http://") || entry.contains("https://"))
    {
        return Err(InputProblem::Link);
    }
    if entries.iter().any(|entry| entry.len() > MAX_ITEM_BYTES) {
        return Err(InputProblem::ItemTooLong);
    }
    if entries
        .iter()
        .any(|entry| entry.chars().any(char::is_control))
    {
        return Err(InputProblem::HiddenCharacters);
    }
    Ok(entries)
}

// Lessons are named after their items so the learner can tell them apart; a long list is cut on
// a word boundary and marked as continuing.
pub fn lesson_title(entries: &[&str]) -> String {
    let full = entries.join(&format!(" {SEPARATOR} "));
    if full.chars().count() <= TITLE_CHARS {
        return full;
    }
    let words: Vec<&str> = full.split(' ').collect();
    let mut kept = 0;
    let mut length = 0;
    for word in &words {
        let next = length + usize::from(kept > 0) + word.chars().count();
        if next + 4 > TITLE_CHARS {
            break;
        }
        length = next;
        kept += 1;
    }
    if kept == 0 {
        let start: String = full.chars().take(TITLE_CHARS - 1).collect();
        return format!("{start}…");
    }
    let mut title = words[..kept].to_vec();
    let at_boundary = title.last() == Some(&SEPARATOR) || words.get(kept) == Some(&SEPARATOR);
    if title.last() == Some(&SEPARATOR) {
        title.pop();
    }
    let title = title.join(" ");
    if at_boundary {
        format!("{title} {SEPARATOR} …")
    } else {
        format!("{title} …")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_follow_the_shared_examples() {
        let examples: serde_json::Value =
            serde_json::from_str(include_str!("../../../../fixtures/lesson-titles.json")).unwrap();
        for example in examples.as_array().unwrap() {
            let items: Vec<&str> = example["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_str().unwrap())
                .collect();
            let title = lesson_title(&items);
            assert_eq!(title, example["title"].as_str().unwrap(), "{items:?}");
            assert!(title.chars().count() <= TITLE_CHARS);
        }
    }

    #[test]
    fn each_learner_input_problem_is_named_specifically() {
        let cases = [
            ("", InputProblem::Empty),
            (" \n\t\n", InputProblem::Empty),
            ("a\nb\nc\nd\ne\nf", InputProblem::TooManyItems),
            ("see https://example.com", InputProblem::Link),
            (&"x".repeat(801), InputProblem::ItemTooLong),
            ("one\ttwo", InputProblem::HiddenCharacters),
            (&"word\n".repeat(900), InputProblem::TextTooLong),
        ];
        for (text, problem) in cases {
            assert_eq!(entries(text), Err(problem), "{text:?}");
        }
        assert_eq!(
            entries("  Let's get started.\r\n\nimpediment\n").unwrap(),
            ["Let's get started.", "impediment"]
        );
    }
}
