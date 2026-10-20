//! The look the main window and the overlay share, after EVE's Photon UI: near-black
//! frames with a hairline border and a thin accent line along the top, quiet mixed-case
//! titles, flat square buttons with a light outline (the main one tinted with the accent),
//! and a short notice that slides in and fades out.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::Theme;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::{self, HEADING};

/// The base font size the sizes here are written against.
pub const BASE_FONT: f32 = 16.0;

/// The base font size at `percent`, kept to sizes the layout holds up at.
pub fn scaled_font(percent: u8) -> f32 {
    BASE_FONT * percent.clamp(75, 150) as f32 / 100.0
}

/// `value` pixels at the base font size, growing and shrinking with the window's font size
/// setting like the text does.
pub fn rpx(value: f32) -> Rems {
    rems(value / BASE_FONT)
}

/// What a button is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The one thing to do here: tinted with the accent, like EVE's "Place Buy Order".
    Primary,
    Normal,
    /// The chosen one of a row of choices: an accent underline, like a Photon tab.
    Selected,
    Danger,
}

/// A flat button with a hairline outline. The caller adds `on_click`; a disabled one is
/// dimmed and takes no clicks.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    kind: Kind,
    enabled: bool,
    theme: &Theme,
) -> Stateful<Div> {
    frame(id, kind, enabled, theme).child(label.into())
}

/// A button for a character: their portrait, then their name. Until the portrait is in,
/// a dim square holds its place.
pub fn character(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    portrait: Option<Arc<Image>>,
    kind: Kind,
    theme: &Theme,
) -> Stateful<Div> {
    let face = rpx(20.0);
    let picture = match portrait {
        Some(portrait) => img(portrait).size(face).rounded(px(1.0)).into_any_element(),
        None => div()
            .size(face)
            .rounded(px(1.0))
            .bg(theme.foreground.opacity(0.08))
            .into_any_element(),
    };
    frame(id, kind, true, theme)
        .pl(rpx(3.0))
        .gap(rpx(6.0))
        .child(picture)
        .child(name.into())
}

fn frame(id: impl Into<ElementId>, kind: Kind, enabled: bool, theme: &Theme) -> Stateful<Div> {
    let accent = theme.primary;
    let white = theme.foreground;
    let (text, border, fill) = match kind {
        Kind::Primary => (white, accent.opacity(0.75), accent.opacity(0.2)),
        Kind::Selected => (white, white.opacity(0.22), white.opacity(0.09)),
        Kind::Normal => (
            white.opacity(0.85),
            white.opacity(0.16),
            white.opacity(0.035),
        ),
        Kind::Danger => (
            theme.danger,
            theme.danger.opacity(0.45),
            theme.danger.opacity(0.08),
        ),
    };
    let hover = match kind {
        Kind::Danger => theme.danger,
        Kind::Primary => accent,
        _ => white,
    };
    div()
        .id(id.into())
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .h(rpx(26.0))
        .px(rpx(12.0))
        .rounded(px(2.0))
        .border_1()
        .border_color(border)
        .bg(fill)
        .font_family(HEADING)
        .text_size(rpx(13.0))
        .text_color(text)
        .when(kind == Kind::Selected, |d| {
            d.child(
                div()
                    .absolute()
                    .left(px(6.0))
                    .right(px(6.0))
                    .bottom(px(-1.0))
                    .h(px(2.0))
                    .bg(accent),
            )
        })
        .map(|d| {
            if enabled {
                d.cursor_pointer()
                    .hover(move |s| s.border_color(hover.opacity(0.6)).bg(hover.opacity(0.12)))
                    .active(move |s| s.bg(hover.opacity(0.22)))
            } else {
                d.opacity(0.4)
            }
        })
}

/// A framed section, like a Photon window: a quiet title, then the children, padded.
pub fn section(title: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .flex_none()
        .relative()
        .flex()
        .flex_col()
        .gap_3()
        .px_4()
        .pt(rpx(10.0))
        .pb_4()
        .border_1()
        .border_color(theme.border)
        .bg(theme.foreground.opacity(0.02))
        .child(top_line(theme))
        .child(title_text(title, theme))
}

/// The thin accent line along a frame's top edge.
pub fn top_line(theme: &Theme) -> Div {
    div()
        .absolute()
        .top(px(-1.0))
        .left(px(-1.0))
        .right(px(-1.0))
        .h(px(1.0))
        .bg(linear_gradient(
            90.0,
            linear_color_stop(theme.primary.opacity(0.7), 0.0),
            linear_color_stop(theme.primary.opacity(0.1), 1.0),
        ))
}

/// A window title the way Photon writes it: small, mixed case, muted.
pub fn title_text(title: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .font_family(HEADING)
        .text_size(rpx(13.0))
        .text_color(theme.muted_foreground)
        .child(title.into())
}

/// How a notice reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Info,
    Done,
    Problem,
}

/// A short line about what just happened, shown in a header for a few seconds.
#[derive(Debug, Clone)]
pub struct Notice {
    pub mood: Mood,
    pub text: String,
    /// Tells one notice from the next, so a timer only clears its own.
    pub id: u64,
    /// Fading out.
    pub leaving: bool,
}

impl Notice {
    /// How long it stays before fading: a problem gets time to be read.
    pub fn lasts(&self) -> Duration {
        match self.mood {
            Mood::Problem => Duration::from_secs(7),
            _ => Duration::from_millis(2600),
        }
    }
}

pub const FADE: Duration = Duration::from_millis(260);

/// The notice: an icon and the line, sliding in from above and fading out when it leaves.
pub fn notice(notice: &Notice, theme: &Theme) -> AnyElement {
    let (icon, color) = match notice.mood {
        Mood::Done => ("icons/circle-check.svg", theme::added()),
        Mood::Info => ("icons/info.svg", theme.primary),
        Mood::Problem => ("icons/circle-alert.svg", theme.danger),
    };
    let chip = div()
        .flex()
        .items_center()
        .gap(rpx(6.0))
        .min_w_0()
        .h(rpx(22.0))
        .px(rpx(10.0))
        .rounded_full()
        .border_1()
        .border_color(color.opacity(0.45))
        .bg(color.opacity(0.12))
        .text_xs()
        .child(
            svg()
                .flex_none()
                .path(icon)
                .size(rpx(13.0))
                .text_color(color),
        )
        .child(div().truncate().child(notice.text.clone()));
    let easing = ease_out_quint();
    if notice.leaving {
        chip.with_animation(
            ElementId::NamedInteger("notice-out".into(), notice.id),
            Animation::new(FADE),
            |d, t| d.opacity(1.0 - t),
        )
        .into_any_element()
    } else {
        chip.with_animation(
            ElementId::NamedInteger("notice-in".into(), notice.id),
            Animation::new(Duration::from_millis(240)).with_easing(easing),
            |d, t| d.opacity(t).relative().top(px((1.0 - t) * -8.0)),
        )
        .into_any_element()
    }
}
