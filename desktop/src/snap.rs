//! Snapping for moving and resizing the overlay: edges snap to the screen edges, and to
//! other boxes' edges (flush, or a small gap apart for stacking), within a few pixels.
//! Plain `f32` geometry so it is easy to test. From the tournament tool's overlays.

/// How close (px) an edge has to come before it snaps.
pub const THRESHOLD: f32 = 10.;
/// Gap between boxes snapped next to each other.
pub const GAP: f32 = 8.;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// A line to draw while an edge is snapped to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Guide {
    Vertical(f32),
    Horizontal(f32),
}

/// A snap target on one axis: where the edge goes, and where the guide line
/// is drawn (the other box's edge, also for gap snaps).
#[derive(Debug, Clone, Copy)]
struct Target {
    at: f32,
    guide: f32,
}

/// Targets for the start (left / top) and end (right / bottom) edge of a box
/// on one axis, given the other boxes' spans on that axis.
fn targets(spans: &[(f32, f32)], screen: f32) -> (Vec<Target>, Vec<Target>) {
    let mut start = vec![Target { at: 0., guide: 0. }];
    let mut end = vec![Target {
        at: screen,
        guide: screen,
    }];
    for &(s, e) in spans {
        start.extend([
            Target { at: s, guide: s },
            Target { at: e, guide: e },
            Target {
                at: e + GAP,
                guide: e,
            },
        ]);
        end.extend([
            Target { at: e, guide: e },
            Target { at: s, guide: s },
            Target {
                at: s - GAP,
                guide: s,
            },
        ]);
    }
    (start, end)
}

/// The closest target within [`THRESHOLD`]: `(shift, guide)`.
fn nearest(edge: f32, targets: &[Target]) -> Option<(f32, f32)> {
    targets
        .iter()
        .map(|t| (t.at - edge, t.guide))
        .filter(|(d, _)| d.abs() <= THRESHOLD)
        .min_by(|a, b| a.0.abs().total_cmp(&b.0.abs()))
}

/// Snap a moving box: whichever of its two edges is closer to a target wins.
fn snap_span(start: f32, len: f32, targets: &(Vec<Target>, Vec<Target>)) -> Option<(f32, f32)> {
    let a = nearest(start, &targets.0);
    let b = nearest(start + len, &targets.1);
    match (a, b) {
        (Some(a), Some(b)) => Some(if a.0.abs() <= b.0.abs() { a } else { b }),
        (a, b) => a.or(b),
    }
}

/// `(start, end)` of each box on one axis.
type Spans = Vec<(f32, f32)>;

fn spans(others: &[Rect]) -> (Spans, Spans) {
    (
        others.iter().map(|o| (o.x, o.x + o.w)).collect(),
        others.iter().map(|o| (o.y, o.y + o.h)).collect(),
    )
}

/// Snap a box being moved; returns the snapped box and the active guides.
pub fn snap_move(r: Rect, others: &[Rect], screen: (f32, f32)) -> (Rect, Vec<Guide>) {
    let (xs, ys) = spans(others);
    let mut out = r;
    let mut guides = Vec::new();
    if let Some((shift, at)) = snap_span(r.x, r.w, &targets(&xs, screen.0)) {
        out.x += shift;
        guides.push(Guide::Vertical(at));
    }
    if let Some((shift, at)) = snap_span(r.y, r.h, &targets(&ys, screen.1)) {
        out.y += shift;
        guides.push(Guide::Horizontal(at));
    }
    (out, guides)
}

/// Snap a box being resized from its bottom-right corner.
pub fn snap_resize(r: Rect, others: &[Rect], screen: (f32, f32)) -> (Rect, Vec<Guide>) {
    let (xs, ys) = spans(others);
    let mut out = r;
    let mut guides = Vec::new();
    if let Some((shift, at)) = nearest(r.x + r.w, &targets(&xs, screen.0).1) {
        out.w += shift;
        guides.push(Guide::Vertical(at));
    }
    if let Some((shift, at)) = nearest(r.y + r.h, &targets(&ys, screen.1).1) {
        out.h += shift;
        guides.push(Guide::Horizontal(at));
    }
    (out, guides)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f32, f32) = (1920., 1080.);

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn aligns_left_edges() {
        let other = rect(100., 100., 200., 100.);
        let (r, guides) = snap_move(rect(106., 400., 150., 80.), &[other], SCREEN);
        assert_eq!(r.x, 100.);
        assert_eq!(guides, vec![Guide::Vertical(100.)]);
    }

    #[test]
    fn stacks_below_with_gap() {
        let other = rect(100., 100., 200., 100.);
        // top edge near the other's bottom (200) + gap
        let (r, guides) = snap_move(rect(300., 205., 150., 80.), &[other], SCREEN);
        assert_eq!(r.y, 200. + GAP);
        assert!(guides.contains(&Guide::Horizontal(200.)));
    }

    #[test]
    fn snaps_to_screen_edges() {
        let (r, _) = snap_move(rect(1915. - 150., 4., 150., 80.), &[], SCREEN);
        assert_eq!((r.x + r.w, r.y), (1920., 0.));
    }

    #[test]
    fn leaves_far_boxes_alone() {
        let other = rect(100., 100., 200., 100.);
        let r = rect(600., 600., 150., 80.);
        assert_eq!(snap_move(r, &[other], SCREEN), (r, vec![]));
    }

    #[test]
    fn resize_matches_neighbour_width() {
        let other = rect(100., 100., 200., 100.);
        let (r, guides) = snap_resize(rect(100., 300., 195., 80.), &[other], SCREEN);
        assert_eq!(r.w, 200.);
        assert_eq!(guides, vec![Guide::Vertical(300.)]);
    }
}
