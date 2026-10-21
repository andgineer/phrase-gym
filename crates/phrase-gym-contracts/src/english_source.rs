use crate::content::TargetKind;
use crate::english::EnglishTask;

pub fn word_form(source: &str) -> Option<&str> {
    let first = source
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_alphanumeric());
    (!first.is_empty()
        && (source.split_whitespace().count() == 1
            || source
                .strip_prefix(first)
                .is_some_and(|rest| rest.trim_start().starts_with(['—', ':']))))
    .then_some(first)
}

pub fn kind(source: &str) -> TargetKind {
    if source.contains('{') && source.contains('}') {
        TargetKind::Frame
    } else if word_form(source).is_some() {
        TargetKind::WordSense
    } else {
        TargetKind::FixedPhrase
    }
}

pub fn matches_task(source: &str, task: Option<&EnglishTask>) -> bool {
    match (kind(source), task) {
        (TargetKind::FixedPhrase, None | Some(EnglishTask::Frame { .. })) => true,
        (TargetKind::Frame, Some(EnglishTask::Frame { pattern, .. })) => pattern == source,
        (TargetKind::WordSense, Some(EnglishTask::WordSense { word, .. })) => {
            word_form(source).is_some_and(|form| form.eq_ignore_ascii_case(word))
        }
        _ => false,
    }
}

pub fn frame_example_matches(
    pattern: &str,
    slots: &[crate::english::FrameSlot],
    answer: &str,
) -> bool {
    if !(1..=3).contains(&slots.len())
        || slots
            .iter()
            .any(|slot| !(2..=5).contains(&slot.values.len()))
    {
        return false;
    }
    let mut examples = vec![pattern.to_owned()];
    for slot in slots {
        let marker = format!("{{{}}}", slot.name);
        if pattern.matches(&marker).count() != 1 {
            return false;
        }
        examples = examples
            .iter()
            .flat_map(|example| {
                slot.values
                    .iter()
                    .map(|value| example.replace(&marker, value))
            })
            .collect();
    }
    examples.iter().all(|example| !example.contains(['{', '}']))
        && examples.iter().any(|example| example == answer)
}

pub const WORD_SENSE_CUE_SEPARATOR: &str = " | ";

pub fn word_gap_matches(prompt: &str, answer: &str, word: &str) -> bool {
    let sentence = match prompt.split_once(WORD_SENSE_CUE_SEPARATOR) {
        Some((sentence, cue)) => {
            if cue.is_empty() || cue.trim() != cue || cue.contains(WORD_SENSE_CUE_SEPARATOR) {
                return false;
            }
            sentence
        }
        None => prompt,
    };
    sentence.matches("_____").count() == 1
        && prompt.matches("_____").count() == 1
        && !prompt
            .split(|c: char| !c.is_alphanumeric())
            .any(|part| part.eq_ignore_ascii_case(word))
        && sentence.replace("_____", word).eq_ignore_ascii_case(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_substitution_does_not_duplicate_articles_and_bounds_expansion() {
        let slots = vec![crate::english::FrameSlot {
            name: "event".into(),
            values: vec!["the meeting".into(), "the interview".into()],
        }];
        assert!(frame_example_matches(
            "Move {event}.",
            &slots,
            "Move the meeting."
        ));
        assert!(!frame_example_matches(
            "Move the {event}.",
            &slots,
            "Move the meeting."
        ));
        assert!(!frame_example_matches(
            "Move {event}.",
            &[],
            "Move the meeting."
        ));
    }

    #[test]
    fn word_prompt_fills_only_the_selected_word_and_never_leaks_it_in_a_cue() {
        assert!(word_gap_matches(
            "That makes _____. | A reasonable meaning.",
            "That makes sense.",
            "sense"
        ));
        assert!(word_gap_matches(
            "_____ this point.",
            "Clarify this point.",
            "clarify"
        ));
        assert!(word_gap_matches(
            "Please _____ this point. | Make it clear.",
            "Please clarify this point.",
            "clarify"
        ));
        assert!(!word_gap_matches(
            "Please _____ this point. | Use clarify.",
            "Please clarify this point.",
            "clarify"
        ));
        assert!(!word_gap_matches(
            "Please make this clear.",
            "Please clarify this point.",
            "clarify"
        ));
        assert!(!word_gap_matches(
            "The instruction is confusing: _____.",
            "Please clarify this point.",
            "clarify"
        ));
    }

    #[test]
    fn word_reference_must_include_the_entire_gap_sentence_before_an_explicit_cue() {
        assert!(!word_gap_matches(
            "Please act. | _____",
            "Please act.",
            "clarify"
        ));
        for prompt in [
            "Please _____ the deadline.",
            "Please _____ the deadline. | Make it clear.",
        ] {
            assert!(word_gap_matches(
                prompt,
                "Please clarify the deadline.",
                "clarify"
            ));
            for truncated in [
                "Please clar",
                "Please clarify",
                "Please clarify the",
                "Please clarify the deadline",
            ] {
                assert!(
                    !word_gap_matches(prompt, truncated, "clarify"),
                    "{prompt}: {truncated}"
                );
            }
        }
        for prompt in [
            "Please _____ the deadline. Make it clear.",
            "Please _____ the deadline. | ",
            "Please _____ the deadline. | Make it clear. | Explain.",
        ] {
            assert!(!word_gap_matches(
                prompt,
                "Please clarify the deadline.",
                "clarify"
            ));
        }
    }

    #[test]
    fn bare_words_require_word_retrieval_and_plain_phrases_allow_inferred_frames() {
        assert_eq!(kind("Could we move {item} to {day}?"), TargetKind::Frame);
        for source in ["clarify — make clear", "clarify: make clear", "clarify"] {
            assert_eq!(word_form(source), Some("clarify"));
            assert_eq!(kind(source), TargetKind::WordSense);
            assert!(!matches_task(source, None));
        }
        assert!(matches_task("Let's get started.", None));
        let frame = EnglishTask::Frame {
            pattern: "Could we move {item} to {day}?".into(),
            slots: vec![],
        };
        assert!(matches_task(
            "Could we move the meeting to Friday?",
            Some(&frame)
        ));
        assert!(!matches_task("clarify", Some(&frame)));
        assert!(!matches_task("Could we move {item} to {day}?", None));
    }
}
