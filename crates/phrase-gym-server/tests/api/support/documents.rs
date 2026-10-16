use std::collections::BTreeMap;

use serde_json::Value;

use super::repository_root;

pub fn scenario_bytes(spec: &Value) -> Vec<u8> {
    let source = spec.as_object().expect("byte source is an object");
    let data = if let Some(path) = source.get("file") {
        std::fs::read(repository_root().join(path.as_str().unwrap())).unwrap()
    } else if let Some(fill) = source.get("fill") {
        let byte = u8::try_from(fill[0].as_u64().unwrap()).unwrap();
        vec![byte; usize::try_from(fill[1].as_u64().unwrap()).unwrap()]
    } else if let Some(text) = source.get("text") {
        text.as_str().unwrap().as_bytes().to_vec()
    } else {
        panic!("unknown byte source {spec}");
    };
    match source.get("slice") {
        Some(range) => {
            let bound = |index: usize| usize::try_from(range[index].as_u64().unwrap()).unwrap();
            data[bound(0)..bound(1)].to_vec()
        }
        None => data,
    }
}

pub fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in data.chunks(3) {
        let bytes = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let group = u32::from(bytes[0]) << 16 | u32::from(bytes[1]) << 8 | u32::from(bytes[2]);
        for index in 0..4 {
            if index <= chunk.len() {
                output.push(char::from(
                    ALPHABET[(group >> (18 - 6 * index) & 63) as usize],
                ));
            } else {
                output.push('=');
            }
        }
    }
    output
}

fn assign(document: &mut Value, pointer: &str, value: Value) {
    let (parent, last) = pointer.rsplit_once('/').expect("pointer has a parent");
    let last = last.replace("~1", "/").replace("~0", "~");
    match document.pointer_mut(parent).expect("pointer parent exists") {
        Value::Array(items) => items[last.parse::<usize>().unwrap()] = value,
        Value::Object(fields) => {
            fields.insert(last, value);
        }
        other => panic!("cannot assign into {other}"),
    }
}

pub fn expand(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(expand).collect()),
        Value::Object(fields) if fields.contains_key("$base64") => {
            Value::String(base64(&scenario_bytes(&fields["$base64"])))
        }
        Value::Object(fields) if fields.contains_key("$json_file") => {
            let path = repository_root().join(fields["$json_file"].as_str().unwrap());
            let mut document: Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            if let Some(Value::Object(replacements)) = fields.get("set") {
                for (pointer, replacement) in replacements {
                    assign(&mut document, pointer, expand(replacement));
                }
            }
            document
        }
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), expand(item)))
                .collect(),
        ),
        other => other.clone(),
    }
}

pub fn substitute(value: &Value, captures: &BTreeMap<String, Value>) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| substitute(item, captures))
                .collect(),
        ),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), substitute(item, captures)))
                .collect(),
        ),
        Value::String(text) => {
            if let Some(name) = text
                .strip_prefix("{{")
                .and_then(|rest| rest.strip_suffix("}}"))
                && !name.contains("{{")
            {
                return captures[name].clone();
            }
            let mut result = text.clone();
            for (name, captured) in captures {
                let replacement = match captured {
                    Value::String(inner) => inner.clone(),
                    other => other.to_string(),
                };
                result = result.replace(&format!("{{{{{name}}}}}"), &replacement);
            }
            assert!(!result.contains("{{"), "unresolved capture in {result}");
            Value::String(result)
        }
        other => other.clone(),
    }
}
