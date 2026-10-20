//! The look: EVE's Photon in the dark theme (near-black, a cyan accent, Saira Semi
//! Condensed for the game's condensed face), the website's in the light one, and the same
//! colors for system classes and holes as the website.

use std::borrow::Cow;
use std::rc::Rc;

use anyhow::Result;
use gpui_kit::component::{Theme, ThemeConfig, ThemeMode, ThemeSet};
use gpui_kit::*;
use serde::{Deserialize, Serialize};

/// The face for everything, EVE's condensed look.
pub const UI: &str = "Saira SemiCondensed";
/// Headings use the same face, heavier.
pub const HEADING: &str = UI;

const THEMES: &str = include_str!("../assets/theme.json");

const FONTS: [&[u8]; 7] = [
    include_bytes!("../assets/fonts/SairaSemiCondensed-Regular.ttf"),
    include_bytes!("../assets/fonts/SairaSemiCondensed-Medium.ttf"),
    include_bytes!("../assets/fonts/SairaSemiCondensed-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
];

/// The kit's Lucide icons, plus the WormholeSystems mark at `logo.svg`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == "logo.svg" {
            return Ok(Some(Cow::Borrowed(include_bytes!("../assets/logo.svg"))));
        }
        gpui_kit::assets::AllAssets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_kit::assets::AllAssets.list(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

impl Appearance {
    pub fn next(self) -> Appearance {
        match self {
            Appearance::System => Appearance::Dark,
            Appearance::Dark => Appearance::Light,
            Appearance::Light => Appearance::System,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Appearance::System => "icons/monitor.svg",
            Appearance::Light => "icons/sun.svg",
            Appearance::Dark => "icons/moon.svg",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Appearance::System => "System",
            Appearance::Light => "Light",
            Appearance::Dark => "Dark",
        }
    }
}

/// Load the fonts and both themes. Once, before the first window.
pub fn init(cx: &mut App) {
    if let Err(err) = cx
        .text_system()
        .add_fonts(FONTS.iter().map(|f| Cow::Borrowed(*f)).collect())
    {
        eprintln!("could not load the fonts: {err}");
    }
    let set: ThemeSet = serde_json::from_str(THEMES).expect("assets/theme.json is valid");
    let theme = Theme::global_mut(cx);
    // Both slots, so switching modes later picks ours rather than the kit's.
    for config in set.themes {
        theme.apply_config(&Rc::new(config));
    }
}

/// Switch to `appearance`, resolving System against the window's.
pub fn apply(appearance: Appearance, window: &mut Window, cx: &mut App) {
    let dark = match appearance {
        Appearance::Dark => true,
        Appearance::Light => false,
        Appearance::System => matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        ),
    };
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    let config: Rc<ThemeConfig> = {
        let theme = Theme::global(cx);
        if dark {
            theme.dark_theme.clone()
        } else {
            theme.light_theme.clone()
        }
    };
    #[cfg(windows)]
    crate::native::set_window_icon(window, dark);
    // A theme brings its own font size; keep the one set in Settings.
    let font_size = Theme::global(cx).font_size;
    Theme::change(mode, Some(window), cx);
    Theme::global_mut(cx).apply_config(&config);
    Theme::global_mut(cx).font_size = font_size;
    cx.refresh_windows();
}

/// The main window's text at `percent` of the base size. The overlay has its own.
pub fn set_text_size(percent: u8, cx: &mut App) {
    Theme::global_mut(cx).font_size = px(crate::ui::scaled_font(percent));
    cx.refresh_windows();
}

/// `#rrggbb` as a color.
pub fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

