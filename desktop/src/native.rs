//! Native tweaks for the overlay window on Windows and macOS, where there is no layer
//! shell: the window must stay above EVE, never take focus from it, and let clicks through
//! to the game. Input is all or nothing per window, so the overlay polls the cursor
//! ([`cursor_position`]) and takes input only while it is over the panel. From the
//! tournament tool's overlays.
//!
//! The only `unsafe` code in the app: plain calls into the window system on the window
//! GPUI created.

#![allow(unsafe_code)]

use gpui_kit::{Pixels, Point, Window, point, px};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// Clicks pass through the whole window to the game (`true`), or the window
/// takes input (cursor over a box or the pencil, dragging).
pub fn set_click_through(window: &Window, on: bool) {
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    apply(handle.as_raw(), on);
}

/// The cursor in the window's coordinates, wherever it is on screen (a
/// click-through window gets no mouse events to track it with).
pub fn cursor_position(window: &Window) -> Option<Point<Pixels>> {
    let handle = HasWindowHandle::window_handle(window).ok()?;
    cursor(handle.as_raw(), window.scale_factor())
}

#[cfg(windows)]
#[allow(clippy::cast_possible_wrap)] // style bits are small flags
fn apply(handle: RawWindowHandle, click_through: bool) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };

    let RawWindowHandle::Win32(h) = handle else {
        return;
    };
    let hwnd = h.hwnd.get() as HWND;
    // SAFETY: `hwnd` is the live window GPUI just gave us a handle to; these
    // calls only change its extended style and z-order.
    unsafe {
        let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        // layered (needed for click-through), no taskbar button, never
        // activated by a click: EVE keeps the keyboard
        style |= (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize;
        if click_through {
            style |= WS_EX_TRANSPARENT as isize;
        } else {
            style &= !(WS_EX_TRANSPARENT as isize);
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
        // a layered window needs its opacity set once, or it stays invisible
        SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

#[cfg(target_os = "macos")]
fn apply(handle: RawWindowHandle, click_through: bool) {
    use objc2_app_kit::{NSView, NSWindowCollectionBehavior};

    let RawWindowHandle::AppKit(h) = handle else {
        return;
    };
    // SAFETY: `ns_view` is the content view of the live window GPUI created,
    // and GPUI renders (and calls us) on the main thread.
    let view: &NSView = unsafe { h.ns_view.cast::<NSView>().as_ref() };
    let Some(window) = view.window() else {
        return;
    };
    window.setIgnoresMouseEvents(click_through);
    // on every Space, over full-screen apps (EVE in its own Space), not in
    // the window cycle, and stays put when the app is in the background
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
    window.setHidesOnDeactivate(false);
}

#[cfg(windows)]
#[allow(clippy::cast_precision_loss)] // screen coordinates are small
fn cursor(handle: RawWindowHandle, scale: f32) -> Option<Point<Pixels>> {
    use windows_sys::Win32::Foundation::{HWND, POINT};
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let RawWindowHandle::Win32(h) = handle else {
        return None;
    };
    let hwnd = h.hwnd.get() as HWND;
    let mut p = POINT { x: 0, y: 0 };
    // SAFETY: `hwnd` is the live window GPUI gave us a handle to and `p` a
    // local the calls write into.
    let ok = unsafe { GetCursorPos(&raw mut p) != 0 && ScreenToClient(hwnd, &raw mut p) != 0 };
    // physical pixels (GPUI is per-monitor DPI aware) to logical
    ok.then(|| point(px(p.x as f32 / scale), px(p.y as f32 / scale)))
}

#[cfg(target_os = "macos")]
#[allow(clippy::cast_possible_truncation)] // points fit in f32
fn cursor(handle: RawWindowHandle, _scale: f32) -> Option<Point<Pixels>> {
    use objc2_app_kit::{NSEvent, NSView};

    let RawWindowHandle::AppKit(h) = handle else {
        return None;
    };
    // SAFETY: as in `apply`
    let view: &NSView = unsafe { h.ns_view.cast::<NSView>().as_ref() };
    let frame = view.window()?.frame();
    // screen points, origin bottom-left; the borderless window's content
    // fills its frame, and GPUI's origin is top-left
    let m = NSEvent::mouseLocation();
    Some(point(
        px((m.x - frame.origin.x) as f32),
        px((frame.origin.y + frame.size.height - m.y) as f32),
    ))
}
