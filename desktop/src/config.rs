//! What the app remembers between runs, as TOML in the platform's config folder. It holds a
//! personal access token, so on Unix the file is readable by its owner only.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

use crate::hotkeys::Action;
use crate::theme::Appearance;

pub const DEFAULT_SERVER: &str = "https://wormhole.systems";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: String,
    pub token: String,
    pub map_id: Option<i64>,
    /// Overrides the folder found on its own.
    pub chatlog_dir: Option<PathBuf>,
    pub copy_on_jump: bool,
    /// The character whose system pastes go to and whose jumps are mapped; `None` follows
    /// whoever moved last.
    pub tracked: Option<String>,
    /// Action to the shortcut that triggers it, in `global-hotkey`'s notation
    /// (`control+alt+KeyR`).
    pub hotkeys: BTreeMap<Action, String>,
    /// What the digits that pick a jump's signature are held with, as in
    /// [`crate::hotkeys::PICK_MODIFIERS`]; empty for bare digits.
    pub pick_modifiers: String,
    pub appearance: Appearance,
    /// The main window's text size in percent.
    pub text_size: u8,
    pub overlay: Overlay,
    /// How the signature list is filtered and sorted, shared by the window and the overlay.
    pub list: ListView,
}

/// The signature list's category filter and sort, like the website's panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListView {
    /// Categories left out. The hidden set is what is kept, so a new category shows.
    pub hidden: Vec<String>,
    pub sort: SortColumn,
    pub descending: bool,
}

impl Default for ListView {
    fn default() -> Self {
        ListView {
            hidden: Vec::new(),
            sort: SortColumn::Id,
            descending: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortColumn {
    Id,
    Category,
    Type,
    Age,
}

/// The signatures window that floats over the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Overlay {
    pub open: bool,
    /// Where it sits until it is dragged somewhere.
    pub corner: Corner,
    /// Its size until it is resized by its grip.
    pub size: OverlaySize,
    /// Where it was dragged to: its top-left corner on the screen.
    pub x: Option<f32>,
    pub y: Option<f32>,
    /// What it was resized to.
    pub width: Option<f32>,
    pub height: Option<f32>,
    /// Background opacity in percent; the text stays solid.
    pub opacity: u8,
    /// Text size in percent, apart from the main window's.
    pub text_size: u8,
    /// Open it when a scan is pasted, so the result is in view.
    pub show_on_paste: bool,
    /// Open it when a jump needs its signature picked.
    pub show_on_jump: bool,
    /// Show it only while an EVE client or this app has focus.
    pub only_when_focused: bool,
    /// With that on, only for these characters' clients; empty for any.
    pub characters: Vec<String>,
}

/// Distance from the screen edges of an overlay in its corner.
pub const OVERLAY_MARGIN: f32 = 24.0;

impl Overlay {
    /// Width and height in logical pixels.
    pub fn dimensions(&self) -> (f32, f32) {
        let (w, h) = self.size.dimensions();
        (self.width.unwrap_or(w), self.height.unwrap_or(h))
    }

    /// The top-left corner on a `screen` of that size: where it was dragged, else its
    /// corner. Kept on the screen, which may have changed since.
    pub fn origin(&self, screen: (f32, f32)) -> (f32, f32) {
        let (w, h) = self.dimensions();
        let m = OVERLAY_MARGIN;
        let (cx, cy) = match self.corner {
            Corner::TopLeft => (m, m),
            Corner::TopRight => (screen.0 - w - m, m),
            Corner::BottomLeft => (m, screen.1 - h - m),
            Corner::BottomRight => (screen.0 - w - m, screen.1 - h - m),
        };
        let x = self.x.unwrap_or(cx).clamp(0.0, (screen.0 - w).max(0.0));
        let y = self.y.unwrap_or(cy).clamp(0.0, (screen.1 - h).max(0.0));
        (x, y)
    }
}

impl Default for Overlay {
    fn default() -> Self {
        Overlay {
            open: false,
            corner: Corner::TopRight,
            size: OverlaySize::Medium,
            x: None,
            y: None,
            width: None,
            height: None,
            opacity: 85,
            text_size: 100,
            show_on_paste: true,
            show_on_jump: true,
            only_when_focused: false,
            characters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlaySize {
    Small,
    Medium,
    Large,
}

impl OverlaySize {
    /// Width and height in logical pixels.
    pub fn dimensions(self) -> (f32, f32) {
        match self {
            OverlaySize::Small => (440.0, 300.0),
            OverlaySize::Medium => (560.0, 420.0),
            OverlaySize::Large => (680.0, 560.0),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: DEFAULT_SERVER.into(),
            token: String::new(),
            map_id: None,
            chatlog_dir: None,
            copy_on_jump: true,
            hotkeys: Action::defaults(),
            pick_modifiers: "control+alt".into(),
            appearance: Appearance::System,
            text_size: 100,
            overlay: Overlay::default(),
            list: ListView::default(),
            tracked: None,
        }
    }
}

pub fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("wormholesystems").join("desktop.toml"))
}

impl Config {
    /// The saved config, or the defaults when there is none. One that does not parse is
    /// moved aside to `desktop.toml.broken` rather than overwritten by the next save, and
    /// the second value says so.
    pub fn load() -> (Config, Option<String>) {
        let Some(path) = path() else {
            return (Config::default(), None);
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return (Config::default(), None);
        };
        match toml::from_str::<Config>(&text) {
            Ok(mut config) => {
                // Actions added since the file was written get their default keys. One the
                // user cleared is saved as empty, so it stays cleared.
                for (action, key) in Action::defaults() {
                    config.hotkeys.entry(action).or_insert(key);
                }
                (config, None)
            }
            Err(err) => {
                let aside = path.with_extension("toml.broken");
                let _ = std::fs::rename(&path, &aside);
                let problem = format!(
                    "The settings file did not parse and was moved to {}: {}",
                    aside.display(),
                    err.message()
                );
                (Config::default(), Some(problem))
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = path().context("no config folder on this system")?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}
