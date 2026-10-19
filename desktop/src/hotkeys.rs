//! System-wide shortcuts, so scanning and mapping happen without leaving the game: paste a
//! scan, sweep what it no longer sees, say which signature a jump was, and mark the hole
//! flown last.
//!
//! Registered through `global-hotkey`: works on Windows, macOS and X11. On a Wayland session
//! only X11 windows (EVE under Proton is one) can trigger them.

use std::collections::BTreeMap;
use std::str::FromStr;

use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::{Deserialize, Serialize};

use crate::api::{MassStatus, StatusChange, TimeStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    PasteScan,
    DeleteMissing,
    ToggleOverlay,
    SetAlias,
    CopyBookmark,
    MassReduced,
    MassCritical,
    MassStable,
    Eol,
    LifeStable,
}

impl Action {
    pub const ALL: [Action; 10] = [
        Action::PasteScan,
        Action::DeleteMissing,
        Action::ToggleOverlay,
        Action::SetAlias,
        Action::CopyBookmark,
        Action::MassReduced,
        Action::MassCritical,
        Action::Eol,
        Action::MassStable,
        Action::LifeStable,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Action::PasteScan => "Paste the scan in the clipboard",
            Action::DeleteMissing => "Delete what the scan no longer sees",
            Action::ToggleOverlay => "Show or hide the overlay",
            Action::SetAlias => "Name the system you are in",
            Action::CopyBookmark => "Copy the bookmark again",
            Action::MassReduced => "Mass reduced",
            Action::MassCritical => "Mass critical",
            Action::MassStable => "Mass stable",
            Action::Eol => "End of life",
            Action::LifeStable => "Lifetime stable",
        }
    }

    /// What the action changes on the connection; `None` for one that does not.
    pub fn change(self) -> Option<StatusChange> {
        let mass = |m| StatusChange {
            mass_status: Some(m),
            ..Default::default()
        };
        let time = |t| StatusChange {
            time_status: Some(t),
            ..Default::default()
        };
        match self {
            Action::PasteScan
            | Action::DeleteMissing
            | Action::ToggleOverlay
            | Action::SetAlias
            | Action::CopyBookmark => None,
            Action::MassReduced => Some(mass(MassStatus::Reduced)),
            Action::MassCritical => Some(mass(MassStatus::Critical)),
            Action::MassStable => Some(mass(MassStatus::Stable)),
            Action::Eol => Some(time(TimeStatus::Eol)),
            Action::LifeStable => Some(time(TimeStatus::Stable)),
        }
    }

    /// Control+Alt, which the game leaves alone, plus a letter that says what it does.
    pub fn defaults() -> BTreeMap<Action, String> {
        [
            (Action::PasteScan, "control+alt+KeyV"),
            (Action::DeleteMissing, "control+alt+KeyD"),
            (Action::ToggleOverlay, "control+alt+KeyO"),
            (Action::SetAlias, "control+alt+KeyA"),
            (Action::CopyBookmark, "control+alt+KeyB"),
            (Action::MassReduced, "control+alt+KeyR"),
            (Action::MassCritical, "control+alt+KeyC"),
            (Action::Eol, "control+alt+KeyE"),
        ]
        .into_iter()
        .map(|(a, k)| (a, k.to_string()))
        .collect()
    }
}

/// What the digit keys that pick a jump's signature are held with. They are only taken
/// while a jump is waiting for an answer, so even bare digits leave the game alone otherwise.
pub const PICK_MODIFIERS: [(&str, &str); 4] = [
    ("control+alt", "Ctrl+Alt+1"),
    ("alt", "Alt+1"),
    ("control+shift", "Ctrl+Shift+1"),
    ("", "1"),
];

/// The pick key for `digit` held with `modifiers`, in `global-hotkey`'s notation.
fn pick_key(modifiers: &str, digit: u8) -> String {
    if modifiers.is_empty() {
        format!("Digit{digit}")
    } else {
        format!("{modifiers}+Digit{digit}")
    }
}

/// How the pick keys read, without the digit: `Ctrl+Alt+`, or nothing for bare digits.
pub fn pick_prefix(modifiers: &str) -> String {
    let shown = display(&pick_key(modifiers, 1));
    shown.strip_suffix('1').unwrap_or(&shown).to_string()
}

/// A shortcut that went off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    Action(Action),
    /// A digit while a jump waits for its signature: 1 to 9 pick, 0 says none of them.
    Pick(u8),
}

/// A key press in the app's window as a shortcut, or `None` when it cannot be one: a bare
/// modifier, or a key without Control, Alt or the platform key (F-keys excepted).
pub fn from_keystroke(keystroke: &gpui_kit::Keystroke) -> Option<String> {
    let m = &keystroke.modifiers;
    let key = keystroke.key.to_lowercase();
    let code = match key.as_str() {
        k if k.len() == 1 && k.chars().all(|c| c.is_ascii_alphabetic()) => {
            format!("Key{}", k.to_uppercase())
        }
        k if k.len() == 1 && k.chars().all(|c| c.is_ascii_digit()) => format!("Digit{k}"),
        k if k.starts_with('f') && k[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) => {
            k.to_uppercase()
        }
        "space" => "Space".into(),
        "up" => "ArrowUp".into(),
        "down" => "ArrowDown".into(),
        "left" => "ArrowLeft".into(),
        "right" => "ArrowRight".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "pageup" => "PageUp".into(),
        "pagedown" => "PageDown".into(),
        "insert" => "Insert".into(),
        "delete" => "Delete".into(),
        "-" => "Minus".into(),
        "=" => "Equal".into(),
        "[" => "BracketLeft".into(),
        "]" => "BracketRight".into(),
        ";" => "Semicolon".into(),
        "'" => "Quote".into(),
        "," => "Comma".into(),
        "." => "Period".into(),
        "/" => "Slash".into(),
        "\\" => "Backslash".into(),
        "`" => "Backquote".into(),
        _ => return None,
    };
    let function_key = code.starts_with('F') && code.len() <= 3;
    if !(m.control || m.alt || m.platform || function_key) {
        return None;
    }
    let mut parts = Vec::new();
    if m.control {
        parts.push("control".to_string());
    }
    if m.alt {
        parts.push("alt".into());
    }
    if m.shift {
        parts.push("shift".into());
    }
    if m.platform {
        parts.push("super".into());
    }
    parts.push(code);
    let text = parts.join("+");
    HotKey::from_str(&text).ok().map(|_| text)
}

