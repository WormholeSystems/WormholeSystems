//! The panel arrangement a viewer stores against a map: the wire shape the client sends,
//! and the check that it describes something the client could actually draw.

use serde::{Deserialize, Serialize};

use crate::maps::error::{MapError, Result};

/// Tile positions keyed by breakpoint (`xs` / `sm` / `md` / `lg`).
pub type PanelLayouts = std::collections::BTreeMap<String, BreakpointLayout>;

/// One breakpoint's arrangement.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct BreakpointLayout {
    pub cols: i32,
    pub row_height: i32,
    pub items: Vec<LayoutItem>,
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

/// Reject anything that would not render: unknown ids, duplicates, or a tile outside the
/// grid it claims to be in.
pub fn validate_layouts(layouts: &PanelLayouts) -> Result<()> {
    for (key, layout) in layouts {
        if !BREAKPOINT_KEYS.contains(&key.as_str()) {
            return Err(invalid(format!("unknown breakpoint {key}")));
        }
        if !(1..=24).contains(&layout.cols) {
            return Err(invalid("cols must be between 1 and 24"));
        }
        if !(40..=400).contains(&layout.row_height) {
            return Err(invalid("row height must be between 40 and 400"));
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
    fn rejects_an_unknown_breakpoint() {
        let bad = PanelLayouts::from([(
            "ultrawide".to_string(),
            BreakpointLayout {
                cols: 10,
                row_height: 100,
                items: vec![],
            },
        )]);
        assert!(validate_layouts(&bad).is_err());
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
