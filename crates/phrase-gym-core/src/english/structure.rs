use std::collections::BTreeSet;

use phrase_gym_contracts::base::Language;
use phrase_gym_contracts::english::{
    PhraseFamiliarity, PreparedEnglishLesson, PreparedEnglishPhrase,
};

use crate::error::CoreError;

pub(super) fn text(value: &str, name: &str, limit: usize) -> Result<(), CoreError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.len() > limit
        || value.chars().any(char::is_control)
    {
        return Err(CoreError::Invalid(format!(
            "{name} must be nonempty, trimmed and at most {limit} bytes without control characters"
        )));
    }
    Ok(())
}

pub(super) fn words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn leaks(prompt: &str, phrase: &PreparedEnglishPhrase) -> bool {
    let prompt = words(prompt);
    std::iter::once(&phrase.text)
        .chain(&phrase.accepted_variants)
        .any(|answer| {
            let answer = words(answer);
            !answer.is_empty() && prompt.windows(answer.len()).any(|window| window == answer)
        })
}

fn phrase(phrase: &PreparedEnglishPhrase, source_ids: &BTreeSet<&str>) -> Result<(), CoreError> {
    for (name, value, limit) in [
        ("phrase source ID", &phrase.source_id, 200),
        ("meaning key", &phrase.meaning_key, 200),
        ("intention key", &phrase.intention_key, 200),
        ("fixed English wording", &phrase.text, 200),
        ("hint", &phrase.hint, 100),
    ] {
        text(value, name, limit)?;
    }
    if phrase.text.split_whitespace().count() > 30
        || phrase.text.contains(['{', '}', '_'])
        || phrase.reference_audio_id.is_nil()
        || phrase
            .expected_revision
            .is_some_and(|revision| revision < 1)
        || !(1..=2).contains(&phrase.situations.len())
        || phrase.accepted_variants.len() > 4
    {
        return Err(CoreError::Invalid("Preview phrases require short fixed wording, supplied audio, one or two situations and at most four accepted variants; slots and other target kinds are unsupported".into()));
    }
    let answer = words(&phrase.text);
    let hint = words(&phrase.hint);
    if hint.is_empty()
        || hint.len() >= answer.len()
        || !answer.windows(hint.len()).any(|window| window == hint)
    {
        return Err(CoreError::Invalid(
            "Hint must be one partial contiguous cue from the fixed phrase".into(),
        ));
    }
    let mut variants = BTreeSet::from([answer]);
    for variant in &phrase.accepted_variants {
        text(variant, "accepted variant", 200)?;
        if words(variant).is_empty()
            || variant.split_whitespace().count() > 30
            || !variants.insert(words(variant))
            || variant.contains(['{', '}', '_'])
        {
            return Err(CoreError::Invalid(
                "Accepted variants must be distinct fixed wording".into(),
            ));
        }
    }
    let mut situations = BTreeSet::new();
    let mut situation_texts = BTreeSet::new();
    for situation in &phrase.situations {
        text(&situation.id, "situation ID", 200)?;
        text(&situation.text, "situation", 800)?;
        if situation.language != Language::En
            || !situations.insert(&situation.id)
            || !situation_texts.insert(words(&situation.text))
            || situation.text.contains(['{', '}', '_'])
            || leaks(&situation.text, phrase)
            || situation.exposed_target_source_ids.len() > 4
            || situation
                .exposed_target_source_ids
                .iter()
                .any(|source| source == &phrase.source_id || !source_ids.contains(source.as_str()))
        {
            return Err(CoreError::Invalid("Situations must be distinct English prompts without answer wording, gaps or invalid declared exposures".into()));
        }
        if let Some(partner) = &situation.partner {
            text(&partner.text, "partner turn", 800)?;
            if leaks(&partner.text, phrase) || partner.text.contains(['{', '}', '_']) {
                return Err(CoreError::Invalid(
                    "Partner turn must not reveal answer wording or contain gaps".into(),
                ));
            }
        }
    }
    if let PhraseFamiliarity::Studied { provenance } = &phrase.familiarity {
        text(provenance, "studied familiarity provenance", 2_000)?;
    }
    if let Some(help) = &phrase.help {
        text(&help.meaning, "help meaning", 2_000)?;
        if let Some(usage) = &help.usage {
            text(usage, "help usage", 2_000)?;
        }
        if let Some(translation) = &help.translation {
            text(translation, "answer translation", 2_000)?;
        }
    }
    Ok(())
}

pub(super) fn validate(package: &PreparedEnglishLesson) -> Result<(), CoreError> {
    for (name, value, limit) in [
        ("namespace", &package.namespace, 200),
        ("source revision", &package.source_revision, 200),
        ("guide version", &package.guide_version, 200),
        ("assessment version", &package.assessment_version, 200),
        ("lesson source ID", &package.lesson.source_id, 200),
        ("lesson title", &package.lesson.title, 300),
    ] {
        text(value, name, limit)?;
    }
    if package.language != Language::En
        || package.profile_revision < 1
        || !(1..=5).contains(&package.phrases.len())
    {
        return Err(CoreError::Invalid("Preview supports one lesson of one to five fixed English phrases with a pinned profile".into()));
    }
    let ids: BTreeSet<_> = package
        .phrases
        .iter()
        .map(|phrase| phrase.source_id.as_str())
        .collect();
    let members: BTreeSet<_> = package
        .lesson
        .target_source_ids
        .iter()
        .map(String::as_str)
        .collect();
    if ids.len() != package.phrases.len()
        || members != ids
        || members.len() != package.lesson.target_source_ids.len()
    {
        return Err(CoreError::Invalid(
            "Lesson membership must name every unique phrase exactly once".into(),
        ));
    }
    for item in &package.phrases {
        if leaks(&package.lesson.title, item) {
            return Err(CoreError::Invalid(
                "Lesson title must not reveal answer wording".into(),
            ));
        }
        phrase(item, &ids)?;
    }
    Ok(())
}