/// A CSS `oklch(l c h)` color, `l` from 0 to 1. The website's tokens are written this way,
/// so they are copied as they are rather than as approximations.
pub fn oklch(l: f32, c: f32, h: f32) -> Hsla {
    let (a, b) = (c * h.to_radians().cos(), c * h.to_radians().sin());
    let l_ = (l + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m_ = (l - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s_ = (l - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    let linear = [
        4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_,
        -1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_,
    ];
    let [r, g, b] = linear.map(|v| {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.003_130_8 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    });
    Rgba { r, g, b, a: 1.0 }.into()
}

pub fn critical() -> Hsla {
    hex(0xef4444)
}
pub fn reduced() -> Hsla {
    hex(0xf97316)
}
pub fn eol() -> Hsla {
    hex(0xa855f7)
}

/// A class's short label and color, from `frontend/src/lib/map/classes.ts`: `C5`, `H`, `L`,
/// `N`, `P`, and `?` for a ghost.
pub fn class_meta(wormhole_class_id: Option<i64>, security: Option<f64>) -> (&'static str, Hsla) {
    if wormhole_class_id.is_none() && security.is_none() {
        return ("?", unknown_class());
    }
    let class = wormhole_class_id
        .unwrap_or_else(|| crate::naming::class_from_security(security.unwrap_or(0.0)));
    dest_class_meta(Some(class))
}

fn unknown_class() -> Hsla {
    oklch(0.708, 0.0, 0.0)
}

/// The same for a static's destination class.
pub fn dest_class_meta(class: Option<i64>) -> (&'static str, Hsla) {
    let drifter = || oklch(0.712, 0.194, 13.428);
    match class {
        Some(1) => ("C1", oklch(0.865, 0.127, 207.078)),
        Some(2) => ("C2", oklch(0.623, 0.214, 259.815)),
        Some(3) => ("C3", oklch(0.827, 0.119, 306.383)),
        Some(4) => ("C4", oklch(0.606, 0.25, 292.717)),
        Some(5) => ("C5", oklch(0.75, 0.183, 55.934)),
        Some(6) => ("C6", oklch(0.637, 0.237, 25.331)),
        Some(7) => ("H", oklch(0.723, 0.219, 149.579)),
        Some(8) => ("L", oklch(0.705, 0.213, 47.604)),
        Some(9) => ("N", oklch(0.637, 0.237, 25.331)),
        Some(12) => ("C12", oklch(0.855, 0.138, 181.071)),
        Some(13) => ("C13", oklch(0.74, 0.238, 322.16)),
        Some(14) => ("C14", drifter()),
        Some(15) => ("C15", drifter()),
        Some(16) => ("C16", drifter()),
        Some(17) => ("C17", drifter()),
        Some(18) => ("C18", drifter()),
        Some(19) => ("C19", unknown_class()),
        Some(20) => ("C20", unknown_class()),
        Some(21) => ("C21", unknown_class()),
        Some(22) => ("C22", unknown_class()),
        Some(23) => ("C23", unknown_class()),
        Some(25) => ("P", oklch(0.505, 0.213, 27.518)),
        _ => ("?", unknown_class()),
    }
}

/// A signature group's short label and color, from `CATEGORIES` in
/// `frontend/src/lib/map/signatures.ts`.
pub fn group_meta(group: &str) -> (&'static str, Option<Hsla>) {
    match group {
        "wormhole" => ("WH", Some(oklch(0.746, 0.16, 232.661))),
        "data" => ("Data", Some(oklch(0.789, 0.154, 211.53))),
        "relic" => ("Relic", Some(oklch(0.828, 0.189, 84.429))),
        "ore" => ("Ore", Some(oklch(0.852, 0.199, 91.936))),
        "gas" => ("Gas", Some(oklch(0.75, 0.183, 55.934))),
        "combat" => ("Combat", Some(oklch(0.792, 0.209, 151.711))),
        "homefront" => ("HF", Some(oklch(0.712, 0.194, 13.428))),
        "faction_warfare" => ("FW", Some(oklch(0.74, 0.238, 322.16))),
        _ => ("—", None),
    }
}

/// The website's icon for a signature category.
pub fn group_icon(group: &str) -> &'static str {
    match group {
        "wormhole" => "icons/fan.svg",
        "data" => "icons/database.svg",
        "relic" => "icons/landmark.svg",
        "ore" => "icons/gem.svg",
        "gas" => "icons/cloud.svg",
        "combat" => "icons/swords.svg",
        "homefront" => "icons/shield.svg",
        "faction_warfare" => "icons/flag.svg",
        _ => "icons/circle-question-mark.svg",
    }
}

/// A neutral dot for a healthy state.
pub fn healthy() -> Hsla {
    oklch(0.556, 0.007, 260.0)
}

/// The categories a signature can be put in, in the website's order: group, category id
/// in the catalog, and the name its picker shows.
pub const CATEGORIES: [(&str, i64, &str); 8] = [
    ("wormhole", 1, "Wormhole"),
    ("data", 2, "Data Site"),
    ("relic", 3, "Relic Site"),
    ("ore", 6, "Ore Site"),
    ("gas", 5, "Gas Site"),
    ("combat", 4, "Combat Site"),
    ("homefront", 7, "Homefront Operations"),
    ("faction_warfare", 8, "Factional Warfare Site"),
];

/// Where a destination class sorts in a type picker: k-space, then Pochven, then the
/// wormhole classes, like the website.
pub fn dest_sort_weight(class: Option<i64>) -> i64 {
    match class {
        Some(7) => 0,
        Some(8) => 1,
        Some(9) => 2,
        Some(25) => 3,
        Some(c @ 1..=6) => 10 + c,
        Some(c @ 12..=23) => 10 + c,
        _ => 99,
    }
}

/// What a paste did to a row: added, refreshed, or no longer seen.
pub fn added() -> Hsla {
    oklch(0.723, 0.219, 149.579)
}
pub fn refreshed() -> Hsla {
    oklch(0.769, 0.188, 70.08)
}
pub fn missing() -> Hsla {
    oklch(0.637, 0.237, 25.331)
}
