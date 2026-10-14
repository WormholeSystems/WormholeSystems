//! The site-wide announcement: a banner an operator puts above every page, for planned
//! downtime, a move to a new address, or anything else people should read first. Set from
//! the `ANNOUNCEMENT_*` environment variables, off whenever the message is empty.

use serde::Serialize;
use sha2::{Digest, Sha256};

/// How loudly the banner is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum AnnouncementLevel {
    #[default]
    Info,
    Warning,
    Critical,
}

impl AnnouncementLevel {
    /// An unknown level is not worth refusing to start over: it reads as `info`.
    fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "warning" => AnnouncementLevel::Warning,
            "critical" => AnnouncementLevel::Critical,
            _ => AnnouncementLevel::Info,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            AnnouncementLevel::Info => "info",
            AnnouncementLevel::Warning => "warning",
            AnnouncementLevel::Critical => "critical",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AnnouncementLink {
    pub url: String,
    /// The url itself when no label was configured.
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Announcement {
    /// Identifies this exact announcement, derived from everything it says. A dismissal is
    /// remembered against it, so changing any of the content brings the banner back for
    /// everyone.
    pub id: String,
    pub level: AnnouncementLevel,
    pub title: Option<String>,
    pub message: String,
    pub link: Option<AnnouncementLink>,
    /// Whether people may close it. One that cannot be closed ignores an old dismissal.
    pub dismissible: bool,
}

/// The raw values, as the environment holds them. Blank counts as absent throughout.
#[derive(Debug, Default)]
pub struct AnnouncementVars<'a> {
    pub level: Option<&'a str>,
    pub title: Option<&'a str>,
    pub message: Option<&'a str>,
    pub link_url: Option<&'a str>,
    pub link_label: Option<&'a str>,
    pub dismissible: Option<&'a str>,
}

impl Announcement {
    /// The announcement these values describe, or `None` when there is no message.
    pub fn from_vars(vars: AnnouncementVars<'_>) -> Option<Announcement> {
        let filled = |v: Option<&str>| {
            v.map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        };
        let message = filled(vars.message)?;
        let level = vars.level.map(AnnouncementLevel::parse).unwrap_or_default();
        let title = filled(vars.title);
        let link = filled(vars.link_url).map(|url| AnnouncementLink {
            label: filled(vars.link_label).unwrap_or_else(|| url.clone()),
            url,
        });
        // Anything but an explicit no leaves it closable, which is the default.
        let dismissible = !vars.dismissible.is_some_and(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "false" | "0" | "no" | "off"
            )
        });

        let id = id_of(level, title.as_deref(), &message, link.as_ref());
        Some(Announcement {
            id,
            level,
            title,
            message,
            link,
            dismissible,
        })
    }
}

fn id_of(
    level: AnnouncementLevel,
    title: Option<&str>,
    message: &str,
    link: Option<&AnnouncementLink>,
) -> String {
    let parts = [
        level.as_str(),
        title.unwrap_or(""),
        message,
        link.map_or("", |l| l.url.as_str()),
        link.map_or("", |l| l.label.as_str()),
    ];
    let digest = Sha256::digest(parts.join("\u{1f}"));
    hex::encode(&digest[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_message(message: &str) -> AnnouncementVars<'_> {
        AnnouncementVars {
            message: Some(message),
            ..Default::default()
        }
    }

    #[test]
    fn no_message_means_no_announcement() {
        assert_eq!(Announcement::from_vars(AnnouncementVars::default()), None);
        assert_eq!(Announcement::from_vars(with_message("  \n ")), None);
    }

    #[test]
    fn carries_everything_configured() {
        let a = Announcement::from_vars(AnnouncementVars {
            level: Some("warning"),
            title: Some("Moving to a new address"),
            message: Some("This page moves to legacy.wormhole.systems on October 1st."),
            link_url: Some("https://legacy.wormhole.systems"),
            link_label: Some("Read more"),
            dismissible: None,
        })
        .unwrap();
        assert_eq!(a.level, AnnouncementLevel::Warning);
        assert_eq!(a.title.as_deref(), Some("Moving to a new address"));
        assert_eq!(
            a.link,
            Some(AnnouncementLink {
                url: "https://legacy.wormhole.systems".into(),
                label: "Read more".into(),
            })
        );
        assert!(a.dismissible);
        assert_eq!(a.id.len(), 12);
    }

    #[test]
    fn an_unknown_level_is_info() {
        let a = Announcement::from_vars(AnnouncementVars {
            level: Some("nonsense"),
            ..with_message("Scheduled maintenance tonight.")
        })
        .unwrap();
        assert_eq!(a.level, AnnouncementLevel::Info);
    }

    #[test]
    fn a_link_without_a_label_is_labelled_with_its_url() {
        let a = Announcement::from_vars(AnnouncementVars {
            link_url: Some("https://legacy.wormhole.systems"),
            ..with_message("We are migrating.")
        })
        .unwrap();
        assert_eq!(a.link.unwrap().label, "https://legacy.wormhole.systems");
    }

    #[test]
    fn a_label_without_a_url_is_no_link() {
        let a = Announcement::from_vars(AnnouncementVars {
            link_label: Some("Read more"),
            ..with_message("We are migrating.")
        })
        .unwrap();
        assert_eq!(a.link, None);
    }

    #[test]
    fn can_be_made_permanent() {
        for no in ["false", "FALSE", "0", "no", "off"] {
            let a = Announcement::from_vars(AnnouncementVars {
                dismissible: Some(no),
                ..with_message("We are migrating.")
            })
            .unwrap();
            assert!(!a.dismissible, "{no} left it dismissible");
        }
        let a = Announcement::from_vars(AnnouncementVars {
            dismissible: Some("true"),
            ..with_message("We are migrating.")
        })
        .unwrap();
        assert!(a.dismissible);
    }

    #[test]
    fn the_id_changes_with_the_content() {
        let first = Announcement::from_vars(with_message("We are migrating.")).unwrap();
        let again = Announcement::from_vars(with_message("We are migrating.")).unwrap();
        let reworded =
            Announcement::from_vars(with_message("We are migrating on October 1st.")).unwrap();
        let louder = Announcement::from_vars(AnnouncementVars {
            level: Some("critical"),
            ..with_message("We are migrating.")
        })
        .unwrap();
        assert_eq!(first.id, again.id);
        assert_ne!(first.id, reworded.id);
        assert_ne!(first.id, louder.id);
    }
}
