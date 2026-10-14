//! Everything the operator sees.
//!
//! The prompts read the real terminal rather than stdin, because the installer pipes the
//! script into a shell and stdin is that pipe. A scripted run (`setup --non-interactive`)
//! asks nothing: every prompt takes its default, which the wizard has already worked out
//! from `.env` and the environment, and a required answer without one is written down for
//! the wizard to refuse on once it has asked everything.

use std::cell::{Cell, RefCell};

use anyhow::{Context, Result, anyhow};
use console::style;
use inquire::{Confirm, Password, Select, Text, validator::Validation};

use crate::checks::{Check, Verdict};

thread_local! {
    static SCRIPTED: Cell<bool> = const { Cell::new(false) };
    static MISSING: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Answer every prompt from here on with its default.
pub fn script() {
    SCRIPTED.set(true);
}

pub fn scripted() -> bool {
    SCRIPTED.get()
}

/// The keys a scripted run needed and did not have, emptied as they are read.
pub fn take_missing() -> Vec<String> {
    MISSING.with_borrow_mut(std::mem::take)
}

/// Note a required answer a scripted run has nothing for, and stand in an empty one.
fn missing(key: &str) -> String {
    MISSING.with_borrow_mut(|m| m.push(key.to_string()));
    String::new()
}

/// What a scripted run took, so its log reads like the questions it skipped.
fn answered(prompt: &str, answer: &str) {
    note(&format!("{prompt}: {answer}"));
}

pub fn heading(text: &str) {
    println!("\n{}", style(text).bold());
}

pub fn note(text: &str) {
    println!("  {}", style(text).dim());
}

pub fn done(text: &str) {
    println!("  {} {text}", style("✓").green());
}

pub fn report(check: &Check) {
    let mark = match check.verdict {
        Verdict::Ok => style("✓").green(),
        Verdict::Warn => style("!").yellow(),
        Verdict::Fatal => style("✗").red(),
    };
    println!("  {mark} {}", check.label);
    if let Some(hint) = &check.hint {
        println!("    {}", style(hint).dim());
    }
}

/// A required answer, kept until there is one. Everything asked for here is something the
/// server refuses to start without, so a blank would only move the failure to the boot.
/// `key` is the variable a scripted run reads it from, named when it is missing.
pub fn ask(key: &str, prompt: &str, default: Option<String>) -> Result<String> {
    if scripted() {
        return Ok(match default {
            Some(value) => {
                answered(prompt, &value);
                value
            }
            None => missing(key),
        });
    }
    let mut text = Text::new(prompt).with_validator(|input: &str| {
        Ok(if input.trim().is_empty() {
            Validation::Invalid("this one is required".into())
        } else {
            Validation::Valid
        })
    });
    if let Some(default) = &default {
        text = text.with_default(default);
    }
    Ok(text.prompt().context("cancelled")?.trim().to_string())
}

/// An answer that may be left blank.
pub fn ask_optional(prompt: &str, default: Option<String>) -> Result<String> {
    if scripted() {
        let value = default.unwrap_or_default();
        answered(prompt, &value);
        return Ok(value);
    }
    let mut text = Text::new(prompt);
    if let Some(default) = &default {
        text = text.with_default(default);
    }
    Ok(text.prompt().context("cancelled")?.trim().to_string())
}

/// An answer that may be left blank, pre-filled with the current one so it can be edited or
/// cleared. `tidy` checks the input and returns what is stored.
pub fn edit(
    prompt: &str,
    current: &str,
    tidy: fn(&str) -> Result<String, String>,
) -> Result<String> {
    if scripted() {
        let value = tidy(current).map_err(|message| anyhow!("{prompt}: {message}"))?;
        answered(prompt, &value);
        return Ok(value);
    }
    let answer = Text::new(prompt)
        .with_initial_value(current)
        .with_validator(move |input: &str| {
            Ok(match tidy(input) {
                Ok(_) => Validation::Valid,
                Err(message) => Validation::Invalid(message.into()),
            })
        })
        .prompt()
        .context("cancelled")?;
    tidy(&answer).map_err(anyhow::Error::msg)
}

/// A secret, echoed as nothing. Not confirmed twice: it is pasted, not remembered. A
/// scripted run needs one, under `key`, unless `key` is empty.
pub fn ask_secret(key: &str, prompt: &str, keep: Option<String>) -> Result<String> {
    if scripted() {
        let keep = keep.filter(|k| !k.is_empty());
        answered(prompt, if keep.is_some() { "(set)" } else { "(none)" });
        return Ok(match keep {
            Some(value) => value,
            None if key.is_empty() => String::new(),
            None => missing(key),
        });
    }
    if let Some(existing) = keep
        && !existing.is_empty()
        && !confirm(&format!("{prompt} — replace the one already set?"), false)?
    {
        return Ok(existing);
    }
    Ok(Password::new(prompt)
        .without_confirmation()
        .prompt()
        .context("cancelled")?
        .trim()
        .to_string())
}

pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    if scripted() {
        answered(prompt, if default { "yes" } else { "no" });
        return Ok(default);
    }
    Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .context("cancelled")
}

/// One of a few options, by index.
pub fn choose(prompt: &str, options: &[&str], default: usize) -> Result<usize> {
    if scripted() {
        answered(prompt, options.get(default).copied().unwrap_or_default());
        return Ok(default);
    }
    Ok(Select::new(prompt, options.to_vec())
        .with_starting_cursor(default)
        .raw_prompt()
        .context("cancelled")?
        .index)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A scripted run takes every default, and a required answer it has none for is named
    // by the variable to set rather than failing on the first.
    #[test]
    fn a_scripted_run_takes_the_defaults_and_names_what_is_missing() {
        script();
        assert_eq!(ask("A", "a", Some("x".into())).unwrap(), "x");
        assert_eq!(ask("B", "b", None).unwrap(), "");
        assert_eq!(ask_secret("C", "c", None).unwrap(), "");
        assert_eq!(ask_secret("", "optional", None).unwrap(), "");
        assert_eq!(ask_secret("D", "d", Some("s".into())).unwrap(), "s");
        assert!(confirm("e", true).unwrap());
        assert_eq!(choose("f", &["one", "two"], 1).unwrap(), 1);
        assert!(edit("g", "nope", |_| Err("bad".into())).is_err());
        assert_eq!(take_missing(), vec!["B", "C"]);
        assert!(take_missing().is_empty());
    }
}
