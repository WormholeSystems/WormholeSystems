//! The signatures in a panel that stays above the game, built like the tournament tool's
//! overlays: one transparent surface over the whole screen with the panel drawn inside it,
//! so moving and resizing it is only drawing it somewhere else. Drag it by its title, size
//! it by the grip in its corner; it snaps to the screen edges and remembers where it went.
//!
//! On Wayland the surface is a layer-shell overlay whose input region is just the panel, so
//! every other click reaches the game. Windows and macOS have no input region: the window
//! lets clicks through and takes them only while the cursor is over the panel (see
//! `native`). Elsewhere (X11, or a compositor without layer shell) it is a pop-up window
//! the size of the panel, where it was last put. It never takes the keyboard.
//!
//! Opened with GPUI's own `open_window` rather than the kit's: the kit's root adds a
//! client-side shadow inset, which would make the surface bigger than the screen.

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::config::Overlay;
#[cfg(not(target_os = "linux"))]
use crate::native;
use crate::sigs::{self, Command, Sigs};
use crate::snap;

/// Smallest the panel can be resized to.
const MIN_W: f32 = 260.0;
const MIN_H: f32 = 140.0;
/// Keep this much of it on the screen while dragging.
const KEEP: f32 = 40.0;

/// What kind of window the overlay lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Surface {
    /// A transparent window over the whole screen, the panel drawn inside it.
    Screen,
    /// A window the size of the panel, which cannot follow a drag.
    Panel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragMode {
    Move,
    /// From the bottom-right grip.
    Resize,
}

#[derive(Debug, Clone)]
struct Drag {
    mode: DragMode,
    mouse_start: Point<Pixels>,
    start: snap::Rect,
    current: snap::Rect,
    guides: Vec<snap::Guide>,
}

pub struct OverlayView {
    sigs: Entity<Sigs>,
    surface: Surface,
    drag: Option<Drag>,
    /// The panel's bounds at the last paint, for the input region.
    measured: Rc<RefCell<Option<Bounds<Pixels>>>>,
    last_region: Option<Vec<Bounds<Pixels>>>,
    /// Whether the native window lets clicks through right now.
    #[cfg(not(target_os = "linux"))]
    click_through: bool,
    /// Follows the cursor while the window lets clicks through; see `sync_native_input`.
    #[cfg(not(target_os = "linux"))]
    cursor_poll: Option<Task<()>>,
    _observe: Subscription,
}

