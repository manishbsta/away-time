use awaytime_core::{pause_gate, settings_errors};
use serde_json::{json, Value};
use std::io::{self, Read};
fn run() -> Result<Value, String> {
    let mut bytes = vec![];
    io::stdin()
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1_048_576 {
        return Err("Request exceeds one MiB".into());
    }
    let request: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if request.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("Unsupported request version".into());
    }
    let input = request
        .get("input")
        .filter(|v| v.is_object())
        .ok_or("Input must be an object")?;
    match request.get("operation").and_then(Value::as_str) {
        Some("validate_settings") => {
            let errors = settings_errors(input);
            Ok(json!({"valid": errors.is_empty(), "errors": errors}))
        }
        Some("pause_gate") => {
            let reasons: Vec<String> =
                serde_json::from_value(input.get("reasons").cloned().ok_or("Missing reasons")?)
                    .map_err(|e| e.to_string())?;
            let excluded: Vec<String> =
                serde_json::from_value(input.get("excluded").cloned().ok_or("Missing exclusions")?)
                    .map_err(|e| e.to_string())?;
            serde_json::to_value(pause_gate(&reasons, &excluded)).map_err(|e| e.to_string())
        }
        _ => Err("Operation not implemented".into()),
    }
}
fn main() {
    match run() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("{}", json!({"error": error}));
            std::process::exit(2);
        }
    }
}
