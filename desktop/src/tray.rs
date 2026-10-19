//! The tray icon. With it the app keeps running when its window is closed, overlay and keys
//! included, and quits from the tray's menu. Linux speaks StatusNotifierItem over D-Bus
//! (Waybar, KDE, GNOME with the AppIndicator extension); with no tray listening there is
//! none, and closing the window quits.

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// What the tray asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Open,
    ToggleOverlay,
    Quit,
}

pub struct Tray {
    icon: TrayIcon,
    dark: bool,
    open: MenuId,
    overlay: MenuId,
    quit: MenuId,
}

impl Tray {
    /// Put the icon up, or `None` where there is no tray to put it in.
    pub fn new() -> Option<Tray> {
        if !host_available() {
            return None;
        }
        let open = MenuItem::new("Open WormholeSystems", true, None);
        let overlay = MenuItem::new("Show or hide the overlay", true, None);
        let quit = MenuItem::new("Quit", true, None);
        let menu = Menu::new();
        menu.append_items(&[&open, &overlay, &PredefinedMenuItem::separator(), &quit])
            .ok()?;
        let builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("WormholeSystems");
        // The menu bar tints a template icon to fit, light or dark.
        #[cfg(target_os = "macos")]
        let builder = builder.with_icon_templated(icon(false)?);
        #[cfg(not(target_os = "macos"))]
        let builder = builder.with_icon(icon(false)?);
        let icon = builder
            // A click opens the window; the menu is on the right button. The menu bar
            // has no right button to speak of, so there a click is the menu.
            .with_menu_on_left_click(cfg!(target_os = "macos"))
            .build()
            .ok()?;
        Some(Tray {
            icon,
            dark: false,
            open: open.id().clone(),
            overlay: overlay.id().clone(),
            quit: quit.id().clone(),
        })
    }

    /// Draw the logo white for a dark theme, black for a light one. The menu bar tints its
    /// template itself.
    pub fn set_dark(&mut self, dark: bool) {
        if dark == self.dark || cfg!(target_os = "macos") {
            return;
        }
        self.dark = dark;
        if let Some(icon) = icon(dark) {
            let _ = self.icon.set_icon(Some(icon));
        }
    }

    /// What happened since the last call.
    pub fn events(&self) -> Vec<TrayEvent> {
        let mut out = Vec::new();
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.open {
                out.push(TrayEvent::Open);
            } else if event.id == self.overlay {
                out.push(TrayEvent::ToggleOverlay);
            } else if event.id == self.quit {
                out.push(TrayEvent::Quit);
            }
        }
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            let open = match event {
                TrayIconEvent::Click { button, .. } => {
                    button == MouseButton::Left && !cfg!(target_os = "macos")
                }
                TrayIconEvent::DoubleClick { .. } => true,
                _ => false,
            };
            if open {
                out.push(TrayEvent::Open);
            }
        }
        out
    }
}

/// The logo alone, white or black.
fn icon(dark: bool) -> Option<Icon> {
    use resvg::tiny_skia::{Pixmap, Transform};
    use resvg::usvg::{Options, Tree};

    const SIZE: u32 = 64;
    let fill = if dark { "white" } else { "black" };
    let svg = std::str::from_utf8(include_bytes!("../assets/logo.svg"))
        .ok()?
        .replace("fill=\"black\"", &format!("fill=\"{fill}\""));
    let tree = Tree::from_str(&svg, &Options::default()).ok()?;
    let (w, h) = (tree.size().width(), tree.size().height());
    let scale = SIZE as f32 / w.max(h);
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    let transform = Transform::from_row(
        scale,
        0.0,
        0.0,
        scale,
        (SIZE as f32 - w * scale) / 2.0,
        (SIZE as f32 - h * scale) / 2.0,
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Icon::from_rgba(rgba, SIZE, SIZE).ok()
}

/// Whether a StatusNotifier host is there to show the icon. The tray library assumes one
/// is, and without it the icon would be nowhere and the app unreachable once closed.
#[cfg(target_os = "linux")]
fn host_available() -> bool {
    let Ok(bus) = zbus::blocking::Connection::session() else {
        return false;
    };
    bus.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "NameHasOwner",
        &("org.kde.StatusNotifierWatcher",),
    )
    .and_then(|reply| reply.body().deserialize::<bool>())
    .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn host_available() -> bool {
    true
}
