//! The undo history is stored: every step's `inverse` and `forward` is a `MapCommand`
//! serialized into `map_events`, and read back days later. Renaming a variant or a field
//! still compiles and still round-trips in memory, but the rows already written stop
//! loading, and undo fails on the very change it was meant to protect.
//!
//! `fixtures/map_commands.json` holds one command per variant as the history stores it,
//! captured from real runs. Never edit a sample to match new code: an old row cannot be
//! edited either. Add a field with a serde default instead, and a sample for a new variant.

use serde_json::Value;
use wormholesystems::maps::MapCommand;

const FIXTURE: &str = include_str!("fixtures/map_commands.json");

/// Whether everything the stored row says is still said the same way. A field the code has
/// added since may appear in `now`; one the row holds may not go missing or change.
fn still_says(stored: &Value, now: &Value, path: &str) -> Result<(), String> {
    match (stored, now) {
        (Value::Object(stored), Value::Object(now)) => {
            for (key, value) in stored {
                let Some(current) = now.get(key) else {
                    return Err(format!("{path}.{key} is no longer written"));
                };
                still_says(value, current, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        (Value::Array(stored), Value::Array(now)) if stored.len() == now.len() => stored
            .iter()
            .zip(now)
            .enumerate()
            .try_for_each(|(i, (s, n))| still_says(s, n, &format!("{path}[{i}]"))),
        // Floats come back as floats whatever the sample spelled them as.
        (Value::Number(s), Value::Number(n)) if s.as_f64() == n.as_f64() => Ok(()),
        (s, n) if s == n => Ok(()),
        (s, n) => Err(format!("{path} was {s}, now {n}")),
    }
}

#[test]
fn every_stored_command_still_loads_and_says_the_same() {
    let samples: Vec<Value> = serde_json::from_str(FIXTURE).unwrap();
    for sample in &samples {
        let tag = sample["command"].as_str().unwrap();
        let cmd: MapCommand = serde_json::from_value(sample.clone())
            .unwrap_or_else(|e| panic!("a stored {tag} no longer loads: {e}"));
        let now = serde_json::to_value(&cmd).unwrap();
        if let Err(e) = still_says(sample, &now, tag) {
            panic!("a stored {tag} changed shape: {e}");
        }
    }
}

/// The tag serde writes for a variant: its name in snake case.
fn tag(variant: &str) -> String {
    let mut tag = String::new();
    for c in variant.chars() {
        if c.is_ascii_uppercase() && !tag.is_empty() {
            tag.push('_');
        }
        tag.push(c.to_ascii_lowercase());
    }
    tag
}

/// A variant without a sample is one whose stored form nothing protects.
#[test]
fn the_fixture_has_one_sample_per_variant() {
    let samples: Vec<Value> = serde_json::from_str(FIXTURE).unwrap();
    let mut stored: Vec<String> = samples
        .iter()
        .map(|s| s["command"].as_str().unwrap().to_string())
        .collect();
    stored.sort();
    let mut variants: Vec<String> = MapCommand::VARIANTS.iter().map(|v| tag(v)).collect();
    variants.sort();
    assert_eq!(stored, variants);
}
