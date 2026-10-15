//! The panel arrangement a viewer stores against a map: the wire shape the client sends,
//! and the check that it describes something the client could actually draw.

use serde::{Deserialize, Serialize};

use crate::maps::error::{MapError, Result};

/// Tile positions keyed by breakpoint: the built-in `xs` / `sm` / `md` / `lg`, plus any the
/// viewer added for wider screens.
pub type PanelLayouts = std::collections::BTreeMap<String, BreakpointLayout>;

/// One breakpoint's arrangement.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct BreakpointLayout {
    pub cols: i32,
    pub row_height: i32,
    pub items: Vec<LayoutItem>,
    /// The narrowest window the breakpoint applies to. Required on an added one; on a
    /// built-in one it is only there if the viewer moved it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_width: Option<i32>,
    /// What the toolbar calls the breakpoint. Required on an added one, a rename on a
    /// built-in one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
}

/// One tile. Minimum sizes deliberately live in the client's panel registry instead, so
/// tightening one still reaches people who have already saved a layout.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct LayoutItem {
    /// Panel id. Named `i` to match the stored shape.
    pub i: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Panels the layout may refer to. The server keeps its own copy so a bad payload is a
/// 400 rather than a page that renders a tile nothing knows how to draw.
pub const PANEL_IDS: [&str; 10] = [
    "map",
    "navigation",
    "system-info",
    "threat",
    "signatures",
    "notes",
    "characters",
    "skyhooks",
    "killmails",
    "evescout",
];

const BREAKPOINT_KEYS: [&str; 4] = ["xs", "sm", "md", "lg"];

/// Enough for every screen anyone owns, and few enough that the toolbar stays a toolbar.
const MAX_BREAKPOINTS: usize = 12;

/// Reject anything that would not render: unknown ids, duplicates, or a tile outside the
/// grid it claims to be in.
pub fn validate_layouts(layouts: &PanelLayouts) -> Result<()> {
    if layouts.len() > MAX_BREAKPOINTS {
        return Err(invalid(format!("at most {MAX_BREAKPOINTS} breakpoints")));
    }
    for (key, layout) in layouts {
        if BREAKPOINT_KEYS.contains(&key.as_str()) {
            validate_renamed(key, layout)?;
        } else {
            validate_added(key, layout)?;
        }
        if !(1..=24).contains(&layout.cols) {
            return Err(invalid("cols must be between 1 and 24"));
        }
        if !(40..=500).contains(&layout.row_height) {
            return Err(invalid("row height must be between 40 and 500"));
        }
        let mut seen = std::collections::HashSet::new();
        for item in &layout.items {
            if !PANEL_IDS.contains(&item.i.as_str()) {
                return Err(invalid(format!("unknown panel {}", item.i)));
            }
            if !seen.insert(item.i.as_str()) {
                return Err(invalid(format!("{} listed twice", item.i)));
            }
            if item.w < 1 || item.h < 1 {
                return Err(invalid("a tile must be at least 1x1"));
            }
            if item.x < 0 || item.y < 0 || item.x + item.w > layout.cols {
                return Err(invalid(format!("{} does not fit the {key} grid", item.i)));
            }
        }
    }
    Ok(())
}

/// A built-in breakpoint may be renamed or moved, within the same limits as an added one.
fn validate_renamed(key: &str, layout: &BreakpointLayout) -> Result<()> {
    if let Some(width) = layout.min_width
        && !(0..=10_000).contains(&width)
    {
        return Err(invalid(format!("{key} needs a width between 0 and 10000")));
    }
    if let Some(label) = layout.label.as_deref()
        && !(1..=32).contains(&label.trim().chars().count())
    {
        return Err(invalid(format!(
            "{key} needs a name of up to 32 characters"
        )));
    }
    Ok(())
}

/// An added breakpoint needs a sane key, a width to switch at and a name to show.
fn validate_added(key: &str, layout: &BreakpointLayout) -> Result<()> {
    let key_ok = (1..=32).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !key_ok {
        return Err(invalid(format!("unknown breakpoint {key}")));
    }
    match layout.min_width {
        Some(width) if (1..=10_000).contains(&width) => {}
        _ => return Err(invalid(format!("{key} needs a width between 1 and 10000"))),
    }
    match layout.label.as_deref().map(str::trim) {
        Some(label) if (1..=32).contains(&label.chars().count()) => Ok(()),
        _ => Err(invalid(format!(
            "{key} needs a name of up to 32 characters"
        ))),
    }
}

