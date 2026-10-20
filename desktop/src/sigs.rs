//! The signatures in the system you are in, what the last paste did to them, and after a
//! jump, which signature it was. One entity, shown by the main window and the overlay alike;
//! their buttons emit [`Command`]s, which the app carries out.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use gpui_kit::component::{ActiveTheme as _, Theme, ThemeStyled as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::api::{
    Connection, MapSnapshot, MassStatus, PlacedSystem, Signature, SignatureCatalog, SignatureEdit,
    SignatureType, TimeStatus,
};
use crate::config::{ListView, Overlay, SortColumn};
use crate::theme::{self, HEADING};
use crate::ui::{self, Kind, Notice, rpx};

/// What a button in a signatures panel asks the app to do.
#[derive(Debug, Clone)]
pub enum Command {
    Paste,
    DeleteMissing,
    Delete(i64),
    ClearReview,
    HideOverlay,
    /// The list's filter or sort changed, to keep.
    SaveList,
    /// Change a signature: its category, type, or the connection it is.
    Edit(i64, SignatureEdit),
    /// The jump was the option at this index; `None` for a hole nobody had scanned.
    Pick(Option<usize>),
    /// Copy the bookmark for where this signature's hole goes.
    CopyBookmark(i64),
    /// Leave the jump off the map.
    SkipJump,
    /// The overlay was dragged or resized to this: top-left corner and size.
    PlaceOverlay {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

/// The system a jump landed in.
#[derive(Debug, Clone)]
pub struct Target {
    pub solar_system_id: i64,
    pub name: String,
    pub class: Option<i64>,
    pub security: f64,
}

/// A jump waiting to be told which of the origin's signatures it went through.
#[derive(Debug, Clone)]
pub struct Ask {
    pub character: String,
    pub map_id: i64,
    pub origin: PlacedSystem,
    pub target: Target,
    /// Likely ones first, then the unlikely; picked by position, numbered from 1.
    pub options: Vec<Signature>,
    pub likely: usize,
    /// Holes off the origin nobody has flown, connection to ghost placement, with their
    /// names. Flying one names the ghost rather than placing the system twice.
    pub ghosts: HashMap<i64, i64>,
    pub ghost_aliases: HashMap<i64, String>,
    /// What the new system will be called, if anything.
    pub alias: Option<String>,
    /// Where it goes on the map.
    pub at: (f64, f64),
}

/// Which of a row's cells has its choices open under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picker {
    pub sig: i64,
    pub kind: PickerKind,
    /// Opened in the overlay rather than the main window; only that one shows it.
    pub overlay: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Category,
    Type,
    Connection,
    /// The row's menu: the hole's mass and lifetime, preserve mass, delete.
    Menu,
}

/// How a row names its type: the wormhole code and where it leads, else the site.
pub struct TypeLabel {
    pub text: String,
    pub dest: Option<(&'static str, Hsla)>,
    pub extra: Option<String>,
    /// Nothing picked yet, or only the scanner's raw name.
    pub muted: bool,
}

/// The far end of one of the system's connections, as a picker lists it.
pub struct Far {
    pub connection: i64,
    pub class: (&'static str, Hsla),
    pub alias: Option<String>,
    /// `None` for a hole nobody has been through.
    pub name: Option<String>,
    pub region: Option<String>,
}

/// Where you are: the character who moved last, by the chat logs.
#[derive(Debug, Clone, PartialEq)]
pub struct Here {
    pub character: String,
    pub system: String,
}

/// What a paste did, kept until the next paste, a sweep, or leaving the system. Only rows
/// there when the scan was pasted can be missing from it: one added since is news the scan
/// never saw.
#[derive(Debug, Clone)]
pub struct Review {
    pub solar_system_id: i64,
    pub before: HashSet<String>,
    pub pasted: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteStatus {
    New,
    Updated,
    Missing,
}

#[derive(Clone)]
pub enum Tone {
    Info,
    Done,
    Problem,
}

#[derive(Default)]
pub struct Sigs {
    pub here: Option<Here>,
    pub snapshot: Option<Arc<MapSnapshot>>,
    pub catalog: Arc<SignatureCatalog>,
    pub review: Option<Review>,
    pub busy: bool,
    /// The outcome of the last thing done here, shown in the header for a moment.
    pub notice: Option<Notice>,
    pub overlay: Overlay,
    pub list: ListView,
    pub ask: Option<Ask>,
    /// How the pick keys read before the digit, `Ctrl+Alt+`.
    pub pick_prefix: String,
    pub picker: Option<Picker>,
    /// Where the open picker's cell is, and where its popup landed, both in window
    /// coordinates: the popup hangs off the one, and the overlay takes clicks on the other.
    pub anchor: Rc<RefCell<Option<Bounds<Pixels>>>>,
    pub popup: Rc<RefCell<Option<Bounds<Pixels>>>>,
    /// List the system's statics first when picking a wormhole's type.
    pub statics_first: bool,
    /// The overlay is out of the way: EVE (or the right character) is not in front.
    pub overlay_hidden: bool,
}

impl EventEmitter<Command> for Sigs {}

impl Sigs {
    /// The system you are in, when it is on the map.
    pub fn system(&self) -> Option<&PlacedSystem> {
        let here = self.here.as_ref()?;
        self.snapshot.as_ref()?.system_named(&here.system)
    }

    /// The signatures in the system you are in, by scanner id.
    pub fn rows(&self) -> Vec<&Signature> {
        let (Some(system), Some(snapshot)) = (self.system(), &self.snapshot) else {
            return Vec::new();
        };
        let mut rows: Vec<&Signature> = snapshot
            .signatures
            .iter()
            .filter(|s| s.solar_system_id == system.solar_system_id)
            .collect();
        rows.sort_by(|a, b| a.signature_id.cmp(&b.signature_id));
        rows
    }

