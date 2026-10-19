//! Connection bookmark names, ported from the website's `frontend/src/lib/naming/bookmark.ts`
//! so a name copied here is the name the map would give. The tests are that file's tests; a
//! change to one side belongs on the other.

use crate::api::{
    Connection, MapNaming, MassStatus, PlacedSystem, Signature, TimeStatus, WormholeSize,
};

pub const DEFAULT_FORMAT_WORMHOLE: &str = "{alias} {sig} {class}";
pub const DEFAULT_FORMAT_KSPACE: &str = "{alias} {class} {sig} {name} {region}";
/// The leading `*` sorts the way home to the top of the in-game folder.
pub const DEFAULT_FORMAT_RETURN: &str = "*{alias} {sig} {class}";
pub const DEFAULT_IGNORED_ALIAS: &str = "HOME";

/// A system as a bookmark describes it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BookmarkSystem {
    pub alias: Option<String>,
    pub name: String,
    pub region: Option<String>,
    pub wormhole_class_id: Option<i64>,
    /// `None` when the far side is unknown, which blanks the class rather than guessing.
    pub security: Option<f64>,
    pub occupier: Option<String>,
    /// The map's home system: a bookmark pointing there is the way home, alias or not.
    pub is_home: bool,
}

/// What a bookmark says about the hole itself.
#[derive(Debug, Clone, Default)]
pub struct BookmarkContext {
    /// The alias of the system the bookmark is made in, for `{current}`.
    pub current_alias: Option<String>,
    pub signature_id: Option<String>,
    pub size: Option<WormholeSize>,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    /// The wormhole code, e.g. `H296`.
    pub wormhole_code: Option<String>,
}

/// The map's formats. A blank or missing one falls back to the default.
#[derive(Debug, Clone, Default)]
pub struct BookmarkFormats {
    pub wormhole: Option<String>,
    pub kspace: Option<String>,
    pub ret: Option<String>,
    pub ignored_alias: Option<String>,
}

impl From<&MapNaming> for BookmarkFormats {
    fn from(naming: &MapNaming) -> Self {
        BookmarkFormats {
            wormhole: Some(naming.bookmark_wormhole.clone()),
            kspace: Some(naming.bookmark_kspace.clone()),
            ret: Some(naming.bookmark_return.clone()),
            ignored_alias: Some(naming.ignored_alias.clone()),
        }
    }
}

fn is_ignored_alias(alias: Option<&str>, ignored: Option<&str>) -> bool {
    let ignored = ignored.unwrap_or("").trim();
    !ignored.is_empty() && alias.unwrap_or("").trim().eq_ignore_ascii_case(ignored)
}

/// Whether naming `destination` describes the way back up the chain. Chain aliases extend
/// their parent's, so an ancestor is always a prefix of the alias you are standing on, while
/// a sibling branch (`B` seen from `AB`) is not.
pub fn is_return_bookmark(
    destination: Option<&str>,
    opposite: Option<&str>,
    ignored: Option<&str>,
) -> bool {
    let opposite = opposite.unwrap_or("").trim();
    if opposite.is_empty() {
        return false;
    }
    if is_ignored_alias(destination, ignored) {
        return true;
    }
    let destination = destination.unwrap_or("").trim();
    !destination.is_empty()
        && opposite
            .to_lowercase()
            .starts_with(&destination.to_lowercase())
}

/// Classes 1 to 6 and 12 to 18 are wormhole space; 7, 8 and 9 high, low and null; 25 Pochven.
pub fn class_short(id: i64) -> Option<&'static str> {
    Some(match id {
        1 => "C1",
        2 => "C2",
        3 => "C3",
        4 => "C4",
        5 => "C5",
        6 => "C6",
        7 => "H",
        8 => "L",
        9 => "N",
        12 => "C12",
        13 => "C13",
        14 => "C14",
        15 => "C15",
        16 => "C16",
        17 => "C17",
        18 => "C18",
        19 => "C19",
        20 => "C20",
        21 => "C21",
        22 => "C22",
        23 => "C23",
        25 => "P",
        _ => return None,
    })
}

pub fn is_wormhole_class(id: Option<i64>) -> bool {
    matches!(id, Some(1..=6 | 12..=18))
}

