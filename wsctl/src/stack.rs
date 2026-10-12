//! The compose stack: what to run, and what it says back.
//!
//! Every command is a value rather than a call, so a test can assert what a step would do
//! without a docker daemon.

use std::path::Path;

use anyhow::{Context, Result};

use crate::exec::Runner;

/// `docker compose <args>`, which is how the whole stack is addressed.
pub fn compose(args: &[&str]) -> Vec<String> {
    let mut out = vec!["compose".to_string()];
    out.extend(args.iter().map(|a| a.to_string()));
    out
}

fn as_refs(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

pub fn build(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    runner.run(dir, "docker", &as_refs(&compose(&["build"])))
}

pub fn up(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    runner.run(dir, "docker", &as_refs(&compose(&["up", "-d"])))
}

/// Drop the build cache and images a rebuild has just superseded. Every api build leaves
/// a full set of layers behind, and a small disk fills within weeks of updates. Cache the
/// build just used is under a day old and stays, so the next update is still incremental.
pub fn prune(runner: &mut dyn Runner, dir: &Path) -> Result<()> {
    runner.run(
        dir,
        "docker",
        &["builder", "prune", "-f", "--filter", "until=24h"],
    )?;
    runner.run(dir, "docker", &["image", "prune", "-f"])
}

pub fn restart(runner: &mut dyn Runner, dir: &Path, service: &str) -> Result<()> {
    runner.run(dir, "docker", &as_refs(&compose(&["restart", service])))
}

pub fn ps(runner: &mut dyn Runner, dir: &Path) -> Result<String> {
    runner.capture(dir, "docker", &as_refs(&compose(&["ps"])))
}

/// One-shot command in a throwaway api container, with no dependencies started.
pub fn api(runner: &mut dyn Runner, dir: &Path, args: &[&str]) -> Result<String> {
    let mut full = vec!["run", "--rm", "--no-deps", "-T", "api", "wormholesystems"];
    full.extend_from_slice(args);
    runner.capture(dir, "docker", &as_refs(&compose(&full)))
}

/// Whether this stack's own proxy is up. It is the thing holding 80 and 443 on a machine
/// that is already running, and a port check that does not know this tells the operator to
/// stop the very stack they are updating.
pub fn proxy_running(runner: &mut dyn Runner, dir: &Path) -> bool {
    runner
        .capture(dir, "docker", &as_refs(&compose(&["ps", "-q", "caddy"])))
        .is_ok_and(|out| !out.trim().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdeStatus {
    pub loaded: String,
    pub latest: String,
    pub update_available: bool,
}

/// Parses the `key=value` lines `sde-status` prints.
pub fn parse_sde_status(output: &str) -> Option<SdeStatus> {
    let field = |name: &str| {
        output.lines().find_map(|line| {
            line.trim()
                .strip_prefix(&format!("{name}="))
                .map(|v| v.split_whitespace().next().unwrap_or(v).to_string())
        })
    };
    Some(SdeStatus {
        loaded: field("loaded")?,
        latest: field("latest").unwrap_or_default(),
        update_available: field("update_available").as_deref() == Some("yes"),
    })
}

pub fn sde_status(runner: &mut dyn Runner, dir: &Path) -> Result<SdeStatus> {
    let out = api(runner, dir, &["sde-status"])?;
    parse_sde_status(&out).context("could not read the SDE status; is the stack built?")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::Recording;

    #[test]
    fn the_stack_is_addressed_as_a_whole() {
        let mut runner = Recording::default();
        up(&mut runner, Path::new(".")).unwrap();
        build(&mut runner, Path::new(".")).unwrap();
        assert_eq!(
            runner.commands,
            vec![
                "docker compose up -d",
                "docker compose build",
            ]
        );
    }

    #[test]
    fn pruning_keeps_the_cache_the_last_build_used() {
        let mut runner = Recording::default();
        prune(&mut runner, Path::new(".")).unwrap();
        assert_eq!(
            runner.commands,
            vec![
                "docker builder prune -f --filter until=24h",
                "docker image prune -f",
            ]
        );
    }

    #[test]
    fn a_one_shot_command_takes_no_dependencies_with_it() {
        let mut runner = Recording::default();
        api(&mut runner, Path::new("."), &["sde-status"]).unwrap();
        assert_eq!(
            runner.commands,
            vec![
                "docker compose run --rm --no-deps -T api wormholesystems sde-status"
            ]
        );
    }

    #[test]
    fn reads_the_sde_status_lines() {
        let out = "loaded=3409592 released=2026-06-25\nlatest=3473160\nupdate_available=yes";
        assert_eq!(
            parse_sde_status(out),
            Some(SdeStatus {
                loaded: "3409592".into(),
                latest: "3473160".into(),
                update_available: true,
            })
        );
    }

    #[test]
    fn a_current_build_is_not_an_update() {
        let out = "loaded=3473160 released=2026-08-19\nlatest=3473160\nupdate_available=no";
        assert!(!parse_sde_status(out).unwrap().update_available);
    }

    #[test]
    fn nothing_useful_reads_as_nothing() {
        assert_eq!(parse_sde_status("bash: no such container"), None);
    }
}