    /// What the list shows: [`Sigs::rows`] without the hidden categories, sorted the way the
    /// website's panel sorts, ties by scanner id so rows never swap on a refresh.
    pub fn shown(&self) -> Vec<&Signature> {
        let mut rows: Vec<&Signature> = self
            .rows()
            .into_iter()
            .filter(|s| !self.list.hidden.contains(&s.group))
            .collect();
        let name = |s: &Signature| {
            self.type_of(s)
                .map(|t| t.name.clone())
                .or_else(|| s.name.clone())
        };
        // A missing value sorts last going up, first going down, like the website.
        fn nullable<T: Ord>(a: Option<T>, b: Option<T>) -> std::cmp::Ordering {
            match (a, b) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        }
        let since = |s: &Signature| {
            if s.group == "wormhole" {
                s.created_at
            } else {
                s.updated_at
            }
        };
        let column = self.list.sort;
        let descending = self.list.descending;
        rows.sort_by(|a, b| {
            let order = match column {
                SortColumn::Id => a.signature_id.cmp(&b.signature_id),
                SortColumn::Category => nullable(
                    Some(&a.group).filter(|g| *g != "unknown"),
                    Some(&b.group).filter(|g| *g != "unknown"),
                ),
                SortColumn::Type => nullable(name(a), name(b)),
                // Newest first going up.
                SortColumn::Age => since(b).cmp(&since(a)),
            };
            let order = if descending { order.reverse() } else { order };
            order.then_with(|| a.signature_id.cmp(&b.signature_id))
        });
        rows
    }

    /// The review, when it is about the system you are in.
    fn current_review(&self) -> Option<&Review> {
        let system = self.system()?;
        self.review
            .as_ref()
            .filter(|r| r.solar_system_id == system.solar_system_id)
    }

    pub fn paste_status(&self, sig: &Signature) -> Option<PasteStatus> {
        let review = self.current_review()?;
        let before = review.before.contains(&sig.signature_id);
        if !review.pasted.contains(&sig.signature_id) {
            return before.then_some(PasteStatus::Missing);
        }
        Some(if before {
            PasteStatus::Updated
        } else {
            PasteStatus::New
        })
    }

    /// Row keys of what the last paste no longer saw.
    pub fn missing(&self) -> Vec<i64> {
        self.rows()
            .into_iter()
            .filter(|s| self.paste_status(s) == Some(PasteStatus::Missing))
            .map(|s| s.id)
            .collect()
    }

    fn type_of(&self, sig: &Signature) -> Option<&SignatureType> {
        let id = sig.signature_type_id?;
        self.catalog.types.iter().find(|t| t.id == id)
    }

    /// What a signature is, the way the website's row says it.
    pub fn type_label(&self, sig: &Signature) -> TypeLabel {
        match self.type_of(sig) {
            Some(t) if sig.group == "wormhole" && t.signature.is_some() => TypeLabel {
                text: t.signature.clone().unwrap_or_default(),
                dest: Some(theme::dest_class_meta(t.target_class)),
                extra: t.extra.clone(),
                muted: false,
            },
            Some(t) => TypeLabel {
                text: t.name.clone(),
                dest: None,
                extra: None,
                muted: false,
            },
            None => TypeLabel {
                text: sig.name.clone().unwrap_or_else(|| "Type".into()),
                dest: None,
                extra: None,
                muted: true,
            },
        }
    }

    /// The far end of a connection off the system you are in.
    fn far(&self, connection: i64) -> Option<Far> {
        let snapshot = self.snapshot.as_ref()?;
        let here = self.system()?;
        let c = snapshot.connection(connection)?;
        let other = if c.from_system == here.id {
            c.to_system
        } else {
            c.from_system
        };
        let alias = |a: &Option<String>| a.clone().filter(|a| !a.trim().is_empty());
        if let Some(s) = snapshot.systems.iter().find(|s| s.id == other) {
            return Some(Far {
                connection,
                class: theme::class_meta(s.wormhole_class_id, Some(s.security_status)),
                alias: alias(&s.alias),
                name: Some(s.name.clone()),
                region: Some(s.region.clone()).filter(|r| !r.is_empty()),
            });
        }
        let ghost = snapshot.ghosts.iter().find(|g| g.id == other)?;
        Some(Far {
            connection,
            class: theme::class_meta(None, None),
            alias: alias(&ghost.alias),
            name: None,
            region: None,
        })
    }

    /// The class of the system a signature's connection leads to, once someone has been.
    fn linked_class(&self, sig: &Signature) -> Option<i64> {
        let snapshot = self.snapshot.as_ref()?;
        let far = self.far(sig.connection_id?)?;
        let name = far.name?;
        snapshot.system_named(&name)?.wormhole_class_id
    }

    /// The types a signature can be, grouped the way the website's picker lists them:
    /// the category's types that spawn in this class, narrowed to the far end's class once
    /// linked. For wormholes, statics (when wanted), then K162, then the rest.
    pub fn type_sections(
        &self,
        sig: &Signature,
    ) -> Vec<(Option<&'static str>, Vec<&SignatureType>)> {
        let Some(&(_, category, _)) = theme::CATEGORIES.iter().find(|c| c.0 == sig.group) else {
            return Vec::new();
        };
        let class = self.system().and_then(|s| s.wormhole_class_id);
        let linked = self.linked_class(sig);
        let mut types: Vec<&SignatureType> = self
            .catalog
            .types
            .iter()
            .filter(|t| t.signature_category_id == category)
            .filter(|t| class.is_none_or(|c| t.spawn_areas.contains(&c)))
            .filter(|t| linked.is_none() || t.target_class == linked)
            .collect();
        types.sort_by_key(|t| theme::dest_sort_weight(t.target_class));
        if sig.group != "wormhole" {
            return vec![(None, types)];
        }
        let statics: HashSet<&str> = match self.system() {
            Some(s) if self.statics_first => s.statics.iter().map(|s| s.code.as_str()).collect(),
            _ => HashSet::new(),
        };
        let code = |t: &SignatureType| t.signature.clone().unwrap_or_default();
        let (first, rest): (Vec<_>, Vec<_>) = types
            .into_iter()
            .partition(|t| statics.contains(code(t).as_str()));
        let (k162, rest): (Vec<_>, Vec<_>) = rest.into_iter().partition(|t| code(t) == "K162");
        [("Statics", first), ("K162", k162), ("Wormholes", rest)]
            .into_iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(label, v)| (Some(label), v))
            .collect()
    }

