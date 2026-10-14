//! Running the commands a plan is made of.
//!
//! Behind a trait so the tests can assert exactly which commands a plan would run without
//! a docker daemon anywhere near them.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub trait Runner {
    /// Run to completion with the output going to the terminal, so a long build shows its
    /// progress rather than sitting silent.
    fn run(&mut self, dir: &Path, program: &str, args: &[&str]) -> Result<()>;

    /// Run quietly and hand back stdout, for the commands whose output we read. What it
    /// said on stderr is kept for the error when it fails, and dropped when it does not.
    fn capture(&mut self, dir: &Path, program: &str, args: &[&str]) -> Result<String>;
}

pub struct Real;

impl Runner for Real {
    fn run(&mut self, dir: &Path, program: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(program)
            .args(args)
            .current_dir(dir)
            .status()
            .with_context(|| format!("could not start `{program}`"))?;
        if !status.success() {
            bail!("`{program} {}` failed", args.join(" "));
        }
        Ok(())
    }

    fn capture(&mut self, dir: &Path, program: &str, args: &[&str]) -> Result<String> {
        let out = Command::new(program)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::null())
            .output()
            .with_context(|| format!("could not start `{program}`"))?;
        if !out.status.success() {
            bail!(
                "`{program} {}` failed{}",
                args.join(" "),
                said(&String::from_utf8_lossy(&out.stderr))
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
}

/// The end of what a failed command printed on stderr, indented under the error. The end,
/// because that is where docker and compose say what went wrong.
fn said(stderr: &str) -> String {
    const LINES: usize = 15;
    let lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    let tail = &lines[lines.len().saturating_sub(LINES)..];
    let mut out = String::from(":");
    for line in tail {
        out.push_str("\n    ");
        out.push_str(line.trim_end());
    }
    out
}

/// Records what it was asked to do and answers from a script, for tests. Every command
/// succeeds unless it is in `failures`.
#[cfg(test)]
#[derive(Default)]
pub struct Recording {
    pub commands: Vec<String>,
    pub replies: std::collections::HashMap<String, String>,
    pub failures: std::collections::HashSet<String>,
}

#[cfg(test)]
impl Recording {
    fn record(&mut self, program: &str, args: &[&str]) -> Result<String> {
        let line = format!("{program} {}", args.join(" "));
        self.commands.push(line.clone());
        if self.failures.contains(&line) {
            bail!("`{line}` failed");
        }
        Ok(line)
    }
}

#[cfg(test)]
impl Runner for Recording {
    fn run(&mut self, _dir: &Path, program: &str, args: &[&str]) -> Result<()> {
        self.record(program, args).map(drop)
    }

    fn capture(&mut self, _dir: &Path, program: &str, args: &[&str]) -> Result<String> {
        let line = self.record(program, args)?;
        Ok(self.replies.get(&line).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Without this a failed `docker compose run` said only that it failed.
    #[test]
    fn a_failure_says_what_the_command_printed() {
        let err = Real
            .capture(
                Path::new("."),
                "sh",
                &["-c", "echo out; echo first >&2; echo why >&2; exit 3"],
            )
            .unwrap_err()
            .to_string();
        assert!(err.contains("failed:\n    first\n    why"), "{err}");
        assert!(!err.contains("\n    out"));
    }

    #[test]
    fn only_the_end_of_a_long_complaint_is_kept() {
        let long: String = (1..=40).map(|n| format!("line {n}\n")).collect();
        let kept = said(&long);
        assert!(kept.ends_with("line 40"));
        assert!(!kept.contains("line 25\n"));
        assert_eq!(said("\n  \n"), "");
    }
}
