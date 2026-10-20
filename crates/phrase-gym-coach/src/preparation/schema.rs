use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

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
    let cues: Vec<_> = text
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect();
    if cues.len() < 2 {
        return Err(
            "This fixed-phrase demo requires a multiword phrase with a partial whole-word hint"
                .into(),
        );
    }
    let phrase = definitions
        .get_mut("PreparedEnglishPhrase")
        .ok_or("Missing phrase schema")?;
    phrase["properties"]["hint"] = json!({"type": "string", "enum": cues, "description": "Exactly one unchanged whole word from the source phrase, never a stem, paraphrase or full answer"});
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_schema_includes_phrase_dependencies_without_learner_or_access_contracts() {
        let schema = phrase_schema("Let's get started.").unwrap();
        assert!(
            schema["$defs"]["PreparedEnglishPhrase"]["properties"]["reference_audio_id"]
                .is_object()
        );
        assert!(schema["$defs"]["PhraseSituation"].is_object());
        assert!(schema["$defs"]["PhraseHelp"].is_object());
        assert!(schema["$defs"]["RecordingUpload"].is_null());
        assert_eq!(schema["type"], "object");
        assert_eq!(
            schema["properties"]["hint"]["enum"],
            json!(["Let's", "get", "started"])
        );
        assert!(!schema.to_string().contains("oneOf"));
        assert_eq!(
            schema["$defs"]["PhraseHelp"]["required"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
        assert_eq!(
            schema["$defs"]["PhraseFamiliarity"]["anyOf"][0]["additionalProperties"],
            false
        );
        assert!(!schema.to_string().contains("#/components/"));
    }
}
