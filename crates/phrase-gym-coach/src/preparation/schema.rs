use std::collections::{BTreeMap, BTreeSet};

use phrase_gym_contracts::english_source;
use serde_json::{Value, json};

const WORD_SITUATION: &str = r"^[^_.!?|]*_____[^_.!?|]*[.!?]( \| [^_|]+)?$";

// Generate the model's bounded output schema from the same committed contract checked by xtask.
// No model-authored field can change the outer generation, profile, guide or publication fence.
pub fn phrase_schema(text: &str) -> Result<Value, String> {
    let api: Value = serde_json::from_str(include_str!(
        "../../../../frontend/tests/rust-contract/openapi.json"
    ))
    .map_err(|_| "Cannot read committed output contract")?;
    let schemas = api["components"]["schemas"]
        .as_object()
        .ok_or("Missing contract schemas")?;
    let mut required = BTreeSet::from(["PreparedEnglishPhrase".to_owned()]);
    let mut definitions = BTreeMap::new();
    while let Some(name) = required.pop_first() {
        if definitions.contains_key(&name) {
            continue;
        }
        let mut schema = schemas
            .get(&name)
            .ok_or("Output contract reference is missing")?
            .clone();
        references(&mut schema, &mut required);
        definitions.insert(name, schema);
    }
    definitions
        .get_mut("FrameSlot")
        .ok_or("Missing frame slot schema")?["properties"]["name"] = json!({
        "type":"string", "minLength":1, "maxLength":english_source::FRAME_SLOT_NAME_MAX_BYTES,
        "pattern":"^[A-Za-z0-9-]+$",
        "description":"Use a short ASCII identifier such as short-wait-period; no spaces or underscores."
    });
    let cues: Vec<_> = text
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();
    if cues.is_empty() {
        return Err("Preparation requires nonempty English material".into());
    }
    let mut task = match english_source::kind(text) {
        phrase_gym_contracts::content::TargetKind::FixedPhrase => {
            json!({"anyOf":[{"type":"null"}, definitions["EnglishTask"]["oneOf"][0].clone()]})
        }
        kind => {
            let name = if kind == phrase_gym_contracts::content::TargetKind::Frame {
                "frame"
            } else {
                "word_sense"
            };
            definitions["EnglishTask"]["oneOf"]
                .as_array()
                .ok_or("Missing task schema")?
                .iter()
                .find(|schema| schema["properties"]["kind"]["enum"][0] == name)
                .ok_or("Missing requested task")?
                .clone()
        }
    };
    if let Some(word) = english_source::word_form(text) {
        task["properties"]["contexts"]["minItems"] = json!(2);
        task["properties"]["contexts"]["maxItems"] = json!(3);
        task["properties"]["word"] = json!({"type":"string", "enum":[word]});
    }
    let mut situation = definitions["PhraseSituation"].clone();
    situation["properties"]["partner"] = json!({"type":"null"});
    if english_source::word_form(text).is_some() {
        // One gapped sentence, then an optional cue: scenario narration cannot precede the gap.
        situation["properties"]["text"]["pattern"] = json!(WORD_SITUATION);
        situation["properties"]["text"]["description"] = json!(
            "Only the short gapped answer sentence, optionally followed by ' | ' and a brief cue; no scenario narration. The reference answer fills the entire sentence before that explicit separator, never a prefix."
        );
    }
    let phrase = definitions
        .get_mut("PreparedEnglishPhrase")
        .ok_or("Missing phrase schema")?;
    phrase["properties"]["task"] = task;
    phrase["properties"]["situations"]["items"] = situation;
    if english_source::word_form(text).is_some() {
        phrase["properties"]["situations"]["minItems"] = json!(2);
        phrase["properties"]["situations"]["maxItems"] = json!(3);
    }
    let first = cues[0];
    let word = english_source::word_form(text).is_some();
    let hints: Vec<String> = if word {
        let letters = first.chars().count();
        (1..=(letters / 2).max(1))
            .filter(|length| *length < letters)
            .map(|length| first.chars().take(length).collect())
            .collect()
    } else {
        cues.iter().map(|cue| (*cue).to_owned()).collect()
    };
    phrase["properties"]["hint"] = if hints.is_empty() {
        json!({"type":"string", "minLength":1, "maxLength":100})
    } else {
        json!({"type": "string", "enum": hints, "description": "Fixed/frame cues are unchanged whole source words; word cues are partial initial forms."})
    };
    let mut root = phrase.clone();
    root["$defs"] = json!(definitions);
    strict(&mut root);
    Ok(root)
}

fn strict(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            if let Some(alternatives) = fields.remove("oneOf") {
                fields.insert("anyOf".into(), alternatives);
            }
            if fields.get("format").and_then(Value::as_str) == Some("int64") {
                fields.remove("format");
            }
            if let Some(Value::Object(properties)) = fields.get("properties") {
                fields.insert(
                    "required".into(),
                    json!(properties.keys().collect::<Vec<_>>()),
                );
                fields.insert("additionalProperties".into(), json!(false));
            }
            for child in fields.values_mut() {
                strict(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                strict(child);
            }
        }
        _ => {}
    }
}

fn references(value: &mut Value, required: &mut BTreeSet<String>) {
    match value {
        Value::Object(fields) => {
            if let Some(Value::String(reference)) = fields.get_mut("$ref")
                && let Some(name) = reference
                    .strip_prefix("#/components/schemas/")
                    .map(str::to_owned)
            {
                *reference = format!("#/$defs/{name}");
                required.insert(name);
            }
            for value in fields.values_mut() {
                references(value, required);
            }
        }
        Value::Array(values) => {
            for value in values {
                references(value, required);
            }
        }
        _ => {}
    }
}

pub fn batch_schema(
    inputs: &[phrase_gym_contracts::inbox::LessonPhraseInput],
) -> Result<Value, String> {
    let mut phrase = phrase_schema(&inputs.first().ok_or("Empty material")?.text)?;
    if inputs.len() == 1 {
        bind_exposures(&mut phrase, &inputs[0].source_id, inputs);
        return Ok(phrase);
    }
    let definitions = phrase
        .as_object_mut()
        .ok_or("Missing phrase schema")?
        .remove("$defs")
        .ok_or("Missing definitions")?;
    let mut alternatives = Vec::new();
    for input in inputs {
        let mut schema = phrase_schema(&input.text)?;
        schema
            .as_object_mut()
            .ok_or("Missing phrase schema")?
            .remove("$defs");
        schema["properties"]["source_id"] = json!({"type":"string", "enum":[input.source_id]});
        bind_exposures(&mut schema, &input.source_id, inputs);
        alternatives.push(schema);
    }
    Ok(
        json!({"type": "object", "properties": {"phrases": {"type": "array", "items": {"anyOf": alternatives}, "minItems": inputs.len(), "maxItems": inputs.len()}}, "required": ["phrases"], "additionalProperties": false, "$defs": definitions}),
    )
}

fn bind_exposures(
    schema: &mut Value,
    source_id: &str,
    inputs: &[phrase_gym_contracts::inbox::LessonPhraseInput],
) {
    let others: Vec<_> = inputs
        .iter()
        .filter(|input| input.source_id != source_id)
        .map(|input| &input.source_id)
        .collect();
    let items = if others.is_empty() {
        json!({"type":"string"})
    } else {
        json!({"type":"string", "enum":others})
    };
    schema["properties"]["situations"]["items"]["properties"]["exposed_target_source_ids"] =
        json!({"type":"array", "items":items, "maxItems":others.len()});
}

#[cfg(test)]
#[path = "schema_tests.rs"]
mod tests;