fn invalid(message: impl Into<String>) -> MapError {
    MapError::Validation(message.into())
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    fn layout(cols: i32, items: Vec<LayoutItem>) -> PanelLayouts {
        PanelLayouts::from([(
            "lg".to_string(),
            BreakpointLayout {
                cols,
                row_height: 100,
                items,
                min_width: None,
                label: None,
            },
        )])
    }

    fn added(key: &str, min_width: Option<i32>, label: Option<&str>) -> PanelLayouts {
        PanelLayouts::from([(
            key.to_string(),
            BreakpointLayout {
                cols: 12,
                row_height: 100,
                items: vec![],
                min_width,
                label: label.map(Into::into),
            },
        )])
    }

    fn tile(i: &str, x: i32, y: i32, w: i32, h: i32) -> LayoutItem {
        LayoutItem {
            i: i.into(),
            x,
            y,
            w,
            h,
        }
    }

    #[test]
    fn accepts_a_sane_arrangement() {
        let ok = layout(10, vec![tile("map", 0, 0, 7, 9), tile("notes", 7, 0, 3, 3)]);
        assert!(validate_layouts(&ok).is_ok());
    }

    #[test]
    fn rejects_a_panel_the_client_could_not_draw() {
        // A name no panel will ever have: a plausible future one would stop testing
        // anything the day it ships.
        let bad = layout(10, vec![tile("not-a-panel", 0, 0, 2, 2)]);
        assert!(validate_layouts(&bad).is_err());
    }

    #[test]
    fn rejects_the_same_panel_twice() {
        let bad = layout(
            10,
            vec![tile("notes", 0, 0, 2, 2), tile("notes", 2, 0, 2, 2)],
        );
        assert!(validate_layouts(&bad).is_err());
    }

    #[test]
    fn rejects_a_tile_hanging_off_the_grid() {
        let bad = layout(4, vec![tile("map", 3, 0, 2, 2)]);
        assert!(validate_layouts(&bad).is_err());
    }

    #[test]
    fn accepts_an_added_breakpoint_with_a_width_and_name() {
        assert!(validate_layouts(&added("ultrawide", Some(2560), Some("Ultrawide"))).is_ok());
    }

    #[test]
    fn rejects_an_added_breakpoint_without_a_width_or_name() {
        assert!(validate_layouts(&added("ultrawide", None, Some("Ultrawide"))).is_err());
        assert!(validate_layouts(&added("ultrawide", Some(2560), None)).is_err());
        assert!(validate_layouts(&added("ultrawide", Some(2560), Some("  "))).is_err());
    }

    #[test]
    fn a_built_in_breakpoint_can_be_renamed_and_moved() {
        let mut moved = layout(10, vec![]);
        let lg = moved.get_mut("lg").unwrap();
        lg.min_width = Some(1800);
        lg.label = Some("Big desk".into());
        assert!(validate_layouts(&moved).is_ok());
        moved.get_mut("lg").unwrap().label = Some(String::new());
        assert!(validate_layouts(&moved).is_err());
    }

    #[test]
    fn rejects_a_breakpoint_key_that_is_not_a_slug() {
        assert!(validate_layouts(&added("Ultra Wide", Some(2560), Some("Ultrawide"))).is_err());
    }

    #[test]
    fn rejects_too_many_breakpoints() {
        let many: PanelLayouts = (0..13)
            .map(|n| {
                let (key, layout) = added(&format!("bp-{n}"), Some(2000 + n), Some("Wide"))
                    .into_iter()
                    .next()
                    .unwrap();
                (key, layout)
            })
            .collect();
        assert!(validate_layouts(&many).is_err());
    }

    #[test]
    fn rejects_absurd_grid_geometry() {
        assert!(validate_layouts(&layout(0, vec![])).is_err());
        assert!(validate_layouts(&layout(64, vec![])).is_err());
        let mut tall = layout(4, vec![]);
        tall.get_mut("lg").unwrap().row_height = 4000;
        assert!(validate_layouts(&tall).is_err());
    }

    #[test]
    fn rejects_a_zero_sized_tile() {
        let bad = layout(4, vec![tile("map", 0, 0, 0, 2)]);
        assert!(validate_layouts(&bad).is_err());
    }
}