impl OverlayView {
    fn new(sigs: Entity<Sigs>, surface: Surface, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&sigs, |_, _, cx| cx.notify());
        OverlayView {
            sigs,
            surface,
            drag: None,
            measured: Rc::default(),
            last_region: None,
            #[cfg(not(target_os = "linux"))]
            click_through: true,
            #[cfg(not(target_os = "linux"))]
            cursor_poll: None,
            _observe: observe,
        }
    }

    /// Where the panel is drawn: following a drag, else where the settings put it.
    fn rect(&self, settings: &Overlay, screen: (f32, f32)) -> snap::Rect {
        if let Some(drag) = &self.drag {
            return drag.current;
        }
        let (x, y) = settings.origin(screen);
        let (w, h) = settings.dimensions();
        snap::Rect { x, y, w, h }
    }

    fn start_drag(&mut self, mode: DragMode, mouse: Point<Pixels>, start: snap::Rect) {
        self.drag = Some(Drag {
            mode,
            mouse_start: mouse,
            start,
            current: start,
            guides: Vec::new(),
        });
    }

    /// Follow the pointer, then snap to the screen edges.
    fn update_drag(&mut self, mouse: Point<Pixels>, screen: (f32, f32)) {
        let Some(drag) = &mut self.drag else { return };
        let dx = f32::from(mouse.x - drag.mouse_start.x);
        let dy = f32::from(mouse.y - drag.mouse_start.y);
        let s = drag.start;
        let (rect, guides) = match drag.mode {
            DragMode::Move => snap::snap_move(
                snap::Rect {
                    x: (s.x + dx).clamp(KEEP - s.w, (screen.0 - KEEP).max(0.0)),
                    y: (s.y + dy).clamp(0.0, (screen.1 - KEEP).max(0.0)),
                    ..s
                },
                &[],
                screen,
            ),
            DragMode::Resize => snap::snap_resize(
                snap::Rect {
                    w: (s.w + dx).clamp(MIN_W, (screen.0 - s.x).max(MIN_W)),
                    h: (s.h + dy).clamp(MIN_H, (screen.1 - s.y).max(MIN_H)),
                    ..s
                },
                &[],
                screen,
            ),
        };
        drag.current = rect;
        drag.guides = guides;
    }

    /// Where a drag ended goes to the app, which keeps it in the settings.
    fn finish_drag(&mut self, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.take() else { return };
        let r = drag.current;
        let command = Command::PlaceOverlay {
            x: r.x.round(),
            y: r.y.round(),
            width: r.w.round(),
            height: r.h.round(),
        };
        self.sigs.update(cx, |sigs, cx| {
            // Straight away, so the next frame does not jump back while the app saves.
            sigs.overlay.x = Some(r.x.round());
            sigs.overlay.y = Some(r.y.round());
            if drag.mode == DragMode::Resize {
                sigs.overlay.width = Some(r.w.round());
                sigs.overlay.height = Some(r.h.round());
            }
            cx.emit(command);
            cx.notify();
        });
    }

    /// Input region: the panel, or the whole surface while dragging so the pointer cannot
    /// slip off it.
    fn update_input_region(&mut self, popup: Option<Bounds<Pixels>>, window: &mut Window) {
        let region = if self.drag.is_some() {
            None
        } else {
            // The panel, and a picker's popup where it hangs past the panel's edge.
            Some(
                self.measured
                    .borrow()
                    .iter()
                    .copied()
                    .chain(popup)
                    .collect::<Vec<_>>(),
            )
        };
        if region != self.last_region {
            window.set_input_region(region.as_deref());
            self.last_region = region;
        }
    }

    /// Windows and macOS have no input region; a window takes all input or none. Take it
    /// while dragging or while the cursor is over the panel, and poll the cursor meanwhile,
    /// since a click-through window hears nothing of it.
    #[cfg(not(target_os = "linux"))]
    fn sync_native_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Read the panel's bounds as last painted, not as the last render saw them: the
        // canvas measures them while painting, after the render, and nothing renders again
        // until something changes.
        let input = self.drag.is_some()
            || native::cursor_position(window).is_some_and(|p| {
                let sigs = self.sigs.read(cx);
                let popup = sigs
                    .picker
                    .filter(|picker| picker.overlay)
                    .and_then(|_| *sigs.popup.borrow());
                let panel = if sigs.overlay_hidden {
                    None
                } else {
                    *self.measured.borrow()
                };
                panel.into_iter().chain(popup).any(|r| r.contains(&p))
            });
        if input == self.click_through {
            native::set_click_through(window, !input);
            self.click_through = !input;
        }
        if self.cursor_poll.is_none() {
            self.cursor_poll = Some(cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(16))
                        .await;
                    if this.update_in(cx, Self::sync_native_input).is_err() {
                        break;
                    }
                }
            }));
        }
    }
}

/// Bottom-right resize grip: three dots in a corner triangle.
fn resize_grip(color: Hsla) -> Stateful<Div> {
    let dot = || div().absolute().size(px(2.0)).bg(color);
    div()
        .id("overlay-resize")
        .absolute()
        .right_0()
        .bottom_0()
        .size(px(14.0))
        .cursor_nwse_resize()
        .child(dot().right(px(3.0)).bottom(px(3.0)))
        .child(dot().right(px(7.0)).bottom(px(3.0)))
        .child(dot().right(px(3.0)).bottom(px(7.0)))
}

/// A snap guide across the whole screen.
fn guide_line(g: snap::Guide, screen: Size<Pixels>, color: Hsla) -> AnyElement {
    let line = div().absolute().bg(color);
    match g {
        snap::Guide::Vertical(x) => line.left(px(x)).top_0().w(px(1.0)).h(screen.height),
        snap::Guide::Horizontal(y) => line.top(px(y)).left_0().h(px(1.0)).w(screen.width),
    }
    .into_any_element()
}

