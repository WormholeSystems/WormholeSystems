//! Moving an install off the names it started with. The compose project used to take its
//! name from the checkout's directory, so containers, the network and the volumes were
//! called whatever the clone was called, and the database role and database were
//! `vector`. Both are `wormholesystems` now.
//!
//! Volumes are copied, never moved: docker cannot rename one, and a copy leaves the old
//! install intact to go back to until the operator removes it.

use std::path::Path;

use anyhow::{Context, Result};

use crate::exec::Runner;

pub const PROJECT: &str = "wormholesystems";
const OLD_ROLE: &str = "vector";
const VOLUMES: [&str; 5] = ["pgdata", "sde", "uploads", "caddy_data", "caddy_config"];
/// Postgres will not rename the role a session is logged in as, and `vector` is the only
/// superuser an old volume has, so a throwaway one does the renaming.
const HELPER_ROLE: &str = "wsctl_rename";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Renamed {
    /// The project the install ran under before, when it was not already ours.
    pub from_project: Option<String>,
    /// The old volumes copied into ours, by their old names.
    pub copied: Vec<String>,
    /// Old volumes left alone because ours already existed.
    pub skipped: Vec<String>,
    pub role: bool,
}

impl Renamed {
    pub fn anything(&self) -> bool {
        self.from_project.is_some() || self.role
    }
}

/// The project compose resolves for this checkout right now: `.env`, then the directory.
pub fn current_project(runner: &mut dyn Runner, dir: &Path) -> Result<String> {
    let out = runner
        .capture(dir, "docker", &["compose", "config"])
        .context("could not read the compose configuration")?;
    parse_project(&out).context("the compose configuration names no project")
}

fn parse_project(config: &str) -> Option<String> {
    config.lines().find_map(|line| {
        line.strip_prefix("name: ")
            .map(|name| name.trim().trim_matches('"').to_string())
    })
}

/// Do whatever this install still needs. `full` is the full setup, the one with a
/// database of its own; a standalone install's database belongs to someone else.
pub fn run(runner: &mut dyn Runner, dir: &Path, full: bool) -> Result<Renamed> {
    let mut renamed = Renamed::default();
    let old = current_project(runner, dir)?;
    if old != PROJECT {
        move_project(runner, dir, &old, &mut renamed)?;
        // A fresh checkout resolves to its directory name too, with nothing under it.
        if !renamed.copied.is_empty() || !renamed.skipped.is_empty() {
            renamed.from_project = Some(old);
        }
    }
    if full {
        renamed.role = rename_role(runner, dir)?;
    }
    Ok(renamed)
}

fn move_project(
    runner: &mut dyn Runner,
    dir: &Path,
    old: &str,
    renamed: &mut Renamed,
) -> Result<()> {
    // Down first: the old containers hold the ports and write to the volumes being copied.
    runner.run(dir, "docker", &["compose", "-p", old, "down"])?;

    let existing = runner.capture(dir, "docker", &["volume", "ls", "--format", "{{.Name}}"])?;
    let existing: Vec<&str> = existing.lines().map(str::trim).collect();
    for volume in VOLUMES {
        let from = format!("{old}_{volume}");
        let to = format!("{PROJECT}_{volume}");
        if !existing.contains(&from.as_str()) {
            continue;
        }
        if existing.contains(&to.as_str()) {
            renamed.skipped.push(from);
            continue;
        }
        // Labelled as compose would, so it adopts the volume instead of warning about it.
        runner.capture(
            dir,
            "docker",
            &[
                "volume",
                "create",
                "--label",
                &format!("com.docker.compose.project={PROJECT}"),
                "--label",
                &format!("com.docker.compose.volume={volume}"),
                &to,
            ],
        )?;
        runner.run(
            dir,
            "docker",
            &[
                "run",
                "--rm",
                "-v",
                &format!("{from}:/from:ro"),
                "-v",
                &format!("{to}:/to"),
                "alpine",
                "cp",
                "-a",
                "/from/.",
                "/to/",
            ],
        )?;
        renamed.copied.push(from);
    }
    Ok(())
}

fn psql(user: &str, sql: &str) -> Vec<String> {
    [
        "compose",
        "-p",
        PROJECT,
        "exec",
        "-T",
        "db",
        "psql",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        user,
        "-d",
        "postgres",
        "-Atc",
        sql,
    ]
    .map(String::from)
    .to_vec()
}

fn exec_psql(runner: &mut dyn Runner, dir: &Path, user: &str, sql: &str) -> Result<String> {
    let args = psql(user, sql);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    runner.capture(dir, "docker", &args)
}