/// `control+alt+KeyR` as people write it: `Ctrl+Alt+R`.
pub fn display(hotkey: &str) -> String {
    hotkey
        .split('+')
        .map(|part| match part.to_lowercase().as_str() {
            "control" | "ctrl" => "Ctrl".to_string(),
            "alt" | "option" => if cfg!(target_os = "macos") {
                "Option"
            } else {
                "Alt"
            }
            .into(),
            "shift" => "Shift".into(),
            "super" | "meta" | "command" | "cmd" => if cfg!(target_os = "macos") {
                "Cmd"
            } else {
                "Super"
            }
            .into(),
            _ => part
                .strip_prefix("Key")
                .or_else(|| part.strip_prefix("Digit"))
                .or_else(|| part.strip_prefix("Arrow"))
                .unwrap_or(part)
                .to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// The shortcuts registered with the system right now.
pub struct Registry {
    manager: Option<GlobalHotKeyManager>,
    bound: Vec<(HotKey, Action)>,
    picks: Vec<(HotKey, u8)>,
    /// Why shortcuts are unavailable, when they are.
    pub problem: Option<String>,
}

impl Registry {
    /// Must run on the main thread, which is where GPUI calls the app from.
    pub fn new() -> Registry {
        match GlobalHotKeyManager::new() {
            Ok(manager) => Registry {
                manager: Some(manager),
                bound: Vec::new(),
                picks: Vec::new(),
                problem: None,
            },
            Err(err) => Registry {
                manager: None,
                bound: Vec::new(),
                picks: Vec::new(),
                problem: Some(format!("Shortcuts are unavailable here: {err}")),
            },
        }
    }

    /// Replace what is registered with `bindings`. Returns the ones the system refused,
    /// usually because another program holds them.
    pub fn apply(&mut self, bindings: &BTreeMap<Action, String>) -> Vec<(Action, String)> {
        let Some(manager) = &self.manager else {
            return Vec::new();
        };
        for (hotkey, _) in self.bound.drain(..) {
            let _ = manager.unregister(hotkey);
        }
        let mut refused = Vec::new();
        for (&action, text) in bindings {
            // Cleared on purpose: kept as empty so the default does not come back.
            if text.is_empty() {
                continue;
            }
            let Ok(hotkey) = HotKey::from_str(text) else {
                refused.push((action, format!("{text} is not a shortcut")));
                continue;
            };
            match manager.register(hotkey) {
                Ok(()) => self.bound.push((hotkey, action)),
                Err(err) => refused.push((action, err.to_string())),
            }
        }
        refused
    }

    /// Take the digit keys held with `modifiers` while a jump waits for its signature, or
    /// give them back with `None`. Returns why the system refused, if it did.
    pub fn set_picks(&mut self, modifiers: Option<&str>) -> Option<String> {
        let manager = self.manager.as_ref()?;
        for (hotkey, _) in self.picks.drain(..) {
            let _ = manager.unregister(hotkey);
        }
        let modifiers = modifiers?;
        let mut refused = None;
        for digit in 0..=9 {
            let text = pick_key(modifiers, digit);
            let Ok(hotkey) = HotKey::from_str(&text) else {
                return Some(format!("{text} is not a shortcut"));
            };
            // One already bound to an action keeps doing that.
            if self.bound.iter().any(|(h, _)| h.id() == hotkey.id()) {
                continue;
            }
            match manager.register(hotkey) {
                Ok(()) => self.picks.push((hotkey, digit)),
                Err(err) => refused = Some(format!("{}: {err}", display(&text))),
            }
        }
        refused
    }

    /// The shortcuts pressed since the last call.
    pub fn pressed(&self) -> Vec<Press> {
        let mut out = Vec::new();
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state() != HotKeyState::Pressed {
                continue;
            }
            if let Some((_, action)) = self.bound.iter().find(|(h, _)| h.id() == event.id()) {
                out.push(Press::Action(*action));
            } else if let Some((_, digit)) = self.picks.iter().find(|(h, _)| h.id() == event.id()) {
                out.push(Press::Pick(*digit));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_parse_and_read_well() {
        for text in Action::defaults().values() {
            assert!(HotKey::from_str(text).is_ok(), "{text}");
        }
        assert_eq!(
            display("control+alt+KeyR"),
            if cfg!(target_os = "macos") {
                "Ctrl+Option+R"
            } else {
                "Ctrl+Alt+R"
            }
        );
        assert_eq!(display("shift+F5"), "Shift+F5");
    }

    #[test]
    fn pick_keys_parse_and_read_well() {
        for (modifiers, label) in PICK_MODIFIERS {
            for digit in 0..=9 {
                assert!(HotKey::from_str(&pick_key(modifiers, digit)).is_ok());
            }
            if !cfg!(target_os = "macos") {
                assert_eq!(format!("{}1", pick_prefix(modifiers)), label);
            }
        }
    }
}