/// Security the way the game shows it: 0.0 stays, anything in (0, 0.05) is 0.1, the rest
/// rounds to the nearest tenth, halves away from zero.
fn ccp_round_security(security: f64) -> f64 {
    if security == 0.0 {
        return 0.0;
    }
    if security > 0.0 && security < 0.05 {
        return 0.1;
    }
    let scaled = security * 10.0;
    scaled.signum() * scaled.abs().round() / 10.0
}

pub fn class_from_security(security: f64) -> i64 {
    let rounded = ccp_round_security(security);
    if rounded >= 0.5 {
        7
    } else if rounded >= 0.1 {
        8
    } else {
        9
    }
}

/// `C3` for wormhole space, otherwise `HS` / `LS` / `NS` / `P`. Blank when the destination is
/// unknown, since a security of zero would label an unscanned hole as lowsec.
pub fn bookmark_class(wormhole_class_id: Option<i64>, security: Option<f64>) -> String {
    if wormhole_class_id.is_none() && security.is_none() {
        return String::new();
    }
    let id = wormhole_class_id.unwrap_or_else(|| class_from_security(security.unwrap_or(0.0)));
    match class_short(id).unwrap_or("?") {
        "H" => "HS".into(),
        "L" => "LS".into(),
        "N" => "NS".into(),
        short => short.into(),
    }
}

/// The scanner id's first three characters: `ABC-123` is bookmarked as `ABC`.
pub fn short_signature_id(signature_id: Option<&str>) -> String {
    signature_id
        .map(|s| s.chars().take(3).collect())
        .unwrap_or_default()
}

fn token_value(token: &str, system: &BookmarkSystem, context: &BookmarkContext) -> Option<String> {
    let text = |value: Option<&str>| value.unwrap_or("").to_string();
    Some(match token {
        "alias" => text(system.alias.as_deref()),
        "current" => text(context.current_alias.as_deref().map(str::trim)),
        "sig" => short_signature_id(context.signature_id.as_deref()),
        "class" => bookmark_class(system.wormhole_class_id, system.security),
        "name" => system.name.clone(),
        "region" => text(system.region.as_deref()),
        "occupier" => text(system.occupier.as_deref()),
        // Large is the common case and deliberately blank.
        "size" => match context.size {
            Some(WormholeSize::Small) => "SM",
            Some(WormholeSize::Medium) => "MD",
            Some(WormholeSize::Xl) => "XM",
            _ => "",
        }
        .into(),
        "wh" => text(context.wormhole_code.as_deref()),
        // Stable says nothing, so mass only shows once the hole has degraded.
        "mass" => match context.mass_status {
            Some(MassStatus::Reduced) => "reduced",
            Some(MassStatus::Critical) => "crit",
            _ => "",
        }
        .into(),
        "life" => match context.time_status {
            Some(TimeStatus::Eol) => "EOL",
            Some(TimeStatus::Critical) => "EOL!",
            Some(TimeStatus::Expired) => "EOL!!",
            _ => "",
        }
        .into(),
        _ => return None,
    })
}

