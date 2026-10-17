use phrase_gym_contracts::content::{Cue, TargetInput, TargetKind};

fn visible<'a>(content: &'a TargetInput, cue: &'a Cue) -> impl Iterator<Item = &'a str> {
    content
        .chunks
        .iter()
        .filter(|chunk| cue.visible_chunks.contains(&chunk.id))
        .map(|chunk| chunk.text.as_str())
}

pub fn has_visible_hint(content: &TargetInput, cue: &Cue) -> bool {
    visible(content, cue).any(|text| text.chars().any(char::is_alphanumeric))
}

pub fn spoken_hint(content: &TargetInput, cue: &Cue) -> String {
    visible(content, cue).collect()
}

// Words and frames must hide every required chunk; a phrase cue may show part of the answer.
pub fn practicable(content: &TargetInput, cue: &Cue) -> bool {
    cue.reviewed
        && has_visible_hint(content, cue)
        && (content.kind == TargetKind::FixedPhrase
            || !content
                .required_chunks
                .iter()
                .any(|chunk| cue.visible_chunks.contains(chunk)))
}
