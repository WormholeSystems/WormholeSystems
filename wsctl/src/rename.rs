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

fn volumes(runner: &mut dyn Runner, dir: &Path) -> Result<Vec<String>> {
    let out = runner.capture(dir, "docker", &["volume", "ls", "--format", "{{.Name}}"])?;
    Ok(out.lines().map(|l| l.trim().to_string()).collect())
}

/// The project this install still runs under when it is not ours yet, and only when there
/// is something under it: a fresh checkout resolves to its directory name too, with nothing
/// to move. Asked before `.env` names the new project, which is what the answer comes from.
pub fn old_project(runner: &mut dyn Runner, dir: &Path) -> Result<Option<String>> {
    let old = current_project(runner, dir)?;
    if old == PROJECT {
        return Ok(None);
    }
    let existing = volumes(runner, dir)?;
    let has_data = VOLUMES
        .iter()
        .any(|v| existing.contains(&format!("{old}_{v}")));
    Ok(has_data.then_some(old))
}

/// Whether there is a database volume already, under either name: an install that has
/// been running, as opposed to one about to be created.
pub fn has_database(runner: &mut dyn Runner, dir: &Path, old: Option<&str>) -> Result<bool> {
    let existing = volumes(runner, dir)?;
    Ok([Some(PROJECT), old]
        .into_iter()
        .flatten()
        .any(|project| existing.contains(&format!("{project}_pgdata"))))
}