    /// The connections a wormhole signature could be: the system's, narrowed to the type's
    /// destination class once it has one, aliased ones first. Split into the ones no other
    /// signature here claims, and the ones another does.
    pub fn link_options(&self, sig: &Signature) -> (Vec<Far>, Vec<Far>) {
        let (Some(snapshot), Some(here)) = (&self.snapshot, self.system()) else {
            return (Vec::new(), Vec::new());
        };
        let class = self.type_of(sig).and_then(|t| t.target_class);
        let mut options: Vec<Far> = snapshot
            .connections
            .iter()
            .filter(|c| c.from_system == here.id || c.to_system == here.id)
            .filter_map(|c| self.far(c.id))
            .filter(|f| {
                class.is_none()
                    || f.name
                        .as_ref()
                        .and_then(|n| snapshot.system_named(n))
                        .is_some_and(|s| s.wormhole_class_id == class)
            })
            .collect();
        options.sort_by(|a, b| match (&a.alias, &b.alias) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        });
        let claimed: HashSet<i64> = snapshot
            .signatures
            .iter()
            .filter(|s| s.solar_system_id == here.solar_system_id && s.id != sig.id)
            .filter_map(|s| s.connection_id)
            .collect();
        options
            .into_iter()
            .partition(|f| !claimed.contains(&f.connection))
    }
}

/// `now`, `47m`, `3h`, `2d`: how old a row is, like the website's age column. Wormholes
/// count from when they were found (the signature or its connection, whichever is older),
/// sites from their last update.
fn age(sig: &Signature, connection: Option<&Connection>, now: DateTime<Utc>) -> String {
    let since = if sig.group == "wormhole" {
        connection
            .and_then(|c| c.created_at)
            .map_or(sig.created_at, |c| c.min(sig.created_at))
    } else {
        sig.updated_at
    };
    let minutes = (now - since).num_minutes();
    match minutes {
        m if m < 1 => "now".into(),
        m if m < 60 => format!("{m}m"),
        m if m < 24 * 60 => format!("{}h", m / 60),
        m => format!("{}d", m / (24 * 60)),
    }
}

/// The age's color: the hole's mass first, then its lifetime, where the signature's own
/// lifetime wins over the connection's unless it is healthy.
fn age_color(sig: &Signature, connection: Option<&Connection>) -> Option<Hsla> {
    let mass = connection.and_then(|c| c.mass_status).or(sig.mass_status);
    let time = match sig.time_status {
        Some(t) if t != TimeStatus::Stable => Some(t),
        _ => connection.and_then(|c| c.time_status).or(sig.time_status),
    };
    match (mass, time) {
        (Some(MassStatus::Critical), _) => Some(theme::critical()),
        (Some(MassStatus::Reduced), _) => Some(theme::reduced()),
        (_, Some(TimeStatus::Critical | TimeStatus::Expired)) => Some(theme::critical()),
        (_, Some(TimeStatus::Eol)) => Some(theme::eol()),
        _ => None,
    }
}

fn emit(
    entity: &Entity<Sigs>,
    command: Command,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + use<> {
    let entity = entity.clone();
    move |_, _, cx| {
        let command = command.clone();
        entity.update(cx, |_, cx| cx.emit(command));
    }
}

/// The signatures panel. `compact` is the overlay's version: a close button in the header,
/// and less room around everything.
pub fn panel(entity: &Entity<Sigs>, compact: bool, cx: &App) -> AnyElement {
    panel_with(entity, compact, None, cx)
}

/// What pressing on the panel's title does: the overlay moves by it.
pub type Grab = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// [`panel`], with `grab` making the title a handle to drag the overlay by.
pub fn panel_with(
    entity: &Entity<Sigs>,
    compact: bool,
    grab: Option<Grab>,
    cx: &App,
) -> AnyElement {
    let theme = cx.theme().clone();
    let sigs = entity.read(cx);
    let now = Utc::now();
    let pad = if compact { rpx(8.0) } else { rpx(16.0) };

    // Header: the system, its class, and who is there.
    let mut title = div().flex().items_center().gap_2().min_w_0();
    let sub;
    match (&sigs.here, sigs.system()) {
        (None, _) => {
            title = title.child(
                div()
                    .text_color(theme.muted_foreground)
                    .child("No location"),
            );
            sub = "Log in a character; the chat logs say where you are.".into();
        }
        (Some(here), None) => {
            title = title.child(div().child(here.system.clone()));
            sub = if sigs.snapshot.is_some() {
                let map = sigs
                    .snapshot
                    .as_ref()
                    .map_or("the map".into(), |s| s.map.name.clone());
                format!("{} · not on {map}", here.character)
            } else {
                here.character.clone()
            };
        }
        (Some(here), Some(system)) => {
            let (class, color) =
                theme::class_meta(system.wormhole_class_id, Some(system.security_status));
            title = title
                .child(div().flex_none().text_color(color).child(class))
                .when_some(
                    system.alias.clone().filter(|a| !a.trim().is_empty()),
                    |d, alias| d.child(div().flex_none().child(alias)),
                )
                .child(
                    div()
                        .truncate()
                        .text_color(theme.muted_foreground)
                        .child(system.name.clone()),
                );
            let count = sigs.rows().len();
            let hidden = count - sigs.shown().len();
            let hidden = if hidden > 0 {
                format!(", {hidden} hidden")
            } else {
                String::new()
            };
            let map = sigs
                .snapshot
                .as_ref()
                .map(|s| format!("{} · ", s.map.name))
                .unwrap_or_default();
            sub = format!(
                "{map}{} · {count} signature{}{hidden}",
                here.character,
                if count == 1 { "" } else { "s" }
            );
        }
    }
    let header = div()
        .flex_none()
        .flex()
        .items_start()
        .gap_2()
        .px(pad)
        .py_2()
        .border_b_1()
        .border_color(theme.border)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(rpx(2.0))
                .when_some(grab, |d, grab| {
                    d.cursor_move()
                        .on_mouse_down(MouseButton::Left, move |e, w, cx| grab(e, w, cx))
                })
                .child(
                    title
                        .font_family(HEADING)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(if compact { rpx(13.0) } else { rpx(16.0) }),
                )
                .child(
                    div()
                        .text_xs()
                        .truncate()
                        .text_color(theme.muted_foreground)
                        .child(sub),
                ),
        )
        .when_some(sigs.notice.as_ref(), |d, n| {
            d.child(
                div()
                    .flex_none()
                    .max_w(rpx(260.0))
                    .child(ui::notice(n, &theme)),
            )
        })
        .when(compact, |d| {
            d.child(
                div()
                    .id("overlay-hide")
                    .flex_none()
                    .p_1()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.muted))
                    .on_click(emit(entity, Command::HideOverlay))
                    .child(
                        svg()
                            .path("icons/x.svg")
                            .size(rpx(12.0))
                            .text_color(theme.muted_foreground),
                    ),
            )
        });

