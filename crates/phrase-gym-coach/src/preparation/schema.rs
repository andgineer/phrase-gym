use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

// Generate the model's bounded output schema from the same committed contract checked by xtask.
// No model-authored field can change the outer generation, profile, guide or publication fence.
pub fn phrase_schema() -> Result<Value, String> {
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
    Ok(
        json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "$ref": "#/$defs/PreparedEnglishPhrase", "$defs": definitions}),
    )
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
        let schema = phrase_schema().unwrap();
        assert!(
            schema["$defs"]["PreparedEnglishPhrase"]["properties"]["reference_audio_id"]
                .is_object()
        );
        assert!(schema["$defs"]["PhraseSituation"].is_object());
        assert!(schema["$defs"]["PhraseHelp"].is_object());
        assert!(schema["$defs"]["RecordingUpload"].is_null());
        assert!(!schema.to_string().contains("#/components/"));
    }
}
