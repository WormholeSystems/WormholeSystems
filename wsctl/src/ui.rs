//! Everything the operator sees.
//!
//! The prompts read the real terminal rather than stdin, because the installer pipes the
//! script into a shell and stdin is that pipe.

use anyhow::{Context, Result};
use console::style;
use inquire::{Confirm, Password, Select, Text, validator::Validation};

use crate::checks::{Check, Verdict};

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
pub fn ask(prompt: &str, default: Option<String>) -> Result<String> {
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

/// A secret, echoed as nothing. Not confirmed twice: it is pasted, not remembered.
pub fn ask_secret(prompt: &str, keep: Option<String>) -> Result<String> {
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
    Confirm::new(prompt)
        .with_default(default)
        .prompt()
        .context("cancelled")
}

/// One of a few options, by index.
pub fn choose(prompt: &str, options: &[&str], default: usize) -> Result<usize> {
    Ok(Select::new(prompt, options.to_vec())
        .with_starting_cursor(default)
        .raw_prompt()
        .context("cancelled")?
        .index)
}