    // The rows.
    let rows = sigs.shown();
    let mut list = div().flex().flex_col();
    let all = sigs.rows().len();
    if rows.is_empty() && all > 0 {
        list = list.child(
            div()
                .px(pad)
                .py_3()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("{all} hidden by the filters.")),
        );
    } else if rows.is_empty() && sigs.system().is_some() {
        list = list.child(
            div()
                .px(pad)
                .py_3()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("No signatures yet. Copy the scan in the probe scanner and paste it."),
        );
    }
    let mono = theme.mono_font_family.clone();
    let mut shown = false;
    for sig in rows {
        let status = sigs.paste_status(sig);
        let (group, group_color) = theme::group_meta(&sig.group);
        let label = sigs.type_label(sig);
        let wormhole = sig.group == "wormhole";
        let connection = sig
            .connection_id
            .and_then(|c| sigs.snapshot.as_ref()?.connection(c));
        let far = sig.connection_id.and_then(|c| sigs.far(c));
        let mark = match status {
            Some(PasteStatus::New) => Some(theme::added()),
            Some(PasteStatus::Updated) => Some(theme::refreshed()),
            Some(PasteStatus::Missing) => Some(theme::missing()),
            None => None,
        };
        let missing = status == Some(PasteStatus::Missing);
        let key = sig.id;
        let open = sigs
            .picker
            .filter(|p| p.sig == key && p.overlay == compact)
            .map(|p| p.kind);
        let cell = |kind: PickerKind| {
            div()
                .id(ElementId::Name(format!("cell-{key}-{kind:?}").into()))
                .h(rpx(20.0))
                .px_1()
                .rounded(px(3.0))
                .flex()
                .items_center()
                .gap_1()
                .min_w_0()
                .overflow_hidden()
                .cursor_pointer()
                .border_1()
                .border_color(if open == Some(kind) {
                    theme.primary.opacity(0.7)
                } else {
                    theme.transparent
                })
                .hover(|s| s.bg(theme.muted).border_color(theme.input))
                .relative()
                .on_mouse_down(
                    MouseButton::Left,
                    toggle(
                        entity,
                        Picker {
                            sig: key,
                            kind,
                            overlay: compact,
                        },
                    ),
                )
                .when(open == Some(kind), |d| d.child(probe(&sigs.anchor)))
        };
        // The open picker's popup, built once and handed to the cell it hangs off.
        let float_el = RefCell::new(open.map(|kind| {
            let content = picker(entity, sigs, sig, connection, kind, &theme);
            let picker = Picker {
                sig: key,
                kind,
                overlay: compact,
            };
            (kind, float(entity, sigs, picker, content, cx))
        }));
        let float_for = |kind: PickerKind| {
            let mut slot = float_el.borrow_mut();
            match slot.take() {
                Some((k, el)) if k == kind => Some(el),
                other => {
                    *slot = other;
                    None
                }
            }
        };
        let category = cell(PickerKind::Category)
            .w(rpx(72.0))
            .flex_none()
            .children(float_for(PickerKind::Category))
            .map(|d| {
                if sig.group == "unknown" || group_color.is_none() {
                    d.child(div().text_color(theme.muted_foreground).child("Category"))
                } else {
                    let color = group_color.unwrap_or(theme.muted_foreground);
                    d.child(
                        svg()
                            .flex_none()
                            .path(theme::group_icon(&sig.group))
                            .size(rpx(12.0))
                            .text_color(color),
                    )
                    .child(div().truncate().child(group))
                }
            });
        let type_cell = cell(PickerKind::Type)
            .children(float_for(PickerKind::Type))
            // A wormhole's type is a code and a class, `K162 C3`: it needs little room, and
            // the connection beside it gets the rest.
            .when(wormhole, |d| d.w(rpx(84.0)).flex_none())
            .when(!wormhole, |d| d.flex_1())
            .map(|d| match label.dest {
                Some((short, color)) => d
                    .font_family(mono.clone())
                    .child(div().flex_none().child(label.text))
                    .child(div().flex_none().text_color(color).child(short))
                    .when_some(label.extra, |d, extra| {
                        d.child(
                            div()
                                .truncate()
                                .text_color(theme.muted_foreground)
                                .child(format!("({extra})")),
                        )
                    }),
                None => d.child(
                    div()
                        .truncate()
                        .when(label.muted, |d| d.text_color(theme.muted_foreground))
                        .child(label.text),
                ),
            });
        let icon_button = |id: &str, path: &'static str| {
            div()
                .id(ElementId::Name(format!("{id}-{key}").into()))
                .flex_none()
                .size(rpx(20.0))
                .rounded(px(3.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(theme.muted))
                .child(
                    svg()
                        .path(path)
                        .size(rpx(13.0))
                        .text_color(theme.muted_foreground),
                )
        };
        list = list.child(
            div()
                .id(ElementId::NamedInteger("sig".into(), key as u64))
                .relative()
                .flex()
                .items_center()
                .gap_2()
                .px(pad)
                .h(rpx(26.0))
                .text_xs()
                .border_b_1()
                .border_color(theme.border.opacity(0.3))
                .hover(|s| s.bg(theme.muted.opacity(0.3)))
                .when_some(mark, |d, color| d.bg(color.opacity(0.1)))
                .when(missing, |d| d.opacity(0.6))
                .child(
                    // The paste's mark down the left edge.
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(2.0))
                        .when_some(mark, |d, color| d.bg(color)),
                )
                .child(
                    div()
                        .w(rpx(56.0))
                        .flex_none()
                        .font_family(mono.clone())
                        .when(missing, |d| d.line_through())
                        .child(if sig.signature_id.is_empty() {
                            "---".to_string()
                        } else {
                            sig.signature_id.clone()
                        }),
                )
                .child(category)
                .child(type_cell)
                .when(wormhole, |d| {
                    d.child(
                        cell(PickerKind::Connection)
                            .flex_1()
                            .children(float_for(PickerKind::Connection))
                            .map(|d| match far {
                                Some(far) => far_label(d, far, None, &theme),
                                None => d.child(
                                    div()
                                        .truncate()
                                        .text_color(theme.muted_foreground)
                                        .child("Connection"),
                                ),
                            }),
                    )
                })
                .child(
                    div()
                        .w(rpx(32.0))
                        .flex_none()
                        .flex()
                        .justify_end()
                        .font_family(mono.clone())
                        .text_color(age_color(sig, connection).unwrap_or(theme.muted_foreground))
                        .child(age(sig, connection, now)),
                )
                .child(
                    div()
                        .w(rpx(44.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(rpx(2.0))
                        .when(wormhole, |d| {
                            d.child(
                                icon_button("sig-copy", "icons/copy.svg")
                                    .on_click(emit(entity, Command::CopyBookmark(key))),
                            )
                        })
                        .child(
                            icon_button("sig-menu", "icons/ellipsis-vertical.svg")
                                .relative()
                                .when(open == Some(PickerKind::Menu), |d| {
                                    d.bg(theme.muted).child(probe(&sigs.anchor))
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    toggle(
                                        entity,
                                        Picker {
                                            sig: key,
                                            kind: PickerKind::Menu,
                                            overlay: compact,
                                        },
                                    ),
                                )
                                .children(float_for(PickerKind::Menu)),
                        ),
                ),
        );
        shown |= open.is_some();
    }
    let mine = sigs.picker.is_some_and(|p| p.overlay == compact);
    if sigs.picker.is_none() || (mine && !shown) {
        *sigs.anchor.borrow_mut() = None;
        *sigs.popup.borrow_mut() = None;
    }

    let ask = sigs
        .ask
        .as_ref()
        .map(|ask| ask_section(entity, sigs, ask, pad, &theme));

    // Footer: the last outcome and what can be done next.
    let missing = sigs.missing().len();
    let can_paste = sigs.system().is_some() && !sigs.busy;
    let footer = div()
        .flex_none()
        .flex()
        .flex_col()
        .gap_2()
        .px(pad)
        .py_2()
        .border_t_1()
        .border_color(theme.border);
    let footer = footer.child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .child(
                ui::button(
                    "sig-paste",
                    if sigs.busy {
                        "Working…"
                    } else {
                        "Paste scan"
                    },
                    Kind::Primary,
                    can_paste,
                    &theme,
                )
                .when(can_paste, |d| d.on_click(emit(entity, Command::Paste))),
            )
            .when(missing > 0, |d| {
                d.child(
                    ui::button(
                        "sig-sweep",
                        format!("Delete {missing} missing"),
                        Kind::Danger,
                        !sigs.busy,
                        &theme,
                    )
                    .when(!sigs.busy, |d| {
                        d.on_click(emit(entity, Command::DeleteMissing))
                    }),
                )
            })
            .when(sigs.current_review().is_some(), |d| {
                d.child(
                    ui::button("sig-clear", "Clear marks", Kind::Normal, true, &theme)
                        .on_click(emit(entity, Command::ClearReview)),
                )
            })
            .child(filters(entity, &sigs.list, &theme)),
    );

    let columns = sigs
        .system()
        .is_some()
        .then(|| column_header(entity, &sigs.list, pad, &theme));

    div()
        .size_full()
        .flex()
        .flex_col()
        .child(header)
        .children(ask)
        .children(columns)
        .child(
            div()
                .id(if compact { "overlay-rows" } else { "page-rows" })
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(list),
        )
        .child(footer)
        .into_any_element()
}

