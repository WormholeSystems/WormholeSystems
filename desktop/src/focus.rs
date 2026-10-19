//! Which window has the keyboard: an EVE client (and whose), this app, or something else.
//! The overlay can hide while you are elsewhere. Hyprland says so on its event socket and
//! Windows can be asked; anywhere else the answer is [`Focus::Unknown`] and the overlay
//! stays up.

#[cfg(any(target_os = "linux", target_os = "windows"))]
use std::sync::mpsc::Sender;
use std::sync::mpsc::{Receiver, channel};

/// What has focus.
// macOS has no backend yet, so only `Unknown` is ever made there.
#[cfg_attr(target_os = "macos", allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Focus {
    /// An EVE client, with the character logged in on it; `None` on the login screens.
    Eve(Option<String>),
    /// One of this app's windows.
    Us,
    Other,
    /// No way to tell on this desktop.
    Unknown,
}

impl Focus {
    /// Read a window's class (Hyprland's, empty elsewhere) and title.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub fn of(class: &str, title: &str) -> Focus {
        if title == "WormholeSystems" || class.starts_with("wormholesystems") {
            return Focus::Us;
        }
        if let Some(name) = title.strip_prefix("EVE - ") {
            return Focus::Eve(Some(name.trim().to_string()));
        }
        if class == "steam_app_8500" || title == "EVE" || title == "EVE Launcher" {
            return Focus::Eve(None);
        }
        Focus::Other
    }
}

/// Whether this desktop can say which window has focus.
pub fn supported() -> bool {
    cfg!(target_os = "windows")
        || (cfg!(target_os = "linux") && std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some())
}

impl Focus {
    /// Whether the overlay belongs on screen with this focus, for these characters (empty
    /// for any).
    pub fn shows_overlay(&self, characters: &[String]) -> bool {
        match self {
            Focus::Us | Focus::Unknown => true,
            Focus::Other => false,
            Focus::Eve(Some(name)) => characters.is_empty() || characters.contains(name),
            Focus::Eve(None) => characters.is_empty(),
        }
    }
}

/// Start watching. The receiver hears every change; nothing at all where focus can't be
/// told, which reads as [`Focus::Unknown`].
pub fn watch() -> Receiver<Focus> {
    let (tx, rx) = channel();
    #[cfg(target_os = "linux")]
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        std::thread::spawn(move || hyprland(tx));
        return rx;
    }
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || windows(tx));
        return rx;
    }
    #[allow(unreachable_code)]
    {
        drop(tx);
        rx
    }
}

#[cfg(target_os = "linux")]
fn hyprland(tx: Sender<Focus>) {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::net::UnixStream;

    let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR")
        .zip(std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE"))
        .map(|(run, sig)| std::path::PathBuf::from(run).join("hypr").join(sig))
    else {
        return;
    };
    // What has focus right now, before any event says.
    if let Ok(mut s) = UnixStream::connect(dir.join(".socket.sock")) {
        let mut reply = String::new();
        if s.write_all(b"j/activewindow").is_ok()
            && s.read_to_string(&mut reply).is_ok()
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&reply)
        {
            let field = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
            let _ = tx.send(Focus::of(&field("class"), &field("title")));
        }
    }
    // Reconnect if Hyprland drops the socket, say on a reload.
    loop {
        let Ok(stream) = UnixStream::connect(dir.join(".socket2.sock")) else {
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        };
        for line in BufReader::new(stream).lines() {
            let Ok(line) = line else { break };
            // `activewindow>>class,title`; the title may hold commas, the class never does.
            let Some(rest) = line.strip_prefix("activewindow>>") else {
                continue;
            };
            let (class, title) = rest.split_once(',').unwrap_or((rest, ""));
            // A layer surface, like the alias dialog, taking the keyboard reads as nothing;
            // keep what was there.
            if class.is_empty() && title.is_empty() {
                continue;
            }
            if tx.send(Focus::of(class, title)).is_err() {
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[cfg(target_os = "windows")]
fn windows(tx: Sender<Focus>) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    let mut last = None;
    loop {
        let (title, pid) = unsafe {
            let hwnd = GetForegroundWindow();
            let mut buf = [0u16; 256];
            let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (String::from_utf16_lossy(&buf[..n.max(0) as usize]), pid)
        };
        // A click activates the overlay, which has no title: it is still us.
        let focus = if pid == std::process::id() {
            Focus::Us
        } else {
            Focus::of("", &title)
        };
        if last.as_ref() != Some(&focus) {
            if tx.send(focus.clone()).is_err() {
                return;
            }
            last = Some(focus);
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_windows() {
        assert_eq!(
            Focus::of("steam_app_8500", "EVE - Tania Kion"),
            Focus::Eve(Some("Tania Kion".into()))
        );
        assert_eq!(Focus::of("steam_app_8500", ""), Focus::Eve(None));
        assert_eq!(
            Focus::of("", "EVE - Nicolas Kion"),
            Focus::Eve(Some("Nicolas Kion".into()))
        );
        assert_eq!(Focus::of("", "WormholeSystems"), Focus::Us);
        assert_eq!(Focus::of("vesktop", "Discord"), Focus::Other);
    }
}