impl Render for OverlayView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let settings = self.sigs.read(cx).overlay.clone();
        let opacity = settings.opacity as f32 / 100.0;
        // Its own window, outside the kit's root, so the text size is set here.
        window.set_rem_size(px(crate::ui::scaled_font(settings.text_size)));
        let chrome = |d: Div| {
            d.bg(theme.background.opacity(opacity))
                .border_1()
                .border_color(theme.border)
                .text_color(theme.foreground)
                .font_family(crate::theme::UI)
        };

        if self.surface == Surface::Panel {
            return chrome(div().size_full())
                .child(sigs::panel(&self.sigs, true, cx))
                .into_any_element();
        }

        let viewport = window.viewport_size();
        if viewport.width <= px(1.0) {
            return div().into_any_element();
        }
        if self.sigs.read(cx).overlay_hidden {
            // Nothing drawn and nothing taking clicks, until focus comes back.
            *self.measured.borrow_mut() = None;
            self.drag = None;
            self.update_input_region(None, window);
            #[cfg(not(target_os = "linux"))]
            self.sync_native_input(window, cx);
            return div().into_any_element();
        }
        let screen = (f32::from(viewport.width), f32::from(viewport.height));
        let r = self.rect(&settings, screen);

        let this = cx.entity().downgrade();
        let grab: sigs::Grab = Box::new(move |event, _, cx| {
            this.update(cx, |this, cx| {
                this.start_drag(DragMode::Move, event.position, r);
                cx.notify();
            })
            .ok();
        });
        let measured = Rc::clone(&self.measured);
        let panel = chrome(div())
            .id("overlay-panel")
            .absolute()
            .left(px(r.x))
            .top(px(r.y))
            .w(px(r.w))
            .h(px(r.h))
            .when(self.drag.is_some(), |d| d.border_color(theme.primary))
            .child(crate::ui::top_line(&theme))
            .child(sigs::panel_with(&self.sigs, true, Some(grab), cx))
            .child(
                canvas(
                    move |bounds, _, _| *measured.borrow_mut() = Some(bounds),
                    |_, (), _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .child(resize_grip(theme.muted_foreground).on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    this.start_drag(DragMode::Resize, event.position, r);
                    cx.notify();
                }),
            ));

        let popup = {
            let sigs = self.sigs.read(cx);
            sigs.picker
                .filter(|p| p.overlay)
                .and_then(|_| *sigs.popup.borrow())
        };
        self.update_input_region(popup, window);
        #[cfg(not(target_os = "linux"))]
        self.sync_native_input(window, cx);
        let guides: Vec<AnyElement> = self
            .drag
            .as_ref()
            .map(|d| {
                d.guides
                    .iter()
                    .map(|g| guide_line(*g, viewport, theme.primary.opacity(0.6)))
                    .collect()
            })
            .unwrap_or_default();

        div()
            .size_full()
            .relative()
            .when(self.drag.is_some(), |d| {
                d.on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                    this.update_drag(event.position, screen);
                    cx.notify();
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseUpEvent, _, cx| {
                        this.finish_drag(cx);
                        cx.notify();
                    }),
                )
            })
            .children(guides)
            .child(panel)
            .into_any_element()
    }
}

/// Open the overlay as `settings` describe it.
pub fn open(
    sigs: Entity<Sigs>,
    settings: &Overlay,
    cx: &mut App,
) -> anyhow::Result<AnyWindowHandle> {
    let display = cx.primary_display();
    let screen = display.as_ref().map(|d| d.bounds()).unwrap_or_else(|| {
        Bounds::new(
            point(px(0.0), px(0.0)),
            gpui_kit::size(px(1920.0), px(1080.0)),
        )
    });
    let build = |sigs: Entity<Sigs>, surface: Surface| {
        move |_: &mut Window, cx: &mut App| cx.new(|cx| OverlayView::new(sigs, surface, cx))
    };

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            use gpui_kit::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
            let options = WindowOptions {
                titlebar: None,
                // Anchored to every edge: the compositor makes it the size of the screen.
                window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                    point(px(0.0), px(0.0)),
                    size(px(0.0), px(0.0)),
                ))),
                app_id: Some("wormholesystems-overlay".into()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "wormholesystems-overlay".into(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                focus: false,
                ..Default::default()
            };
            if let Ok(handle) = cx.open_window(options, build(sigs.clone(), Surface::Screen)) {
                return Ok(handle.into());
            }
            // A compositor without layer shell (GNOME) gets the pop-up below.
        }
        let screen_size = (f32::from(screen.size.width), f32::from(screen.size.height));
        let (x, y) = settings.origin(screen_size);
        let (w, h) = settings.dimensions();
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                point(screen.origin.x + px(x), screen.origin.y + px(y)),
                size(px(w), px(h)),
            ))),
            titlebar: None,
            focus: false,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            app_id: Some("wormholesystems-overlay".into()),
            ..Default::default()
        };
        let handle = cx.open_window(options, build(sigs, Surface::Panel))?;
        Ok(handle.into())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = settings;
        let options = WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(screen)),
            display_id: display.map(|d| d.id()),
            app_id: Some("wormholesystems-overlay".into()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::PopUp,
            focus: false,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            ..Default::default()
        };
        let handle = cx.open_window(options, |window, cx| {
            native::set_click_through(window, true);
            build(sigs, Surface::Screen)(window, cx)
        })?;
        Ok(handle.into())
    }
}
