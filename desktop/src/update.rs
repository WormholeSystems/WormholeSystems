//! Self-update. The newest `desktop-v*` release comes from GitHub's API: this repo also
//! releases wsctl, so "the latest release" is not necessarily ours. Its `latest.json` lists
//! a signed bundle per platform, checked against the public key compiled into release
//! builds; local builds have no key and only point at the release page.

use cargo_packager_updater::semver::Version;
use cargo_packager_updater::{Config, Update, UpdaterBuilder};
use serde::Deserialize;

pub const REPO: &str = "wormholesystems/wormholesystems";
const TAG_PREFIX: &str = "desktop-v";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Set by CI from the repository's `DESKTOP_UPDATER_PUBKEY` variable.
const PUBLIC_KEY: Option<&str> = option_env!("WS_UPDATER_PUBKEY");

/// A newer release than the one running.
#[derive(Clone)]
pub struct Available {
    pub version: String,
    pub page: String,
    /// Set when this copy can replace itself: a signed bundle exists, and the app runs
    /// from something the updater knows how to swap (the installed exe, the `.app`, the
    /// AppImage).
    pub installer: Option<Update>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// Ask GitHub. Blocking, so off the UI thread.
pub fn check() -> Result<Option<Available>, String> {
    let current: Version = VERSION.parse().map_err(|e| format!("{e}"))?;
    let releases: Vec<Release> = ureq::get(format!(
        "https://api.github.com/repos/{REPO}/releases?per_page=50"
    ))
    .header(
        "User-Agent",
        concat!("wormholesystems-desktop/", env!("CARGO_PKG_VERSION")),
    )
    .header("Accept", "application/vnd.github+json")
    .call()
    .map_err(|e| format!("could not ask GitHub for releases: {e}"))?
    .body_mut()
    .read_json()
    .map_err(|e| format!("could not read GitHub's releases: {e}"))?;

    let Some((release, version)) = newest(&releases, &current) else {
        return Ok(None);
    };
    let installer = installer_for(&release.tag_name, &version);
    Ok(Some(Available {
        version: version.to_string(),
        page: release.html_url.clone(),
        installer,
    }))
}

/// The newest desktop release above `current`. A pre-release build hears about newer
/// pre-releases too; a stable one only about stable releases.
fn newest<'a>(releases: &'a [Release], current: &Version) -> Option<(&'a Release, Version)> {
    let previews = !current.pre.is_empty();
    releases
        .iter()
        .filter(|r| !r.draft && (previews || !r.prerelease))
        .filter_map(|r| {
            let version: Version = r.tag_name.strip_prefix(TAG_PREFIX)?.parse().ok()?;
            (version > *current).then_some((r, version))
        })
        .max_by(|a, b| a.1.cmp(&b.1))
}

/// The signed bundle for this platform in `tag`'s `latest.json`, when this copy can use it.
fn installer_for(tag: &str, version: &Version) -> Option<Update> {
    let pubkey = PUBLIC_KEY.filter(|key| !key.is_empty())?;
    if !replaceable() {
        return None;
    }
    let config = Config {
        endpoints: vec![
            format!("https://github.com/{REPO}/releases/download/{tag}/latest.json")
                .parse()
                .ok()?,
        ],
        pubkey: pubkey.to_string(),
        ..Default::default()
    };
    let update = UpdaterBuilder::new(VERSION.parse().ok()?, config)
        .build()
        .ok()?
        .check()
        .ok()??;
    (update.version.trim_start_matches('v') == version.to_string()).then_some(update)
}

/// Whether the running copy is one the updater can swap in place. A binary unpacked from
/// the Linux tarball or the portable zip is not; those get the release page instead.
fn replaceable() -> bool {
    if cfg!(target_os = "linux") {
        return std::env::var_os("APPIMAGE").is_some();
    }
    if cfg!(target_os = "macos") {
        return std::env::current_exe()
            .is_ok_and(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS/"));
    }
    // The installer puts the app under the user's local app data.
    std::env::current_exe()
        .is_ok_and(|exe| dirs::data_local_dir().is_some_and(|data| exe.starts_with(data)))
}

/// Download, check the signature and install. On Windows the installer takes over and the
/// process ends here; elsewhere the caller restarts.
pub fn install(update: &Update) -> Result<(), String> {
    update.download_and_install().map_err(|e| format!("{e}"))
}

/// Start the new version. An AppImage runs from a mount that goes away with this process,
/// so start the file itself again, a moment later so the shortcuts are free by then.
pub fn restart(cx: &mut gpui_kit::App) {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        let started = std::process::Command::new("sh")
            .arg("-c")
            .arg("sleep 1; exec \"$0\"")
            .arg(appimage)
            .spawn();
        if started.is_ok() {
            cx.quit();
            return;
        }
    }
    cx.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool) -> Release {
        Release {
            tag_name: tag.into(),
            html_url: format!("https://example.test/{tag}"),
            draft: false,
            prerelease,
        }
    }

    #[test]
    fn picks_the_newest_desktop_release_and_skips_wsctl() {
        let releases = [
            release("wsctl-v9.0.0", false),
            release("desktop-v0.1.0-alpha.2", true),
            release("desktop-v0.1.0-alpha.3", true),
            release("desktop-v0.1.0-alpha.1", true),
        ];
        let current: Version = "0.1.0-alpha.1".parse().unwrap();
        let (r, v) = newest(&releases, &current).unwrap();
        assert_eq!(r.tag_name, "desktop-v0.1.0-alpha.3");
        assert_eq!(v.to_string(), "0.1.0-alpha.3");
    }

    #[test]
    fn a_stable_build_ignores_previews() {
        let releases = [
            release("desktop-v0.2.0-alpha.1", true),
            release("desktop-v0.1.1", false),
        ];
        let current: Version = "0.1.0".parse().unwrap();
        assert_eq!(newest(&releases, &current).unwrap().1.to_string(), "0.1.1");
        let current: Version = "0.1.1".parse().unwrap();
        assert!(newest(&releases, &current).is_none());
    }

    #[test]
    fn an_alpha_hears_about_the_stable_release() {
        let releases = [release("desktop-v0.1.0", false)];
        let current: Version = "0.1.0-alpha.4".parse().unwrap();
        assert_eq!(newest(&releases, &current).unwrap().1.to_string(), "0.1.0");
    }
}
