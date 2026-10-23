use serde_json::Value;

// A zero exit is not an answer: the result envelope must report success and carry the
// schema-validated object, never prose or a partial reply.
pub fn answer(stdout: &[u8]) -> Result<Value, String> {
    let envelope: Value = serde_json::from_slice(stdout).map_err(|_| {
        "Claude returned empty or malformed output; inspect the private attempt".to_owned()
    })?;
    if envelope["type"] != "result" {
        return Err("Claude returned no final result; inspect the private attempt".into());
    }
    if envelope["is_error"] != false || envelope["subtype"] != "success" {
        let subtype = envelope["subtype"].as_str().unwrap_or("unknown");
        let reason = envelope["result"].as_str().unwrap_or_default();
        return Err(format!(
            "Claude stopped without a valid answer ({subtype}): {}",
            reason.chars().take(200).collect::<String>()
        ));
    }
    match envelope.get("structured_output") {
        Some(answer @ Value::Object(_)) => Ok(answer.clone()),
        _ => Err(
            "Claude finished without the required structured answer; nothing was published".into(),
        ),
    }
}

// Token counts the CLI reports for the attempt, kept with its private diagnostics.
pub fn usage(stdout: &[u8]) -> Value {
    serde_json::from_slice::<Value>(stdout).map_or(Value::Null, |envelope| {
        serde_json::json!({
            "usage": envelope["usage"],
            "models": envelope["modelUsage"],
            "turns": envelope["num_turns"],
            "duration_ms": envelope["duration_ms"],
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn envelope(overrides: &Value) -> Vec<u8> {
        let mut value = json!({
            "type": "result", "subtype": "success", "is_error": false,
            "result": "{\"phrases\":[]}", "structured_output": {"phrases": []},
            "usage": {"input_tokens": 6, "output_tokens": 687}, "num_turns": 5,
        });
        for (key, field) in overrides.as_object().unwrap() {
            value[key] = field.clone();
        }
        serde_json::to_vec(&value).unwrap()
    }

    #[test]
    fn only_a_successful_envelope_with_a_structured_object_is_an_answer() {
        assert_eq!(
            answer(&envelope(&json!({}))).unwrap(),
            json!({"phrases": []})
        );
        for (overrides, expected) in [
            (json!({"is_error": true}), "stopped without a valid answer"),
            (
                json!({"subtype": "error_max_structured_output_retries"}),
                "error_max_structured_output_retries",
            ),
            (json!({"structured_output": null}), "without the required"),
            (json!({"structured_output": "[]"}), "without the required"),
            (json!({"type": "assistant"}), "no final result"),
        ] {
            let problem = answer(&envelope(&overrides)).unwrap_err();
            assert!(problem.contains(expected), "{problem}");
        }
        for malformed in [&b""[..], b"{\"phrases\": []} trailing", b"Not logged in"] {
            assert!(answer(malformed).unwrap_err().contains("malformed"));
        }
    }

    #[test]
    fn reported_usage_is_retained_and_unknown_usage_stays_null() {
        let usage = usage(&envelope(&json!({})));
        assert_eq!(usage["usage"]["output_tokens"], 687);
        assert_eq!(usage["turns"], 5);
        assert!(super::usage(b"not json").is_null());
    }
}