/// Stop the old project and copy its volumes into ours. Run after the new images are
/// built, so the map is only down for the copy, not for a build.
pub fn move_project(runner: &mut dyn Runner, dir: &Path, old: &str) -> Result<Renamed> {
    let mut renamed = Renamed {
        from_project: Some(old.to_string()),
        ..Default::default()
    };
    // Down first: the old containers hold the ports and write to the volumes being copied.
    runner.run(dir, "docker", &["compose", "-p", old, "down"])?;

    let existing = volumes(runner, dir)?;
    for volume in VOLUMES {
        let from = format!("{old}_{volume}");
        let to = format!("{PROJECT}_{volume}");
        if !existing.contains(&from) {
            continue;
        }
        if existing.contains(&to) {
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
        let copied = runner.run(
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
        );
        // A half-filled copy would read as "already there" next time and be kept.
        if let Err(err) = copied {
            let _ = runner.capture(dir, "docker", &["volume", "rm", &to]);
            return Err(err.context(format!("could not copy {from} into {to}")));
        }
        renamed.copied.push(from);
    }
    Ok(renamed)
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

fn answers_one(runner: &mut dyn Runner, dir: &Path, user: &str, sql: &str) -> bool {
    exec_psql(runner, dir, user, sql).is_ok_and(|out| out == "1")
}

/// Rename `vector` in place, when the database still has it. Checking needs the database
/// up, which is harmless; the API is only stopped once there is something to rename, since
/// a database cannot be renamed with anyone connected to it. Either half may already be
/// done from a run that failed in between, so each is checked on its own. The password
/// survives (it is stored as SCRAM, which does not depend on the role's name), and inside
/// the container psql logs in over the local socket without one.
pub fn rename_role(runner: &mut dyn Runner, dir: &Path) -> Result<bool> {
    if !volumes(runner, dir)?.contains(&format!("{PROJECT}_pgdata")) {
        return Ok(false);
    }
    runner.run(
        dir,
        "docker",
        &["compose", "-p", PROJECT, "up", "-d", "--wait", "db"],
    )?;
    let superuser = if answers_one(runner, dir, OLD_ROLE, "select 1") {
        OLD_ROLE
    } else if answers_one(runner, dir, PROJECT, "select 1") {
        PROJECT
    } else {
        return Ok(false);
    };
    let role_left = superuser == OLD_ROLE;
    let database_left = answers_one(
        runner,
        dir,
        superuser,
        &format!("select 1 from pg_database where datname = '{OLD_ROLE}'"),
    );
    if !role_left && !database_left {
        return Ok(false);
    }

    runner.run(dir, "docker", &["compose", "-p", PROJECT, "stop", "api"])?;
    exec_psql(
        runner,
        dir,
        superuser,
        &format!("drop role if exists {HELPER_ROLE}"),
    )?;
    exec_psql(
        runner,
        dir,
        superuser,
        &format!("create role {HELPER_ROLE} superuser login"),
    )?;
    if role_left {
        exec_psql(
            runner,
            dir,
            HELPER_ROLE,
            &format!("alter role {OLD_ROLE} rename to {PROJECT}"),
        )?;
    }
    if database_left {
        exec_psql(
            runner,
            dir,
            HELPER_ROLE,
            &format!("alter database {OLD_ROLE} rename to {PROJECT}"),
        )?;
    }
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

    const VOLUME_LS: &str = "docker volume ls --format {{.Name}}";

    fn old_install() -> Recording {
        let mut runner = Recording::default();
        runner.replies.insert(
            "docker compose config".into(),
            "name: wormholesystems-next\nservices: {}".into(),
        );
        runner.replies.insert(
            VOLUME_LS.into(),
            "wormholesystems-next_pgdata\nwormholesystems-next_sde\nwormholesystems_sde\nother_pgdata"
                .into(),
        );
        runner
    }

    #[test]
    fn an_old_install_is_found_by_its_volumes() {
        let mut runner = old_install();
        assert_eq!(
            old_project(&mut runner, Path::new(".")).unwrap(),
            Some("wormholesystems-next".into())
        );
        assert!(has_database(&mut runner, Path::new("."), Some("wormholesystems-next")).unwrap());
    }

    #[test]
    fn a_fresh_checkout_has_nothing_to_move() {
        let mut runner = Recording::default();
        runner.replies.insert(
            "docker compose config".into(),
            "name: wormholesystems-next\n".into(),
        );
        assert_eq!(old_project(&mut runner, Path::new(".")).unwrap(), None);
        assert!(!has_database(&mut runner, Path::new("."), None).unwrap());
        assert!(!rename_role(&mut runner, Path::new(".")).unwrap());
        assert!(
            !runner
                .commands
                .iter()
                .any(|c| c.contains("stop") || c.contains("up"))
        );
    }

    #[test]
    fn an_old_install_is_copied_over() {
        let mut runner = old_install();
        let renamed = move_project(&mut runner, Path::new("."), "wormholesystems-next").unwrap();
        assert_eq!(
            renamed,
            Renamed {
                from_project: Some("wormholesystems-next".into()),
                copied: vec!["wormholesystems-next_pgdata".into()],
                skipped: vec!["wormholesystems-next_sde".into()],
                role: false,
            }
        );
        assert_eq!(
            runner.commands,
            vec![
                "docker compose -p wormholesystems-next down".to_string(),
                VOLUME_LS.into(),
                "docker volume create --label com.docker.compose.project=wormholesystems --label com.docker.compose.volume=pgdata wormholesystems_pgdata".into(),
                "docker run --rm -v wormholesystems-next_pgdata:/from:ro -v wormholesystems_pgdata:/to alpine cp -a /from/. /to/".into(),
            ]
        );
    }

    #[test]
    fn the_old_role_and_database_are_renamed() {
        let mut runner = Recording::default();
        runner
            .replies
            .insert(VOLUME_LS.into(), "wormholesystems_pgdata".into());
        runner
            .replies
            .insert(psql_line("vector", "select 1"), "1".into());
        runner.replies.insert(
            psql_line(
                "vector",
                "select 1 from pg_database where datname = 'vector'",
            ),
            "1".into(),
        );
        assert!(rename_role(&mut runner, Path::new(".")).unwrap());
        assert_eq!(
            runner.commands,
            vec![
                VOLUME_LS.to_string(),
                "docker compose -p wormholesystems up -d --wait db".into(),
                psql_line("vector", "select 1"),
                psql_line(
                    "vector",
                    "select 1 from pg_database where datname = 'vector'"
                ),
                "docker compose -p wormholesystems stop api".into(),
                psql_line("vector", "drop role if exists wsctl_rename"),
                psql_line("vector", "create role wsctl_rename superuser login"),
                psql_line(
                    "wsctl_rename",
                    "alter role vector rename to wormholesystems"
                ),
                psql_line(
                    "wsctl_rename",
                    "alter database vector rename to wormholesystems"
                ),
                psql_line("wormholesystems", "drop role wsctl_rename"),
            ]
        );
    }

    #[test]
    fn a_half_done_rename_finishes_the_database() {
        let mut runner = Recording::default();
        runner
            .replies
            .insert(VOLUME_LS.into(), "wormholesystems_pgdata".into());
        runner
            .replies
            .insert(psql_line("wormholesystems", "select 1"), "1".into());
        runner.replies.insert(
            psql_line(
                "wormholesystems",
                "select 1 from pg_database where datname = 'vector'",
            ),
            "1".into(),
        );
        assert!(rename_role(&mut runner, Path::new(".")).unwrap());
        assert!(!runner.commands.iter().any(|c| c.contains("alter role")));
        assert!(
            runner
                .commands
                .iter()
                .any(|c| c.contains("alter database vector"))
        );
    }

    #[test]
    fn a_renamed_database_keeps_the_api_running() {
        let mut runner = Recording::default();
        runner
            .replies
            .insert(VOLUME_LS.into(), "wormholesystems_pgdata".into());
        runner
            .replies
            .insert(psql_line("wormholesystems", "select 1"), "1".into());
        assert!(!rename_role(&mut runner, Path::new(".")).unwrap());
        assert!(!runner.commands.iter().any(|c| c.contains("stop api")));
    }
}