/// Rename `vector` in place. Only the database runs meanwhile: a database cannot be renamed
/// with anyone connected to it. The password survives (it is stored as SCRAM, which does not
/// depend on the role's name), and inside the container psql logs in over the local socket
/// without one.
fn rename_role(runner: &mut dyn Runner, dir: &Path) -> Result<bool> {
    runner.run(dir, "docker", &["compose", "-p", PROJECT, "stop", "api"])?;
    runner.run(
        dir,
        "docker",
        &["compose", "-p", PROJECT, "up", "-d", "--wait", "db"],
    )?;
    // A fresh volume or one already renamed has no such role to log in as.
    let old_exists = exec_psql(runner, dir, OLD_ROLE, "select 1").is_ok_and(|out| out == "1");
    if !old_exists {
        return Ok(false);
    }

    exec_psql(
        runner,
        dir,
        OLD_ROLE,
        &format!("create role {HELPER_ROLE} superuser login"),
    )?;
    exec_psql(
        runner,
        dir,
        HELPER_ROLE,
        &format!("alter role {OLD_ROLE} rename to {PROJECT}"),
    )?;
    exec_psql(
        runner,
        dir,
        HELPER_ROLE,
        &format!("alter database {OLD_ROLE} rename to {PROJECT}"),
    )?;
    exec_psql(runner, dir, PROJECT, &format!("drop role {HELPER_ROLE}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::Recording;

    fn psql_line(user: &str, sql: &str) -> String {
        format!("docker {}", psql(user, sql).join(" "))
    }

    #[test]
    fn reads_the_project_off_the_compose_config() {
        assert_eq!(
            parse_project("name: wormholesystems-next\nservices:\n  api:\n"),
            Some("wormholesystems-next".into())
        );
        assert_eq!(parse_project("services: {}"), None);
    }

    #[test]
    fn an_old_install_is_copied_over_and_its_role_renamed() {
        let mut runner = Recording::default();
        runner.replies.insert(
            "docker compose config".into(),
            "name: wormholesystems-next\nservices: {}".into(),
        );
        runner.replies.insert(
            "docker volume ls --format {{.Name}}".into(),
            "wormholesystems-next_pgdata\nwormholesystems-next_sde\nwormholesystems_sde\nother_pgdata"
                .into(),
        );
        runner
            .replies
            .insert(psql_line("vector", "select 1"), "1".into());

        let renamed = run(&mut runner, Path::new("."), true).unwrap();
        assert_eq!(
            renamed,
            Renamed {
                from_project: Some("wormholesystems-next".into()),
                copied: vec!["wormholesystems-next_pgdata".into()],
                skipped: vec!["wormholesystems-next_sde".into()],
                role: true,
            }
        );
        assert_eq!(
            runner.commands,
            vec![
                "docker compose config".to_string(),
                "docker compose -p wormholesystems-next down".into(),
                "docker volume ls --format {{.Name}}".into(),
                "docker volume create --label com.docker.compose.project=wormholesystems --label com.docker.compose.volume=pgdata wormholesystems_pgdata".into(),
                "docker run --rm -v wormholesystems-next_pgdata:/from:ro -v wormholesystems_pgdata:/to alpine cp -a /from/. /to/".into(),
                "docker compose -p wormholesystems stop api".into(),
                "docker compose -p wormholesystems up -d --wait db".into(),
                psql_line("vector", "select 1"),
                psql_line("vector", "create role wsctl_rename superuser login"),
                psql_line("wsctl_rename", "alter role vector rename to wormholesystems"),
                psql_line("wsctl_rename", "alter database vector rename to wormholesystems"),
                psql_line("wormholesystems", "drop role wsctl_rename"),
            ]
        );
    }

    #[test]
    fn a_current_install_is_left_alone() {
        let mut runner = Recording::default();
        runner.replies.insert(
            "docker compose config".into(),
            "name: wormholesystems\n".into(),
        );
        let renamed = run(&mut runner, Path::new("."), true).unwrap();
        assert!(!renamed.anything());
        assert!(!runner.commands.iter().any(|c| c.contains("rename")));
    }

    #[test]
    fn a_standalone_install_has_no_database_to_rename() {
        let mut runner = Recording::default();
        runner
            .replies
            .insert("docker compose config".into(), "name: ws-old\n".into());
        let renamed = run(&mut runner, Path::new("."), false).unwrap();
        assert!(!renamed.role);
        assert!(!runner.commands.iter().any(|c| c.contains("psql")));
    }
}
