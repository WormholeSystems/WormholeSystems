//! The little window a shortcut opens to name the system you are in. Unlike the overlay it
//! takes the keyboard, since it is there to be typed into: Enter saves, Esc closes. On
//! Wayland it is a layer-shell surface with the keyboard to itself, so it shows up above the
//! game; elsewhere it is a focused pop-up window.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use crate::theme::HEADING;

const WIDTH: f32 = 360.0;
const HEIGHT: f32 = 132.0;

/// What the dialog was closed with: the alias typed, or `None` for Esc.
pub type Done = Box<dyn Fn(Option<String>, &mut App)>;

pub struct AliasDialog {
    system: String,
    input: Entity<InputState>,
    done: Option<Done>,
    _subscription: Subscription,
}

impl AliasDialog {
    fn new(
        system: String,
        current: String,
        done: Done,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Alias, blank to clear")
                .default_value(current)
        });
        input.update(cx, |input, cx| {
            input.focus(window, cx);
            // Typing replaces what is there; arrows keep it.
            input.select_all(window, cx);
        });
        let subscription = cx.subscribe_in(&input, window, |this, input, event, window, cx| {
            if let InputEvent::PressEnter { .. } = event {
                let value = input.read(cx).value().trim().to_string();
                this.close(Some(value), window, cx);
            }
        });
        AliasDialog {
            system,
            input,
            done: Some(done),
            _subscription: subscription,
        }
    }

    fn close(&mut self, value: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(done) = self.done.take() {
            done(value, cx);
        }
        window.remove_window();
    }
}

impl Render for AliasDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(theme.background)
            .border_1()
            .border_color(theme.primary.opacity(0.6))
            .text_color(theme.foreground)
            .font_family(crate::theme::UI)
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    cx.stop_propagation();
                    this.close(None, window, cx);
                }
            }))
            .child(
                div()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_sm()
                    .child(format!("Name {}", self.system)),
            )
            .child(Input::new(&self.input))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Enter saves, Esc closes."),
            )
    }
}

/// Open the dialog for `system`, its alias filled in. `done` hears how it was closed.
pub fn open(
    system: String,
    current: String,
    done: Done,
    cx: &mut App,
) -> anyhow::Result<AnyWindowHandle> {
    // Grown with the text size set for the main window.
    let scale = f32::from(cx.theme().font_size) / crate::ui::BASE_FONT;
    let size = size(px(WIDTH * scale), px(HEIGHT * scale));
    let display = cx.primary_display().map(|d| d.bounds()).unwrap_or_else(|| {
        Bounds::new(
            point(px(0.0), px(0.0)),
            gpui_kit::size(px(1920.0), px(1080.0)),
        )
    });
    let options = |kind| WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered_at(
            display.center(),
            size,
        ))),
        titlebar: None,
        focus: true,
        kind,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        app_id: Some("wormholesystems-alias".into()),
        ..Default::default()
    };
    // Only one of the two windows below is ever built, but each wants the parts for itself.
    let parts = std::rc::Rc::new(std::cell::RefCell::new(Some((system, current, done))));
    let build = |parts: std::rc::Rc<std::cell::RefCell<Option<(String, String, Done)>>>| {
        move |window: &mut Window, cx: &mut App| {
            let (system, current, done) = parts.borrow_mut().take().expect("built once");
            cx.new(|cx| AliasDialog::new(system, current, done, window, cx))
        }
    };

    #[cfg(target_os = "linux")]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        use gpui_kit::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
        let layer = WindowKind::LayerShell(LayerShellOptions {
            namespace: "wormholesystems-alias".into(),
            layer: Layer::Overlay,
            // No edge: the compositor centres it.
            anchor: Anchor::empty(),
            exclusive_zone: None,
            exclusive_edge: None,
            margin: None,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
        });
        if let Ok((handle, _)) = gpui_kit::open_window(options(layer), cx, build(parts.clone())) {
            return Ok(handle);
        }
    }

    let (handle, _) = gpui_kit::open_window(options(WindowKind::PopUp), cx, build(parts))?;
    Ok(handle)
}