/// Change the list's filter or sort, and keep it.
fn relist(
    entity: &Entity<Sigs>,
    change: impl Fn(&mut ListView) + 'static,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |_, _, cx| {
        entity.update(cx, |sigs, cx| {
            change(&mut sigs.list);
            cx.emit(Command::SaveList);
            cx.notify();
        });
    }
}

/// The list's column names, the sortable ones clickable: the same column again flips the
/// direction, another starts it going up. Widths follow the rows'.
fn column_header(entity: &Entity<Sigs>, list: &ListView, pad: Rems, theme: &Theme) -> Div {
    let head = |label: &'static str, column: Option<SortColumn>| {
        let active = column.is_some_and(|c| c == list.sort);
        let arrow = if list.descending {
            "icons/chevron-down.svg"
        } else {
            "icons/chevron-up.svg"
        };
        div()
            .id(ElementId::Name(format!("col-{label}").into()))
            .flex()
            .items_center()
            .gap(rpx(2.0))
            .min_w_0()
            .text_color(if active {
                theme.foreground
            } else {
                theme.muted_foreground
            })
            .child(label)
            .when(active, |d| {
                d.child(
                    svg()
                        .flex_none()
                        .path(arrow)
                        .size(rpx(11.0))
                        .text_color(theme.primary),
                )
            })
            .when_some(column, |d, column| {
                d.cursor_pointer()
                    .hover(|s| s.text_color(theme.foreground))
                    .on_click(relist(entity, move |l| {
                        if l.sort == column {
                            l.descending = !l.descending;
                        } else {
                            l.sort = column;
                            l.descending = false;
                        }
                    }))
            })
    };
    div()
        .flex_none()
        .flex()
        .items_center()
        .gap_2()
        .px(pad)
        .h(rpx(22.0))
        .text_size(rpx(11.0))
        .border_b_1()
        .border_color(theme.border.opacity(0.5))
        .child(head("ID", Some(SortColumn::Id)).w(rpx(56.0)).flex_none())
        .child(
            head("Category", Some(SortColumn::Category))
                .w(rpx(72.0))
                .flex_none()
                .pl_1(),
        )
        .child(
            head("Type", Some(SortColumn::Type))
                .w(rpx(84.0))
                .flex_none()
                .pl_1(),
        )
        .child(head("Connection", None).flex_1().pl_1())
        .child(
            head("Age", Some(SortColumn::Age))
                .w(rpx(32.0))
                .flex_none()
                .justify_end(),
        )
        .child(div().w(rpx(44.0)).flex_none())
}

