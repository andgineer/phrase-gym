use std::collections::HashSet;

use phrase_gym_contracts::content::{
    ContentPackage, Cue, DialogueInput, Explanation, LessonInput, TargetInput,
};

use crate::error::CoreError;

const MAX_IDENTIFIER: usize = 160;
const MAX_TEXT: usize = 10_000;

type Checked = Result<(), String>;

pub(crate) fn identifier(field: &str, value: &str) -> Checked {
    let length = value.chars().count();
    let allowed = value
        .chars()
        .all(|character| character.is_alphanumeric() || "_.:-".contains(character));
    if (1..=MAX_IDENTIFIER).contains(&length) && allowed {
        Ok(())
    } else {
        Err(format!(
            "{field} must be 1-{MAX_IDENTIFIER} letters, digits or _.:- characters"
        ))
    }
}

pub(crate) fn text(field: &str, value: &str) -> Checked {
    if (1..=MAX_TEXT).contains(&value.chars().count()) {
        Ok(())
    } else {
        Err(format!("{field} must be 1-{MAX_TEXT} characters"))
    }
}

fn count(field: &str, length: usize, bounds: std::ops::RangeInclusive<usize>) -> Checked {
    if bounds.contains(&length) {
        Ok(())
    } else {
        Err(format!(
            "{field} must have {}-{} items",
            bounds.start(),
            bounds.end()
        ))
    }
}

fn distinct<'a>(values: impl IntoIterator<Item = &'a String>) -> Option<HashSet<&'a str>> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .all(|value| seen.insert(value.as_str()))
        .then_some(seen)
}

fn require(condition: bool, message: &str) -> Checked {
    if condition {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn explanation(value: &Explanation) -> Checked {
    text("explanation meaning", &value.meaning)?;
    if let Some(usage) = &value.usage {
        text("explanation usage", usage)?;
    }
    count("explanation breakdown", value.breakdown.len(), 0..=50)
}

fn cue(value: &Cue, chunks: &HashSet<&str>, required: &HashSet<&str>) -> Checked {
    identifier("cue id", &value.id)?;
    text("cue prompt", &value.prompt)?;
    for chunk in &value.visible_chunks {
        identifier("visible chunk", chunk)?;
    }
    let visible = distinct(&value.visible_chunks)
        .filter(|visible| visible.is_subset(chunks))
        .ok_or("Visible chunks must be distinct existing chunk IDs")?;
    require(
        !required.is_subset(&visible),
        "A retrieval cue must hide required material",
    )?;
    require(value.reviewed, "Manual packages accept reviewed cues only")
}

fn target(value: &TargetInput) -> Checked {
    identifier("source_id", &value.source_id)?;
    identifier("meaning_key", &value.meaning_key)?;
    text("recall_expression", &value.recall_expression)?;
    text("text", &value.text)?;
    text("purpose", &value.purpose)?;
    count("chunks", value.chunks.len(), 1..=100)?;
    count("required_chunks", value.required_chunks.len(), 1..=100)?;
    count("cues", value.cues.len(), 0..=50)?;
    count("concepts", value.concepts.len(), 0..=100)?;
    count(
        "acceptable_equivalents",
        value.acceptable_equivalents.len(),
        0..=100,
    )?;
    for chunk in &value.chunks {
        identifier("chunk id", &chunk.id)?;
        text("chunk text", &chunk.text)?;
    }
    for equivalent in &value.acceptable_equivalents {
        text("acceptable equivalent", equivalent)?;
    }
    for (slot, fillers) in &value.slots {
        identifier("slot", slot)?;
        for filler in fillers {
            text("slot filler", filler)?;
        }
    }
    for item in value.explanations.values() {
        explanation(item)?;
    }
    let joined: String = value
        .chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect();
    let chunks = distinct(value.chunks.iter().map(|chunk| &chunk.id))
        .filter(|_| joined == value.text)
        .ok_or("Chunk IDs must be unique and chunks must concatenate to the phrase")?;
    require(
        value
            .slots
            .keys()
            .all(|slot| chunks.contains(slot.as_str())),
        "Frame slots must reference existing chunks",
    )?;
    for chunk in &value.required_chunks {
        identifier("required chunk", chunk)?;
    }
    let required = distinct(&value.required_chunks)
        .filter(|required| required.is_subset(&chunks))
        .ok_or("Required chunks must be distinct existing chunk IDs")?;
    distinct(value.cues.iter().map(|cue| &cue.id)).ok_or("Cue IDs must be unique")?;
    value
        .cues
        .iter()
        .try_for_each(|item| cue(item, &chunks, &required))
}

fn lesson(value: &LessonInput, targets: &HashSet<&str>) -> Checked {
    identifier("lesson source_id", &value.source_id)?;
    text("lesson title", &value.title)?;
    count("lesson members", value.target_source_ids.len(), 1..=1000)?;
    for member in &value.target_source_ids {
        identifier("lesson member", member)?;
    }
    distinct(&value.target_source_ids)
        .filter(|members| members.is_subset(targets))
        .map(|_| ())
        .ok_or_else(|| "Lesson members must be distinct target source IDs in this package".into())
}

fn dialogue(value: &DialogueInput, package: &ContentPackage) -> Checked {
    identifier("dialogue source_id", &value.source_id)?;
    text("dialogue title", &value.title)?;
    count("dialogue turns", value.turns.len(), 2..=100)?;
    for turn in &value.turns {
        text("turn speaker", &turn.speaker)?;
        text("turn text", &turn.text)?;
        let Some(source) = &turn.target_source_id else {
            continue;
        };
        identifier("turn target_source_id", source)?;
        let linked = package
            .targets
            .iter()
            .find(|target| &target.source_id == source)
            .ok_or("Dialogue target must exist in this package")?;
        require(
            linked.text == turn.text,
            "Dialogue target text must match its complete phrase",
        )?;
    }
    Ok(())
}

fn package(value: &ContentPackage) -> Checked {
    identifier("namespace", &value.namespace)?;
    text("source_revision", &value.source_revision)?;
    count("targets", value.targets.len(), 1..=1000)?;
    count("lessons", value.lessons.len(), 0..=100)?;
    count("dialogues", value.dialogues.len(), 0..=100)?;
    value.targets.iter().try_for_each(target)?;
    let targets = distinct(value.targets.iter().map(|target| &target.source_id))
        .ok_or("Target source IDs must be unique")?;
    let unique = distinct(value.lessons.iter().map(|lesson| &lesson.source_id)).is_some()
        && distinct(value.dialogues.iter().map(|dialogue| &dialogue.source_id)).is_some();
    require(
        unique,
        "Lesson/dialogue source IDs must be unique within their kind",
    )?;
    for item in &value.lessons {
        lesson(item, &targets)?;
    }
    value
        .dialogues
        .iter()
        .try_for_each(|item| dialogue(item, value))
}

// Shape rules that need no stored state; violations are invalid requests, never publication conflicts.
pub fn check(value: &ContentPackage) -> Result<(), CoreError> {
    package(value).map_err(CoreError::Invalid)
}