/// Substitute `{token}` placeholders and collapse the gaps empty ones leave. An unknown
/// placeholder stays as written, so a typo in the format is visible.
pub fn render_bookmark(
    template: &str,
    system: &BookmarkSystem,
    context: &BookmarkContext,
) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close)
                if !after[..close].is_empty()
                    && after[..close]
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_') =>
            {
                let token = &after[..close];
                match token_value(token, system, context) {
                    Some(value) => out.push_str(&value),
                    None => {
                        out.push('{');
                        out.push_str(token);
                        out.push('}');
                    }
                }
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn non_blank(format: Option<&String>) -> Option<&str> {
    format.map(String::as_str).filter(|f| !f.is_empty())
}

/// The bookmark name for a system across a connection. `opposite_alias` is the alias at the
/// other end, where the bookmark is made: it fills `{current}`, and when it marks this bookmark as the way back up the chain, or the system is
/// home, the return format replaces the wormhole or k-space one.
pub fn format_bookmark(
    system: &BookmarkSystem,
    context: &BookmarkContext,
    formats: &BookmarkFormats,
    opposite_alias: Option<&str>,
) -> String {
    let ignored = formats
        .ignored_alias
        .as_deref()
        .unwrap_or(DEFAULT_IGNORED_ALIAS);
    let template = if system.is_home
        || is_return_bookmark(system.alias.as_deref(), opposite_alias, Some(ignored))
    {
        non_blank(formats.ret.as_ref()).unwrap_or(DEFAULT_FORMAT_RETURN)
    } else if is_wormhole_class(system.wormhole_class_id) {
        non_blank(formats.wormhole.as_ref()).unwrap_or(DEFAULT_FORMAT_WORMHOLE)
    } else {
        non_blank(formats.kspace.as_ref()).unwrap_or(DEFAULT_FORMAT_KSPACE)
    };
    let context = BookmarkContext {
        current_alias: opposite_alias.map(Into::into),
        ..context.clone()
    };
    render_bookmark(template, system, &context)
}

pub fn bookmark_system_of(system: &PlacedSystem) -> BookmarkSystem {
    BookmarkSystem {
        alias: system.alias.clone(),
        name: system.name.clone(),
        region: Some(system.region.clone()),
        wormhole_class_id: system.wormhole_class_id,
        security: Some(system.security_status),
        occupier: system.occupying_group.clone(),
        is_home: system.is_home,
    }
}

/// The bookmark for `end` of a connection, made while standing in `near` at the other end.
/// It carries the signature scanned on the near side, since that is where the scanner was;
/// the name describes the far side, which is where the hole goes.
pub fn connection_bookmark(
    connection: &Connection,
    end: &PlacedSystem,
    near: &PlacedSystem,
    signatures: &[Signature],
    code_of: impl Fn(Option<i64>) -> Option<String>,
    formats: &BookmarkFormats,
) -> String {
    let sig = signatures.iter().find(|s| {
        s.connection_id == Some(connection.id) && s.solar_system_id == near.solar_system_id
    });
    format_bookmark(
        &bookmark_system_of(end),
        &BookmarkContext {
            current_alias: None,
            signature_id: sig.map(|s| s.signature_id.clone()),
            size: connection.size,
            mass_status: connection.mass_status,
            time_status: connection.time_status,
            wormhole_code: sig.and_then(|s| code_of(s.signature_type_id)),
        },
        formats,
        near.alias.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wormhole() -> BookmarkSystem {
        BookmarkSystem {
            alias: Some("D2".into()),
            name: "J122515".into(),
            region: Some("E-R00024".into()),
            wormhole_class_id: Some(5),
            security: Some(-0.99),
            occupier: None,
            is_home: false,
        }
    }

    fn kspace() -> BookmarkSystem {
        BookmarkSystem {
            alias: None,
            name: "Jita".into(),
            region: Some("The Forge".into()),
            wormhole_class_id: None,
            security: Some(0.95),
            occupier: None,
            is_home: false,
        }
    }

    fn context() -> BookmarkContext {
        BookmarkContext {
            signature_id: Some("ABC-123".into()),
            ..Default::default()
        }
    }

    fn wormhole_format(format: &str) -> BookmarkFormats {
        BookmarkFormats {
            wormhole: Some(format.into()),
            ..Default::default()
        }
    }

    #[test]
    fn class_uses_the_wormhole_class_and_two_letters_for_kspace() {
        assert_eq!(bookmark_class(Some(5), Some(-0.99)), "C5");
        assert_eq!(bookmark_class(None, Some(0.95)), "HS");
        assert_eq!(bookmark_class(None, Some(0.3)), "LS");
        assert_eq!(bookmark_class(None, Some(-0.5)), "NS");
        assert_eq!(bookmark_class(Some(7), Some(0.9)), "HS");
    }

    #[test]
    fn short_signature_id_keeps_three_characters() {
        assert_eq!(short_signature_id(Some("ABC-123")), "ABC");
        assert_eq!(short_signature_id(None), "");
    }

    #[test]
    fn names_where_the_bookmark_is_made() {
        let formats = BookmarkFormats {
            wormhole: Some("{alias} {sig} {current}".into()),
            ret: Some("*{alias} {sig} {current}".into()),
            ..Default::default()
        };
        let deeper = BookmarkSystem {
            alias: Some("AB".into()),
            ..wormhole()
        };
        let back = BookmarkSystem {
            alias: Some("A".into()),
            ..wormhole()
        };
        assert_eq!(
            format_bookmark(&deeper, &context(), &formats, Some("A")),
            "AB ABC A"
        );
        assert_eq!(
            format_bookmark(&back, &context(), &formats, Some("AB")),
            "*A ABC AB"
        );
        assert_eq!(
            format_bookmark(&wormhole(), &context(), &formats, None),
            "D2 ABC"
        );
    }

    #[test]
    fn render_drops_empty_tokens_and_keeps_unknown_ones() {
        let mut system = wormhole();
        system.region = None;
        assert_eq!(
            render_bookmark("{alias} {sig} {class} {region}", &system, &context()),
            "D2 ABC C5"
        );
        assert_eq!(
            render_bookmark("{alias} {nope}", &system, &context()),
            "D2 {nope}"
        );
    }

    #[test]
    fn formats_by_kind_of_space() {
        let none = BookmarkFormats::default();
        assert_eq!(
            format_bookmark(&wormhole(), &context(), &none, None),
            "D2 ABC C5"
        );
        assert_eq!(
            format_bookmark(&kspace(), &context(), &none, None),
            "HS ABC Jita The Forge"
        );
        assert_eq!(
            format_bookmark(
                &wormhole(),
                &context(),
                &wormhole_format("{class} {alias} [{sig}]"),
                None
            ),
            "C5 D2 [ABC]"
        );
    }

    #[test]
    fn names_only_restrictive_sizes() {
        let format = wormhole_format("{alias} {sig} {size}");
        let sized = |size| BookmarkContext {
            size: Some(size),
            ..context()
        };
        assert_eq!(
            format_bookmark(&wormhole(), &sized(WormholeSize::Small), &format, None),
            "D2 ABC SM"
        );
        assert_eq!(
            format_bookmark(&wormhole(), &sized(WormholeSize::Medium), &format, None),
            "D2 ABC MD"
        );
        assert_eq!(
            format_bookmark(&wormhole(), &sized(WormholeSize::Xl), &format, None),
            "D2 ABC XM"
        );
        assert_eq!(
            format_bookmark(&wormhole(), &sized(WormholeSize::Large), &format, None),
            "D2 ABC"
        );
    }

    #[test]
    fn mass_and_life_show_once_degraded() {
        let format = wormhole_format("{alias} {sig} {mass} {life}");
        let status = |mass, time| BookmarkContext {
            mass_status: Some(mass),
            time_status: Some(time),
            ..context()
        };
        assert_eq!(
            format_bookmark(
                &wormhole(),
                &status(MassStatus::Stable, TimeStatus::Stable),
                &format,
                None
            ),
            "D2 ABC"
        );
        assert_eq!(
            format_bookmark(
                &wormhole(),
                &status(MassStatus::Critical, TimeStatus::Eol),
                &format,
                None
            ),
            "D2 ABC crit EOL"
        );
        assert_eq!(
            format_bookmark(
                &wormhole(),
                &status(MassStatus::Stable, TimeStatus::Critical),
                &format,
                None
            ),
            "D2 ABC EOL!"
        );
        assert_eq!(
            format_bookmark(
                &wormhole(),
                &status(MassStatus::Stable, TimeStatus::Expired),
                &format,
                None
            ),
            "D2 ABC EOL!!"
        );
    }

    #[test]
    fn return_bookmarks() {
        assert!(is_return_bookmark(Some("A"), Some("AB"), Some("HOME")));
        assert!(!is_return_bookmark(Some("B"), Some("AB"), Some("HOME")));
        assert!(is_return_bookmark(Some("HOME"), Some("AB"), Some("HOME")));
        assert!(!is_return_bookmark(Some("A"), None, Some("HOME")));
        let mut system = wormhole();
        system.alias = Some("A".into());
        assert_eq!(
            format_bookmark(&system, &context(), &BookmarkFormats::default(), Some("AB")),
            "*A ABC C5"
        );
    }

    #[test]
    fn an_unmapped_hole_never_guesses_a_class() {
        let mut unknown = BookmarkSystem::default();
        let none = BookmarkFormats::default();
        assert_eq!(format_bookmark(&unknown, &context(), &none, None), "ABC");
        unknown.wormhole_class_id = Some(5);
        assert_eq!(format_bookmark(&unknown, &context(), &none, None), "ABC C5");
    }

    fn placed(
        id: i64,
        solar_system_id: i64,
        name: &str,
        alias: &str,
        class: Option<i64>,
        sec: f64,
    ) -> PlacedSystem {
        PlacedSystem {
            id,
            solar_system_id,
            name: name.into(),
            alias: Some(alias.into()),
            region: String::new(),
            wormhole_class_id: class,
            security_status: sec,
            occupying_group: None,
            is_home: false,
            position_x: 0.0,
            position_y: 0.0,
            statics: Vec::new(),
        }
    }

    #[test]
    fn each_end_carries_the_signature_read_on_the_other() {
        let home = placed(1, 30000142, "Jita", "A", None, 0.95);
        let c5 = placed(2, 31000005, "J122515", "A1", Some(5), -0.99);
        let hole = Connection {
            id: 7,
            from_system: 1,
            to_system: 2,
            kind: crate::api::ConnectionKind::Wormhole,
            size: None,
            mass_status: None,
            time_status: None,
            preserve_mass: false,
            created_at: None,
        };
        let sig = |signature_id: &str, solar_system_id, type_id| Signature {
            group: "wormhole".into(),
            signature_id: signature_id.into(),
            solar_system_id,
            connection_id: Some(7),
            signature_type_id: Some(type_id),
            ..Default::default()
        };
        let sigs = [sig("HOM-111", 30000142, 1), sig("FAR-222", 31000005, 2)];
        let codes = |id: Option<i64>| match id {
            Some(1) => Some("H296".to_string()),
            Some(2) => Some("K162".to_string()),
            _ => None,
        };
        let formats = BookmarkFormats {
            wormhole: Some("{alias} {sig} {wh}".into()),
            kspace: Some("{alias} {sig} {wh}".into()),
            ret: Some("*{alias} {sig} {wh}".into()),
            ignored_alias: None,
        };
        // Standing in the C5, the way back to Jita carries the C5's own signature.
        assert_eq!(
            connection_bookmark(&hole, &home, &c5, &sigs, codes, &formats),
            "*A FAR K162"
        );
        assert_eq!(
            connection_bookmark(&hole, &c5, &home, &sigs, codes, &formats),
            "A1 HOM H296"
        );
        // Not scanned on the near side yet: no signature.
        assert_eq!(
            connection_bookmark(&hole, &c5, &home, &sigs[1..], codes, &formats),
            "A1"
        );
    }

    #[test]
    fn a_jump_away_from_home_copies_the_way_back() {
        let formats = BookmarkFormats::default();
        let hole = |from, to| Connection {
            id: 9,
            from_system: from,
            to_system: to,
            kind: crate::api::ConnectionKind::Wormhole,
            size: None,
            mass_status: None,
            time_status: None,
            preserve_mass: false,
            created_at: None,
        };
        let one = placed(2, 31000001, "J100001", "1", Some(3), -1.0);
        let deeper = placed(3, 31000002, "J100002", "11", Some(5), -1.0);
        // Nested: out of 1 into 11, the bookmark is the star one back to 1.
        let back = connection_bookmark(&hole(2, 3), &one, &deeper, &[], |_| None, &formats);
        assert_eq!(back, "*1 C3");
        // Flying back into 1, the bookmark is the hole on into 11.
        let on = connection_bookmark(&hole(2, 3), &deeper, &one, &[], |_| None, &formats);
        assert_eq!(on, "11 C5");
        // Out of a home named with the ignored alias, the way back is a star one too.
        let home = placed(1, 30000142, "Jita", "HOME", None, 0.95);
        let back = connection_bookmark(&hole(1, 2), &home, &one, &[], |_| None, &formats);
        assert!(back.starts_with('*'), "{back}");
        // A home system without an alias is still the way back.
        let home = PlacedSystem {
            alias: None,
            is_home: true,
            ..placed(1, 30002086, "Turnur", "", None, 0.4)
        };
        let back = connection_bookmark(&hole(1, 2), &home, &one, &[], |_| None, &formats);
        assert_eq!(back, "* LS");
    }
}