/// One toggle per category, lit while it shows, like the website's panel header.
fn filters(entity: &Entity<Sigs>, list: &ListView, theme: &Theme) -> Div {
    let mut row = div().flex().items_center().gap(rpx(2.0)).ml_auto();
    for (group, _, name) in theme::CATEGORIES {
        let shown = !list.hidden.iter().any(|g| g == group);
        let (_, color) = theme::group_meta(group);
        row = row.child(
            div()
                .id(ElementId::Name(format!("filter-{group}").into()))
                .size(rpx(22.0))
                .rounded(px(3.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(theme.muted))
                .tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(if shown {
                        format!("Hide {name}")
                    } else {
                        format!("Show {name}")
                    })
                    .build(window, cx)
                })
                .on_click(relist(entity, move |l| {
                    if let Some(i) = l.hidden.iter().position(|g| g == group) {
                        l.hidden.remove(i);
                    } else {
                        l.hidden.push(group.to_string());
                    }
                }))
                .child(
                    svg()
                        .path(theme::group_icon(group))
                        .size(rpx(13.0))
                        .text_color(if shown {
                            color.unwrap_or(theme.foreground)
                        } else {
                            theme.muted_foreground.opacity(0.35)
                        }),
                ),
        );
    }
    row
}

/// Write an element's bounds into `slot` as it is laid out, redrawing when they move.
fn probe(slot: &Rc<RefCell<Option<Bounds<Pixels>>>>) -> impl IntoElement {
    let slot = Rc::clone(slot);
    canvas(
        move |bounds, window, _| {
            if *slot.borrow() != Some(bounds) {
                *slot.borrow_mut() = Some(bounds);
                window.refresh();
            }
        },
        |_, (), _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// A picker's choices in a popup hung under its cell, over the rows rather than between
/// them. Laid out as the cell's child, so it is placed in the same frame it opens. It fades
/// in and closes on a press anywhere else.
fn float(entity: &Entity<Sigs>, sigs: &Sigs, picker: Picker, content: Div, cx: &App) -> AnyElement {
    let width = match picker.kind {
        PickerKind::Category => rpx(200.0),
        PickerKind::Menu => rpx(210.0),
        PickerKind::Type => rpx(240.0),
        PickerKind::Connection => rpx(300.0),
    };
    let close = {
        let entity = entity.clone();
        move |event: &MouseDownEvent, _: &mut Window, cx: &mut App| {
            entity.update(cx, |sigs, cx| {
                // A press on the cell itself toggles it, which closes it.
                let on_cell = sigs
                    .anchor
                    .borrow()
                    .is_some_and(|a| a.contains(&event.position));
                if !on_cell && sigs.picker == Some(picker) {
                    sigs.picker = None;
                    cx.notify();
                }
            });
        }
    };
    let surface = div()
        .id(ElementId::Name(
            format!("popup-{}-{:?}", picker.sig, picker.kind).into(),
        ))
        .occlude()
        .relative()
        .w(width)
        .max_h(rpx(340.0))
        .overflow_y_scroll()
        .rounded(px(4.0))
        .popover_style(cx)
        .border_1()
        .border_color(cx.theme().border)
        .p_1()
        .child(content)
        .child(probe(&sigs.popup))
        .on_mouse_down_out(close)
        .with_animation(
            ElementId::Name(format!("popup-in-{}-{:?}", picker.sig, picker.kind).into()),
            Animation::new(std::time::Duration::from_millis(120)).with_easing(ease_out_quint()),
            |d, t| d.opacity(t).mt(px((1.0 - t) * -4.0)),
        );
    let right = picker.kind == PickerKind::Menu;
    div()
        .absolute()
        .top_full()
        .mt(rpx(4.0))
        .map(|d| if right { d.right_0() } else { d.left_0() })
        .child(
            deferred(
                anchored()
                    .anchor(if right {
                        Anchor::TopRight
                    } else {
                        Anchor::TopLeft
                    })
                    .snap_to_window_with_margin(px(8.0))
                    .child(surface),
            )
            .with_priority(1),
        )
        .into_any_element()
}

/// Open `picker`, or close it when it is the one open.
fn toggle(
    entity: &Entity<Sigs>,
    picker: Picker,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + use<> {
    let entity = entity.clone();
    move |_, _, cx| {
        entity.update(cx, |sigs, cx| {
            sigs.picker = (sigs.picker != Some(picker)).then_some(picker);
            cx.notify();
        });
    }
}

/// Close the picker and make the change.
fn choose(
    entity: &Entity<Sigs>,
    key: i64,
    edit: SignatureEdit,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + use<> {
    let entity = entity.clone();
    move |_, _, cx| {
        let edit = edit.clone();
        entity.update(cx, |sigs, cx| {
            sigs.picker = None;
            cx.emit(Command::Edit(key, edit));
            cx.notify();
        });
    }
}

/// A connection's far end: its class, alias and name, the alias tinted by the hole's state.
fn far_label(d: Stateful<Div>, far: Far, hole: Option<Hsla>, theme: &Theme) -> Stateful<Div> {
    let (class, color) = far.class;
    d.child(div().flex_none().text_color(color).child(class))
        .when_some(far.alias.clone(), |d, alias| {
            d.child(
                div()
                    .flex_none()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(hole.unwrap_or(theme.foreground))
                    .child(alias),
            )
        })
        // The system keeps its room; the region gives way first when it is tight.
        .child(
            div()
                .flex_shrink_0()
                .text_color(if far.alias.is_some() {
                    theme.foreground.opacity(0.8)
                } else {
                    hole.unwrap_or(theme.foreground)
                })
                .child(far.name.unwrap_or_else(|| "Unmapped".into())),
        )
        .when_some(far.region, |d, region| {
            d.child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_color(theme.muted_foreground.opacity(0.6))
                    .child(region),
            )
        })
}

/// One choice in a picker's popup; the chosen one is tinted and ticked.
fn option(id: String, selected: bool, theme: &Theme) -> Stateful<Div> {
    div()
        .id(ElementId::Name(id.into()))
        .relative()
        .flex()
        .items_center()
        .gap(rpx(6.0))
        .h(rpx(26.0))
        .pl_2()
        .pr(rpx(26.0))
        .rounded(px(3.0))
        .text_xs()
        .cursor_pointer()
        .when(selected, |d| {
            d.bg(theme.primary.opacity(0.14)).child(
                svg()
                    .absolute()
                    .right(rpx(8.0))
                    .path("icons/check.svg")
                    .size(rpx(12.0))
                    .text_color(theme.primary),
            )
        })
        .hover(|s| s.bg(theme.muted))
}

fn heading(text: &'static str, theme: &Theme) -> Div {
    div()
        .px_2()
        .pt_2()
        .pb_1()
        .text_size(rpx(11.0))
        .text_color(theme.muted_foreground)
        .child(text)
}

fn separator(theme: &Theme) -> Div {
    div().my_1().h(px(1.0)).bg(theme.border)
}

/// The choices for one of a row's cells: what the website's dropdowns offer.
fn picker(
    entity: &Entity<Sigs>,
    sigs: &Sigs,
    sig: &Signature,
    connection: Option<&Connection>,
    kind: PickerKind,
    theme: &Theme,
) -> Div {
    let key = sig.id;
    let mut body = div().flex().flex_col();
    match kind {
        PickerKind::Category => {
            for (group, _, name) in theme::CATEGORIES {
                let (_, color) = theme::group_meta(group);
                body = body.child(
                    option(format!("pick-{key}-cat-{group}"), sig.group == group, theme)
                        .text_color(color.unwrap_or(theme.foreground))
                        .on_click(choose(entity, key, SignatureEdit::Group(group.into())))
                        .child(name),
                );
            }
        }
        PickerKind::Type => {
            let sections = sigs.type_sections(sig);
            if sections.is_empty() {
                return body.child(heading("Pick a category first.", theme));
            }
            body = body.child(
                option(
                    format!("pick-{key}-type-none"),
                    sig.signature_type_id.is_none(),
                    theme,
                )
                .text_color(theme.muted_foreground)
                .on_click(choose(entity, key, SignatureEdit::Type(None)))
                .child("Unknown"),
            );
            for (label, types) in sections {
                if let Some(label) = label {
                    body = body.child(heading(label, theme));
                }
                for t in types {
                    let mut chip = option(
                        format!("pick-{key}-type-{}", t.id),
                        sig.signature_type_id == Some(t.id),
                        theme,
                    )
                    .on_click(choose(
                        entity,
                        key,
                        SignatureEdit::Type(Some(t.id)),
                    ));
                    chip = match &t.signature {
                        Some(code) if sig.group == "wormhole" => {
                            let (short, color) = theme::dest_class_meta(t.target_class);
                            chip.child(code.clone())
                                .child(div().text_color(color).child(short))
                                .when_some(t.extra.clone(), |d, extra| {
                                    d.child(
                                        div()
                                            .text_color(theme.muted_foreground)
                                            .child(format!("({extra})")),
                                    )
                                })
                        }
                        _ => chip.child(t.name.clone()),
                    };
                    body = body.child(chip);
                }
            }
        }
        PickerKind::Menu => {
            let dot = |color: Hsla| div().size(rpx(8.0)).rounded_full().bg(color);
            if sig.group == "wormhole" {
                body = body.child(heading("Mass", theme));
                for (value, name, color) in [
                    (None, "Fresh Mass", theme::healthy()),
                    (Some(MassStatus::Reduced), "Reduced Mass", theme::reduced()),
                    (
                        Some(MassStatus::Critical),
                        "Critical Mass",
                        theme::critical(),
                    ),
                ] {
                    let current = sig.mass_status.filter(|m| *m != MassStatus::Stable);
                    body = body.child(
                        option(format!("pick-{key}-mass-{name}"), current == value, theme)
                            .on_click(choose(entity, key, SignatureEdit::Mass(value)))
                            .child(dot(color))
                            .child(name),
                    );
                }
                body = body.child(heading("Lifetime", theme));
                for (value, name, color) in [
                    (None, "Healthy", theme::healthy()),
                    (Some(TimeStatus::Eol), "End of Life", theme::eol()),
                    (Some(TimeStatus::Critical), "Critical", theme::critical()),
                    (Some(TimeStatus::Expired), "Expired", theme::critical()),
                ] {
                    let current = sig.time_status.filter(|t| *t != TimeStatus::Stable);
                    body = body.child(
                        option(format!("pick-{key}-time-{name}"), current == value, theme)
                            .on_click(choose(entity, key, SignatureEdit::Time(value)))
                            .child(dot(color))
                            .child(name),
                    );
                }
                if let Some(c) = connection {
                    body = body.child(separator(theme)).child(
                        option(format!("pick-{key}-preserve"), c.preserve_mass, theme)
                            .on_click(choose(
                                entity,
                                key,
                                SignatureEdit::PreserveMass {
                                    connection: c.id,
                                    on: !c.preserve_mass,
                                },
                            ))
                            .child("Preserve mass"),
                    );
                }
                body = body.child(separator(theme));
            }
            let entity = entity.clone();
            body = body.child(
                option(format!("pick-{key}-delete"), false, theme)
                    .text_color(theme.danger)
                    .on_click(move |_, _, cx| {
                        entity.update(cx, |sigs, cx| {
                            sigs.picker = None;
                            cx.emit(Command::Delete(key));
                            cx.notify();
                        });
                    })
                    .child(
                        svg()
                            .path("icons/trash-2.svg")
                            .size(rpx(12.0))
                            .text_color(theme.danger),
                    )
                    .child("Delete Signature"),
            );
        }
        PickerKind::Connection => {
            let (free, taken) = sigs.link_options(sig);
            body = body.child(
                option(
                    format!("pick-{key}-conn-none"),
                    sig.connection_id.is_none(),
                    theme,
                )
                .text_color(theme.muted_foreground)
                .on_click(choose(entity, key, SignatureEdit::Unlink))
                .child("Unknown"),
            );
            if free.is_empty() && taken.is_empty() {
                body = body.child(heading("No connection here fits this type yet.", theme));
            }
            for (label, list) in [("Connections", free), ("Connected", taken)] {
                if list.is_empty() {
                    continue;
                }
                body = body.child(heading(label, theme));
                for far in list {
                    let id = far.connection;
                    let chip = option(
                        format!("pick-{key}-conn-{id}"),
                        sig.connection_id == Some(id),
                        theme,
                    )
                    .on_click(choose(entity, key, SignatureEdit::Link(id)));
                    body = body.child(far_label(chip, far, None, theme));
                }
            }
        }
    }
    body
}

/// The question after a jump: which of the origin's signatures was it? One row per option,
/// numbered for the pick keys, and a last row for a hole nobody had scanned.
fn ask_section(entity: &Entity<Sigs>, sigs: &Sigs, ask: &Ask, pad: Rems, theme: &Theme) -> Div {
    let origin = ask
        .origin
        .alias
        .clone()
        .filter(|a| !a.trim().is_empty())
        .unwrap_or_else(|| ask.origin.name.clone());
    let (class, class_color) = theme::class_meta(ask.target.class, Some(ask.target.security));
    let key = |n: usize| format!("{}{n}", sigs.pick_prefix);

    let mut section = div()
        .flex_none()
        .flex()
        .flex_col()
        .border_b_1()
        .border_color(theme.primary.opacity(0.6))
        .bg(theme.primary.opacity(0.06))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .px(pad)
                .pt_2()
                .pb_1()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_1()
                        .font_family(HEADING)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_xs()
                        .child(div().truncate().child(format!("{origin} →")))
                        .child(div().flex_none().text_color(class_color).child(class))
                        .child(div().flex_none().child(ask.target.name.clone()))
                        .when_some(ask.alias.clone(), |d, alias| {
                            d.child(
                                div()
                                    .flex_none()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("as {alias}")),
                            )
                        }),
                )
                .child(
                    div()
                        .id("ask-skip")
                        .flex_none()
                        .px_1()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .cursor_pointer()
                        .hover(|s| s.text_color(theme.foreground))
                        .on_click(emit(entity, Command::SkipJump))
                        .child("Skip"),
                ),
        )
        .child(
            div()
                .px(pad)
                .pb_1()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("Which signature did {} jump?", ask.character)),
        );

    let row = |id: ElementId, number: Option<String>, dim: bool| {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .px(pad)
            .h(rpx(22.0))
            .text_xs()
            .cursor_pointer()
            .hover(|s| s.bg(theme.primary.opacity(0.12)))
            .when(dim, |d| d.opacity(0.55))
            .child(
                div()
                    .w(rpx(64.0))
                    .flex_none()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.primary)
                    .child(number.unwrap_or_default()),
            )
    };
    for (i, sig) in ask.options.iter().enumerate() {
        let label = sigs.type_label(sig);
        let ghost = sig
            .connection_id
            .and_then(|c| ask.ghost_aliases.get(&c).cloned());
        let (group, group_color) = theme::group_meta(&sig.group);
        let number = (i < 9).then(|| key(i + 1));
        section = section.child(
            row(
                ElementId::NamedInteger("ask".into(), i as u64),
                number,
                i >= ask.likely,
            )
            .on_click(emit(entity, Command::Pick(Some(i))))
            .child(
                div()
                    .w(rpx(54.0))
                    .flex_none()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(sig.signature_id.clone()),
            )
            .child(
                div()
                    .w(rpx(40.0))
                    .flex_none()
                    .text_color(group_color.unwrap_or(theme.muted_foreground))
                    .child(group),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .truncate()
                            .when(label.muted, |d| d.text_color(theme.muted_foreground))
                            .child(label.text),
                    )
                    .when_some(label.dest, |d, (short, color)| {
                        d.child(div().flex_none().text_color(color).child(short))
                    })
                    .when_some(ghost, |d, alias| {
                        d.child(
                            div()
                                .flex_none()
                                .text_color(theme.muted_foreground)
                                .child(format!("→ {alias}")),
                        )
                    }),
            ),
        );
    }
    section.child(
        row("ask-none".into(), Some(key(0)), false)
            .on_click(emit(entity, Command::Pick(None)))
            .child(
                div()
                    .text_color(theme.muted_foreground)
                    .child("None of these: a hole nobody scanned"),
            ),
    )
}
