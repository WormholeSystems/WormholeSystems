//! The main window: a sidebar and four pages. Signatures shows the system you are in and
//! what the last paste did, Activity what happened, Shortcuts the global keys and Settings
//! the rest. Where you are comes from the chat logs; the shortcuts paste scans, sweep what a
//! scan no longer sees, say which signature a jump went through, name systems and mark the
//! hole flown last.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::alias;
use crate::api::{
    Client, MapEntry, MapSnapshot, ResolveGhost, Signature, SignatureCatalog, SignatureEdit,
    SignatureType, TrackJump,
};
use crate::chatlog::{self, ChatEvent, Watcher};
use crate::config::{Config, Corner, OverlaySize};
use crate::focus::{self, Focus};
use crate::hotkeys::{self, Action, PICK_MODIFIERS, Press, Registry};
use crate::jump::{self, AliasScheme, Grid};
use crate::naming::{
    BookmarkContext, BookmarkFormats, BookmarkSystem, connection_bookmark, format_bookmark,
};
use crate::overlay;
use crate::scan;
use crate::sigs::{self, Ask, Command, Here, Review, Sigs, Target, Tone};
use crate::theme::{self, Appearance, HEADING};
use crate::ui::{self, Kind, Mood, Notice, rpx};

/// How often the map and its signatures are fetched again.
const REFRESH: Duration = Duration::from_secs(5);
const LOG_LINES: usize = 60;
const SIDEBAR: f32 = 220.0;
/// The text size choices, in percent.
const TEXT_SIZES: &[(u8, &str)] = &[
    (90, "Small"),
    (100, "Normal"),
    (115, "Large"),
    (130, "Larger"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Signatures,
    Activity,
    Shortcuts,
    Settings,
}

impl Page {
    const ALL: [Page; 4] = [
        Page::Signatures,
        Page::Activity,
        Page::Shortcuts,
        Page::Settings,
    ];

    fn label(self) -> &'static str {
        match self {
            Page::Signatures => "Signatures",
            Page::Activity => "Activity",
            Page::Shortcuts => "Shortcuts",
            Page::Settings => "Settings",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Page::Signatures => "icons/radar.svg",
            Page::Activity => "icons/activity.svg",
            Page::Shortcuts => "icons/keyboard.svg",
            Page::Settings => "icons/settings.svg",
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Page::Signatures => "The system you are in, and what the last paste did.",
            Page::Activity => "Jumps, bookmarks and marks, newest first.",
            Page::Shortcuts => "Keys that work while the game has focus.",
            Page::Settings => "Account, map, chat logs, overlay and looks.",
        }
    }
}

/// The hole flown last, which the shortcuts act on.
#[derive(Clone, Debug)]
struct LastJump {
    map_id: i64,
    connection_id: i64,
    /// The system the bookmark names: where the hole goes from here.
    back_to: String,
    here: String,
}

enum Level {
    Info,
    Done,
    Problem,
}

struct LogLine {
    at: String,
    text: String,
    level: Level,
}

pub struct DesktopApp {
    config: Config,
    page: Page,
    sigs: Entity<Sigs>,
    server: Entity<InputState>,
    token: Entity<InputState>,
    chatlog_dir: Entity<InputState>,
    account: Option<String>,
    maps: Vec<MapEntry>,
    connecting: bool,
    /// Wormhole codes by signature type, for the `{wh}` token.
    codes: Arc<HashMap<i64, String>>,
    catalog: Arc<SignatureCatalog>,
    grid: Grid,
    /// Which systems each system has a stargate to, so gate travel is never mapped.
    stargates: Arc<HashMap<i64, Vec<i64>>>,
    /// The website's "suggest an alias" setting for this map.
    suggest_alias: bool,
    watcher: Option<Arc<Mutex<Watcher>>>,
    /// This machine's characters: name, system, id.
    characters: Vec<(String, String, Option<i64>)>,
    /// Portraits by character name, from EVE's image server; `None` while one loads or
    /// when it could not be had.
    portraits: HashMap<String, Option<Arc<Image>>>,
    /// The character whose system a paste goes to: the tracked one, or whoever moved last.
    active: Option<String>,
    last: Option<LastJump>,
    last_bookmark: Option<String>,
    /// A jump is being looked into or mapped; the next waits its turn.
    thinking: bool,
    /// Jumps that landed while another was being asked about, oldest first.
    queued: VecDeque<(String, String, String)>,
    alias_window: Option<AnyWindowHandle>,
    log: VecDeque<LogLine>,
    capturing: Option<Action>,
    notice_seq: u64,
    focus: Focus,
    focus_rx: Option<std::sync::mpsc::Receiver<Focus>>,
    capture_focus: FocusHandle,
    hotkeys: Registry,
    clipboard: Option<arboard::Clipboard>,
    overlay: Option<AnyWindowHandle>,
    /// A newer release, from the last check.
    update: Option<crate::update::Available>,
    checking_update: bool,
    installing_update: bool,
    /// What the last check said when it found nothing or failed, for Settings.
    update_note: Option<String>,
    _tasks: Vec<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl DesktopApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (config, config_problem) = Config::load();
        let tracked = config.tracked.clone();
        theme::set_text_size(config.text_size, cx);
        theme::apply(config.appearance, window, cx);

        let server = cx.new(|cx| InputState::new(window, cx).default_value(config.server.clone()));
        let token = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("wst_…")
                .default_value(config.token.clone())
        });
        let found = config.chatlog_dir.clone().or_else(chatlog::default_dir);
        let chatlog_dir = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Documents/EVE/logs/Chatlogs")
                .default_value(
                    found
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                )
        });
        let sigs = cx.new(|_| Sigs {
            overlay: config.overlay.clone(),
            list: config.list.clone(),
            pick_prefix: hotkeys::pick_prefix(&config.pick_modifiers),
            ..Default::default()
        });

        let subscriptions = vec![
            cx.subscribe_in(
                &chatlog_dir,
                window,
                |this, _, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                        this.use_chatlog_dir(cx);
                    }
                },
            ),
            cx.observe(&sigs, |_, _, cx| cx.notify()),
            cx.subscribe(&sigs, |this, _, command: &Command, cx| {
                this.command(command.clone(), cx)
            }),
            // Follow the system's light or dark when asked to.
            cx.observe_window_appearance(window, |this, window, cx| {
                if this.config.appearance == Appearance::System {
                    theme::apply(Appearance::System, window, cx);
                }
            }),
        ];

        let mut hotkeys = Registry::new();
        let refused = hotkeys.apply(&config.hotkeys);
        let clipboard = arboard::Clipboard::new().ok();
        let open_overlay = config.overlay.open;

        let mut this = DesktopApp {
            config,
            page: Page::Signatures,
            sigs,
            server,
            token,
            chatlog_dir,
            account: None,
            maps: Vec::new(),
            connecting: false,
            codes: Arc::default(),
            catalog: Arc::default(),
            grid: Grid::default(),
            stargates: Arc::default(),
            suggest_alias: false,
            watcher: None,
            characters: Vec::new(),
            portraits: HashMap::new(),
            active: tracked,
            last: None,
            last_bookmark: None,
            thinking: false,
            queued: VecDeque::new(),
            alias_window: None,
            log: VecDeque::new(),
            capturing: None,
            notice_seq: 0,
            focus: Focus::Unknown,
            focus_rx: Some(focus::watch()),
            capture_focus: cx.focus_handle(),
            hotkeys,
            clipboard,
            overlay: None,
            update: None,
            checking_update: false,
            installing_update: false,
            update_note: None,
            _tasks: Vec::new(),
            _subscriptions: subscriptions,
        };
        if let Some(problem) = config_problem {
            this.note(Level::Problem, problem);
        }
        if let Some(problem) = this.hotkeys.problem.clone() {
            this.note(Level::Problem, problem);
        }
        for (action, why) in refused {
            this.note(Level::Problem, format!("{}: {why}", action.label()));
        }
        if this.clipboard.is_none() {
            this.note(
                Level::Problem,
                "No clipboard to copy to on this system".into(),
            );
        }
        if let Some(dir) = found {
            this.watch(dir);
        } else {
            this.note(
                Level::Problem,
                "Could not find EVE's chat logs; set the folder in Settings".into(),
            );
        }
        this._tasks.push(this.chatlog_loop(cx));
        this._tasks.push(this.hotkey_loop(cx));
        this._tasks.push(this.focus_loop(cx));
        this._tasks.push(this.refresh_loop(cx));
        this._tasks.push(this.update_loop(cx));
        if !this.config.token.is_empty() {
            this.connect(cx);
        } else {
            this.page = Page::Settings;
        }
        if open_overlay {
            this.set_overlay(true, cx);
        }
        this
    }

    fn note(&mut self, level: Level, text: String) {
        let at = chrono::Utc::now().format("%H:%M:%S").to_string();
        self.log.push_front(LogLine { at, text, level });
        self.log.truncate(LOG_LINES);
    }

    fn save(&mut self) {
        if let Err(err) = self.config.save() {
            self.note(
                Level::Problem,
                format!("Could not save the settings: {err}"),
            );
        }
    }

    fn client(&self) -> Option<Client> {
        (!self.config.token.is_empty())
            .then(|| Client::new(&self.config.server, &self.config.token))
    }

    fn copy(&mut self, text: &str) -> bool {
        match self
            .clipboard
            .as_mut()
            .map(|c| c.set_text(text.to_string()))
        {
            Some(Ok(())) => true,
            Some(Err(err)) => {
                self.note(Level::Problem, format!("Could not copy: {err}"));
                false
            }
            None => false,
        }
    }

    // Account and map.

    fn connect(&mut self, cx: &mut Context<Self>) {
        self.config.server = self.server.read(cx).value().trim().to_string();
        self.config.token = self.token.read(cx).value().trim().to_string();
        self.save();
        let Some(client) = self.client() else {
            self.note(Level::Problem, "Paste a token first".into());
            return;
        };
        self.connecting = true;
        cx.notify();
        let fetch = cx.background_spawn(async move {
            let me = client.me()?;
            let maps = client.maps()?;
            let catalog = client.catalog()?;
            let grid = client.grid()?;
            let stargates = client.stargates()?;
            anyhow::Ok((me, maps, catalog, grid, stargates))
        });
        cx.spawn(async move |this, cx| {
            let result = fetch.await;
            this.update(cx, |this, cx| {
                this.connecting = false;
                match result {
                    Ok((me, maps, catalog, grid, stargates)) => {
                        this.note(Level::Done, format!("Signed in as {}", me.name));
                        this.account = Some(me.name);
                        this.grid = grid;
                        this.stargates = Arc::new(stargates);
                        this.codes = Arc::new(
                            catalog
                                .types
                                .iter()
                                .filter_map(|t| Some((t.id, t.signature.clone()?)))
                                .collect(),
                        );
                        this.catalog = Arc::new(catalog);
                        let catalog = this.catalog.clone();
                        this.sigs.update(cx, |sigs, cx| {
                            sigs.catalog = catalog;
                            cx.notify();
                        });
                        if this
                            .config
                            .map_id
                            .is_none_or(|id| !maps.iter().any(|m| m.id == id))
                        {
                            this.config.map_id = maps.first().map(|m| m.id);
                            this.save();
                        }
                        this.maps = maps;
                        if this.page == Page::Settings && this.config.map_id.is_some() {
                            this.page = Page::Signatures;
                        }
                        this.load_map_settings(cx);
                        this.refresh(cx);
                    }
                    Err(err) => {
                        this.account = None;
                        this.note(Level::Problem, format!("Could not sign in: {err}"));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn sign_out(&mut self, cx: &mut Context<Self>) {
        self.config.token.clear();
        self.account = None;
        self.maps.clear();
        self.last = None;
        self.save();
        self.clear_map(cx);
        self.drop_jumps(cx);
        cx.notify();
    }

    fn clear_map(&mut self, cx: &mut Context<Self>) {
        self.sigs.update(cx, |sigs, cx| {
            sigs.snapshot = None;
            sigs.review = None;
            sigs.notice = None;
            cx.notify();
        });
    }

    fn pick_map(&mut self, id: i64, cx: &mut Context<Self>) {
        self.config.map_id = Some(id);
        self.last = None;
        self.save();
        self.clear_map(cx);
        self.drop_jumps(cx);
        self.load_map_settings(cx);
        self.refresh(cx);
        cx.notify();
    }

    /// Fetch the map and its signatures now.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        if self.account.is_none() {
            return;
        }
        let fetch = cx.background_spawn(async move { client.snapshot(map_id) });
        cx.spawn(async move |this, cx| match fetch.await {
            Ok(snapshot) => {
                this.update(cx, |this, cx| {
                    let snapshot = Arc::new(snapshot);
                    this.mapped_elsewhere(&snapshot, cx);
                    this.sigs.update(cx, |sigs, cx| {
                        sigs.snapshot = Some(snapshot);
                        cx.notify();
                    });
                })
                .ok();
            }
            Err(err) => {
                this.update(cx, |this, cx| {
                    this.note(Level::Problem, format!("Could not read the map: {err}"));
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// The website's per-user settings for the map that the app follows.
    fn load_map_settings(&mut self, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        let fetch = cx.background_spawn(async move { client.user_settings(map_id) });
        cx.spawn(async move |this, cx| {
            let result = fetch.await;
            this.update(cx, |this, cx| match result {
                Ok(settings) => {
                    this.suggest_alias = settings.suggest_alias;
                    this.sigs.update(cx, |sigs, cx| {
                        sigs.statics_first = settings.show_statics_first;
                        cx.notify();
                    });
                }
                Err(err) => this.note(
                    Level::Problem,
                    format!("Could not read your map settings: {err}"),
                ),
            })
            .ok();
        })
        .detach();
    }

    fn refresh_loop(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(REFRESH).await;
                if this.update(cx, |this, cx| this.refresh(cx)).is_err() {
                    return;
                }
            }
        })
    }

    // Chat logs.

    fn watch(&mut self, dir: PathBuf) {
        self.note(Level::Info, format!("Watching {}", dir.display()));
        self.watcher = Some(Arc::new(Mutex::new(Watcher::new(dir))));
        self.characters.clear();
    }

    fn use_chatlog_dir(&mut self, cx: &mut Context<Self>) {
        let text = self.chatlog_dir.read(cx).value().trim().to_string();
        let dir = PathBuf::from(&text);
        if self
            .watcher
            .as_ref()
            .is_some_and(|w| w.lock().unwrap().dir() == dir)
        {
            return;
        }
        if !dir.is_dir() {
            self.note(Level::Problem, format!("{text} is not a folder"));
            cx.notify();
            return;
        }
        self.config.chatlog_dir = Some(dir.clone());
        self.save();
        self.watch(dir);
        cx.notify();
    }

    fn chatlog_loop(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let Ok(watcher) = this.read_with(cx, |this, _| this.watcher.clone()) else {
                    return;
                };
                let Some(watcher) = watcher else { continue };
                let polled = cx
                    .background_spawn(async move {
                        let mut watcher = watcher.lock().unwrap();
                        let events = watcher.poll();
                        let mut seen: Vec<(String, String, Option<i64>)> = watcher
                            .locations()
                            .map(|(c, s, id)| (c.to_string(), s.to_string(), id))
                            .collect();
                        seen.sort();
                        events.map(|events| (events, seen))
                    })
                    .await;
                let alive = this.update(cx, |this, cx| match polled {
                    Ok((events, seen)) => {
                        let moved = seen != this.characters;
                        if moved {
                            this.characters = seen;
                            this.fetch_portraits(cx);
                            cx.notify();
                        }
                        // Oldest first, so the last one is whoever moved last.
                        if this.config.tracked.is_none()
                            && let Some(event) = events.last()
                        {
                            let character = match event {
                                ChatEvent::Located { character, .. }
                                | ChatEvent::Jumped { character, .. } => character.clone(),
                            };
                            this.active = Some(character);
                        }
                        if moved {
                            this.locate(cx);
                        }
                        for event in events {
                            if let ChatEvent::Jumped {
                                character,
                                from,
                                to,
                            } = event
                                && this
                                    .config
                                    .tracked
                                    .as_ref()
                                    .is_none_or(|tracked| *tracked == character)
                            {
                                this.jumped(character, from, to, cx);
                            }
                        }
                        if moved {
                            this.refresh(cx);
                        }
                    }
                    Err(err) => {
                        this.note(
                            Level::Problem,
                            format!("Could not read the chat logs: {err}"),
                        );
                        this.watcher = None;
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    return;
                }
            }
        })
    }

    /// Follow one character, or `None` for whoever moves.
    fn track(&mut self, character: Option<String>, cx: &mut Context<Self>) {
        if self.config.tracked == character {
            return;
        }
        self.config.tracked = character.clone();
        self.save();
        if character.is_some() {
            self.active = character;
        }
        self.locate(cx);
        self.refresh(cx);
        cx.notify();
    }

    /// Tell the signatures where you are now.
    fn locate(&mut self, cx: &mut Context<Self>) {
        let here = self.active.as_ref().and_then(|active| {
            self.characters
                .iter()
                .find(|(name, _, _)| name == active)
                .map(|(name, system, _)| Here {
                    character: name.clone(),
                    system: system.clone(),
                })
        });
        self.sigs.update(cx, |sigs, cx| {
            if sigs.here != here {
                sigs.here = here;
                sigs.notice = None;
                cx.notify();
            }
        });
    }

    // Jumps.

    fn jumped(&mut self, character: String, from: String, to: String, cx: &mut Context<Self>) {
        self.note(Level::Info, format!("{character}: {from} → {to}"));
        cx.notify();
        // One at a time, in order: the next jump usually starts where this one lands, which
        // is only on the map once this one is answered.
        if self.thinking || self.sigs.read(cx).ask.is_some() {
            self.queued.push_back((character, from, to));
            return;
        }
        self.consider(character, from, to, cx);
    }

    /// Work out what a jump means for the map, then map it, ask about it or let it be.
    fn consider(&mut self, character: String, from: String, to: String, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        if self.account.is_none() {
            return;
        }
        self.thinking = true;
        let context = self.jump_context();
        let work = cx.background_spawn(async move {
            plan_jump(&client, &context, map_id, &character, &from, &to)
        });
        cx.spawn(async move |this, cx| {
            let plan = work.await;
            this.update(cx, |this, cx| {
                this.thinking = false;
                match plan {
                    Ok(Plan::Skip(why)) => this.note(Level::Info, why),
                    Ok(Plan::Known(jump, bookmark)) => this.landed(jump, bookmark, None, cx),
                    Ok(Plan::Mapped(jump, bookmark)) => {
                        let what = format!("Mapped {} from {}", jump.here, jump.back_to);
                        this.landed(jump, bookmark, Some(what), cx)
                    }
                    Ok(Plan::Ask(ask)) => return this.ask(*ask, cx),
                    Err(err) => this.note(Level::Problem, format!("Could not map the jump: {err}")),
                }
                this.refresh(cx);
                this.next_jump(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn jump_context(&self) -> JumpContext {
        JumpContext {
            catalog: self.catalog.clone(),
            codes: self.codes.clone(),
            grid: self.grid,
            stargates: self.stargates.clone(),
            suggest_alias: self.suggest_alias,
        }
    }

    fn next_jump(&mut self, cx: &mut Context<Self>) {
        if self.thinking || self.sigs.read(cx).ask.is_some() {
            return;
        }
        if let Some((character, from, to)) = self.queued.pop_front() {
            self.consider(character, from, to, cx);
        }
    }

    /// Ask which signature the jump went through, in the panels and with the pick keys.
    fn ask(&mut self, ask: Ask, cx: &mut Context<Self>) {
        if let Some(why) = self.hotkeys.set_picks(Some(&self.config.pick_modifiers)) {
            self.note(Level::Problem, format!("Pick keys: {why}"));
        }
        let text = format!(
            "{} jumped from {} to {}: which signature was it?",
            ask.character, ask.origin.name, ask.target.name
        );
        self.sigs.update(cx, |sigs, cx| {
            sigs.ask = Some(ask);
            cx.notify();
        });
        if self.config.overlay.show_on_jump && self.overlay.is_none() {
            self.set_overlay(true, cx);
        }
        self.note(Level::Info, text);
        cx.notify();
    }

    fn clear_ask(&mut self, cx: &mut Context<Self>) -> Option<Ask> {
        self.hotkeys.set_picks(None);
        self.sigs.update(cx, |sigs, cx| {
            cx.notify();
            sigs.ask.take()
        })
    }

    /// Forget every jump not yet mapped: the map they were for is gone.
    fn drop_jumps(&mut self, cx: &mut Context<Self>) {
        self.clear_ask(cx);
        self.queued.clear();
    }

    /// The answer: the jump went through the option at `index`, or through a hole nobody
    /// scanned.
    fn pick(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        let Some(ask) = self.sigs.read(cx).ask.clone() else {
            return;
        };
        let picked = match index {
            Some(i) => match ask.options.get(i) {
                Some(sig) => Some(sig.clone()),
                None => return,
            },
            None => None,
        };
        let Some(client) = self.client() else { return };
        self.clear_ask(cx);
        self.thinking = true;
        let catalog = self.catalog.clone();
        let codes = self.codes.clone();
        let work = cx.background_spawn(async move {
            submit(&client, &catalog, &codes, &ask, picked.as_ref(), false)
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                this.thinking = false;
                match result {
                    Ok(Some((jump, bookmark))) => {
                        let what = format!("Mapped {} from {}", jump.here, jump.back_to);
                        this.landed(jump, bookmark, Some(what), cx);
                    }
                    Ok(None) => this.note(
                        Level::Problem,
                        "Mapped, but the hole is not on the map now".into(),
                    ),
                    Err(err) => this.note(Level::Problem, format!("Could not map the jump: {err}")),
                }
                this.refresh(cx);
                this.next_jump(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn skip_jump(&mut self, cx: &mut Context<Self>) {
        if let Some(ask) = self.clear_ask(cx) {
            self.note(
                Level::Info,
                format!(
                    "Left {} to {} off the map",
                    ask.origin.name, ask.target.name
                ),
            );
        }
        self.next_jump(cx);
        cx.notify();
    }

    /// A jump now on the map: the hole keys act on it, and its way back is copied.
    fn landed(
        &mut self,
        jump: LastJump,
        bookmark: String,
        what: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let copied = self.config.copy_on_jump && !bookmark.is_empty() && self.copy(&bookmark);
        let mut text = what.map(|w| format!("{w}. ")).unwrap_or_default();
        text += &if copied {
            format!("Copied “{bookmark}”, the way back to {}", jump.back_to)
        } else {
            format!("The way back to {} is “{bookmark}”", jump.back_to)
        };
        self.tell(Tone::Done, text, cx);
        self.last_bookmark = Some(bookmark);
        self.last = Some(jump);
    }

    /// Someone mapped the jump being asked about (the website's own prompt, or a fleet
    /// mate): the question is answered.
    fn mapped_elsewhere(&mut self, snapshot: &MapSnapshot, cx: &mut Context<Self>) {
        let Some(ask) = self.sigs.read(cx).ask.clone() else {
            return;
        };
        let Some(found) = landed_on(
            snapshot,
            ask.map_id,
            &ask.origin.name,
            &ask.target.name,
            &self.codes,
        ) else {
            return;
        };
        if snapshot.signature_on(found.0.connection_id).is_none() {
            return;
        }
        self.clear_ask(cx);
        let (jump, bookmark) = found;
        let what = format!("{} was mapped on the website", jump.here);
        self.landed(jump, bookmark, Some(what), cx);
        self.next_jump(cx);
    }

    // Focus.

    /// Follow which window has focus, hiding the overlay while it should not be up.
    fn focus_loop(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let alive = this.update(cx, |this, cx| {
                    let latest = this.focus_rx.as_ref().and_then(|rx| rx.try_iter().last());
                    if let Some(focus) = latest.filter(|f| *f != this.focus) {
                        this.focus = focus;
                        this.apply_focus(cx);
                    }
                });
                if alive.is_err() {
                    return;
                }
            }
        })
    }

    // Updates.

    /// Look for a new release now, and again every few hours.
    fn update_loop(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                if this
                    .update(cx, |this, cx| this.check_for_update(cx))
                    .is_err()
                {
                    return;
                }
                cx.background_executor()
                    .timer(Duration::from_secs(6 * 60 * 60))
                    .await;
            }
        })
    }

    fn check_for_update(&mut self, cx: &mut Context<Self>) {
        if self.checking_update || self.installing_update {
            return;
        }
        self.checking_update = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let found = cx.background_spawn(async { crate::update::check() }).await;
            this.update(cx, |this, cx| {
                this.checking_update = false;
                match found {
                    Ok(Some(available)) => {
                        this.update = Some(available);
                        this.update_note = None;
                    }
                    Ok(None) => {
                        this.update = None;
                        this.update_note = Some("This is the newest version.".into());
                    }
                    Err(err) => this.update_note = Some(err),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Install in place where this copy can be, else open the release page.
    fn install_update(&mut self, cx: &mut Context<Self>) {
        let Some(available) = self.update.clone() else {
            return;
        };
        let Some(installer) = available.installer else {
            cx.open_url(&available.page);
            return;
        };
        if self.installing_update {
            return;
        }
        self.installing_update = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let done = cx
                .background_spawn(async move { crate::update::install(&installer) })
                .await;
            this.update(cx, |this, cx| match done {
                Ok(()) => crate::update::restart(cx),
                Err(err) => {
                    this.installing_update = false;
                    this.tell(Tone::Problem, format!("Could not update: {err}"), cx);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// The sidebar card that says a new version is out.
    fn update_card(&self, theme: &Theme, cx: &mut Context<Self>) -> Option<Div> {
        let available = self.update.as_ref()?;
        let label = if self.installing_update {
            "Updating…"
        } else if available.installer.is_some() {
            "Update and restart"
        } else {
            "Download"
        };
        Some(
            div()
                .mx_3()
                .mb_3()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .rounded(px(4.0))
                .border_1()
                .border_color(theme.primary.opacity(0.5))
                .bg(theme.primary.opacity(0.08))
                .text_xs()
                .child(
                    div()
                        .font_family(HEADING)
                        .font_weight(FontWeight::MEDIUM)
                        .text_sm()
                        .child(format!("Version {} is out", available.version)),
                )
                .child(
                    ui::button(
                        "update-install",
                        label,
                        Kind::Primary,
                        !self.installing_update,
                        theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.install_update(cx))),
                ),
        )
    }

    fn apply_focus(&mut self, cx: &mut Context<Self>) {
        let o = &self.config.overlay;
        let hidden = o.only_when_focused && !self.focus.shows_overlay(&o.characters);
        self.sigs.update(cx, |sigs, cx| {
            if sigs.overlay_hidden != hidden {
                sigs.overlay_hidden = hidden;
                cx.notify();
            }
        });
        cx.notify();
    }

    // Shortcuts.

    fn hotkey_loop(&self, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                let alive = this.update(cx, |this, cx| {
                    for press in this.hotkeys.pressed() {
                        match press {
                            Press::Action(action) => this.run(action, cx),
                            Press::Pick(0) => this.pick(None, cx),
                            Press::Pick(n) => this.pick(Some(n as usize - 1), cx),
                        }
                    }
                });
                if alive.is_err() {
                    return;
                }
            }
        })
    }

    fn run(&mut self, action: Action, cx: &mut Context<Self>) {
        match action {
            Action::PasteScan => return self.paste(cx),
            Action::DeleteMissing => return self.delete_missing(cx),
            Action::ToggleOverlay => return self.set_overlay(self.overlay.is_none(), cx),
            Action::SetAlias => return self.open_alias(cx),
            _ => {}
        }
        if self.sigs.read(cx).ask.is_some() {
            let keys = format!("{0}1 to {0}9", self.sigs.read(cx).pick_prefix);
            return self.tell(
                Tone::Problem,
                format!(
                    "{}: say which signature the jump was first ({keys})",
                    action.label()
                ),
                cx,
            );
        }
        let (Some(client), Some(last)) = (self.client(), self.last.clone()) else {
            self.note(
                Level::Problem,
                format!("{}: no jump to act on yet", action.label()),
            );
            cx.notify();
            return;
        };
        let codes = self.codes.clone();
        let change = action.change();
        let work = cx.background_spawn(async move {
            if let Some(change) = &change {
                client.set_status(last.map_id, last.connection_id, change)?;
            }
            let snapshot = client.snapshot(last.map_id)?;
            anyhow::Ok(bookmark_for(&snapshot, &last, &codes))
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(bookmark) => {
                        let copied = bookmark.as_deref().is_some_and(|b| this.copy(b));
                        let what = match (action, copied, &bookmark) {
                            (Action::CopyBookmark, true, Some(b)) => format!("Copied “{b}”"),
                            (_, true, Some(b)) => format!("{}, copied “{b}”", action.label()),
                            _ => action.label().to_string(),
                        };
                        if let Some(b) = bookmark {
                            this.last_bookmark = Some(b);
                        }
                        this.note(Level::Done, what);
                    }
                    Err(err) => this.note(Level::Problem, format!("{}: {err}", action.label())),
                }
                this.refresh(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn start_capture(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        self.capturing = Some(action);
        window.focus(&self.capture_focus, cx);
        cx.notify();
    }

    fn captured(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(action) = self.capturing else { return };
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.capturing = None;
        } else if key == "backspace" || key == "delete" {
            // Empty rather than gone, so the default does not come back on the next start.
            self.config.hotkeys.insert(action, String::new());
            self.capturing = None;
            self.rebind();
        } else if let Some(hotkey) = hotkeys::from_keystroke(&event.keystroke) {
            // A shortcut does one thing: taking it away from another action is the point.
            for h in self.config.hotkeys.values_mut() {
                if *h == hotkey {
                    h.clear();
                }
            }
            self.config.hotkeys.insert(action, hotkey);
            self.capturing = None;
            self.rebind();
        } else {
            return;
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn set_pick_modifiers(&mut self, modifiers: &str, cx: &mut Context<Self>) {
        self.config.pick_modifiers = modifiers.to_string();
        self.save();
        let prefix = hotkeys::pick_prefix(modifiers);
        let asking = self.sigs.update(cx, |sigs, cx| {
            sigs.pick_prefix = prefix;
            cx.notify();
            sigs.ask.is_some()
        });
        if asking && let Some(why) = self.hotkeys.set_picks(Some(modifiers)) {
            self.note(Level::Problem, format!("Pick keys: {why}"));
        }
        cx.notify();
    }

    fn rebind(&mut self) {
        for (action, why) in self.hotkeys.apply(&self.config.hotkeys) {
            self.note(Level::Problem, format!("{}: {why}", action.label()));
        }
        self.save();
    }

    // Signatures.

    fn command(&mut self, command: Command, cx: &mut Context<Self>) {
        match command {
            Command::Paste => self.paste(cx),
            Command::DeleteMissing => self.delete_missing(cx),
            Command::Delete(key) => self.delete(vec![key], "Deleted the signature".into(), cx),
            Command::Edit(key, edit) => self.edit_signature(key, edit, cx),
            Command::SaveList => {
                self.config.list = self.sigs.read(cx).list.clone();
                self.save();
            }
            Command::CopyBookmark(key) => self.copy_signature_bookmark(key, cx),
            Command::ClearReview => self.sigs.update(cx, |sigs, cx| {
                sigs.review = None;
                cx.notify();
            }),
            Command::HideOverlay => self.set_overlay(false, cx),
            Command::Pick(index) => self.pick(index, cx),
            Command::SkipJump => self.skip_jump(cx),
            Command::PlaceOverlay {
                x,
                y,
                width,
                height,
            } => {
                let o = &mut self.config.overlay;
                (o.x, o.y) = (Some(x), Some(y));
                if o.dimensions() != (width, height) {
                    (o.width, o.height) = (Some(width), Some(height));
                }
                self.save();
            }
        }
    }

    /// Show the outcome in the panels and the log.
    fn tell(&mut self, tone: Tone, text: String, cx: &mut Context<Self>) {
        let level = match tone {
            Tone::Info => Level::Info,
            Tone::Done => Level::Done,
            Tone::Problem => Level::Problem,
        };
        self.note(level, text.clone());
        self.notice_seq += 1;
        let notice = Notice {
            mood: match tone {
                Tone::Info => Mood::Info,
                Tone::Done => Mood::Done,
                Tone::Problem => Mood::Problem,
            },
            text,
            id: self.notice_seq,
            leaving: false,
        };
        let (id, lasts) = (notice.id, notice.lasts());
        self.sigs.update(cx, |sigs, cx| {
            sigs.notice = Some(notice);
            cx.notify();
        });
        cx.notify();
        // Fade it out, then drop it, unless a newer one took its place.
        let sigs = self.sigs.downgrade();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(lasts).await;
            let still = |cx: &mut App, leave: bool| {
                sigs.update(cx, |sigs, cx| {
                    if sigs.notice.as_ref().is_some_and(|n| n.id == id) {
                        if leave {
                            if let Some(n) = sigs.notice.as_mut() {
                                n.leaving = true;
                            }
                        } else {
                            sigs.notice = None;
                        }
                        cx.notify();
                    }
                })
                .ok();
            };
            cx.update(|cx| still(cx, true));
            this.update(cx, |_, cx| cx.notify()).ok();
            cx.background_executor().timer(ui::FADE).await;
            cx.update(|cx| still(cx, false));
            this.update(cx, |_, cx| cx.notify()).ok();
        })
        .detach();
    }

    fn set_busy(&mut self, busy: bool, cx: &mut Context<Self>) {
        self.sigs.update(cx, |sigs, cx| {
            sigs.busy = busy;
            cx.notify();
        });
    }

    /// Paste the probe scanner's clipboard into the system you are in.
    fn paste(&mut self, cx: &mut Context<Self>) {
        if self.config.overlay.show_on_paste && self.overlay.is_none() {
            self.set_overlay(true, cx);
        }
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return self.tell(Tone::Problem, "Sign in and pick a map first".into(), cx);
        };
        let sigs = self.sigs.read(cx);
        if sigs.busy {
            return;
        }
        let Some(here) = sigs.here.clone() else {
            return self.tell(
                Tone::Problem,
                "No idea where you are yet: the chat logs have no character".into(),
                cx,
            );
        };
        let Some(system) = sigs.system().cloned() else {
            return self.tell(
                Tone::Problem,
                format!("{} is not on the map, so it has nowhere to go", here.system),
                cx,
            );
        };
        let text = match self.clipboard.as_mut().map(|c| c.get_text()) {
            Some(Ok(text)) => text,
            _ => return self.tell(Tone::Problem, "The clipboard holds no text".into(), cx),
        };
        let rows = scan::parse(&text, &self.catalog);
        if rows.is_empty() {
            return self.tell(
                Tone::Problem,
                "Nothing in the clipboard looked like a scan. Select the rows in the probe scanner and copy them first.".into(),
                cx,
            );
        }

        self.set_busy(true, cx);
        let solar_system_id = system.solar_system_id;
        let work = cx.background_spawn(async move {
            // Fresh, so a teammate's paste a moment ago counts as there before this one.
            let before: std::collections::HashSet<String> = client
                .snapshot(map_id)?
                .signatures
                .into_iter()
                .filter(|s| s.solar_system_id == solar_system_id)
                .map(|s| s.signature_id)
                .collect();
            client.paste(map_id, solar_system_id, &rows)?;
            let after = client.snapshot(map_id)?;
            let pasted = rows.into_iter().map(|r| r.signature_id).collect();
            anyhow::Ok((before, pasted, after))
        });
        let name = here.system.clone();
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                this.set_busy(false, cx);
                match result {
                    Ok((before, pasted, after)) => {
                        let review = Review {
                            solar_system_id,
                            before,
                            pasted,
                        };
                        let new = review.pasted.difference(&review.before).count();
                        let updated = review.pasted.intersection(&review.before).count();
                        let missing = review.before.difference(&review.pasted).count();
                        this.sigs.update(cx, |sigs, cx| {
                            sigs.snapshot = Some(Arc::new(after));
                            sigs.review = Some(review);
                            cx.notify();
                        });
                        let mut text = format!("Pasted into {name}: {new} new, {updated} updated");
                        if missing > 0 {
                            text += &format!(", {missing} missing");
                        }
                        this.tell(Tone::Done, text, cx);
                    }
                    Err(err) => this.tell(Tone::Problem, format!("Could not paste: {err}"), cx),
                }
            })
            .ok();
        })
        .detach();
    }

    /// Delete what the last paste no longer saw.
    fn delete_missing(&mut self, cx: &mut Context<Self>) {
        let missing = self.sigs.read(cx).missing();
        if missing.is_empty() {
            return self.tell(Tone::Info, "Nothing missing from the last paste".into(), cx);
        }
        let n = missing.len();
        let text = format!(
            "Deleted {n} signature{} the scan no longer sees",
            if n == 1 { "" } else { "s" }
        );
        self.delete(missing, text, cx);
    }

    fn delete(&mut self, keys: Vec<i64>, done: String, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        if self.sigs.read(cx).busy {
            return;
        }
        self.set_busy(true, cx);
        let work = cx.background_spawn(async move {
            client.remove_signatures(map_id, &keys)?;
            client.snapshot(map_id)
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                this.set_busy(false, cx);
                match result {
                    Ok(after) => {
                        this.sigs.update(cx, |sigs, cx| {
                            sigs.snapshot = Some(Arc::new(after));
                            cx.notify();
                        });
                        this.tell(Tone::Done, done, cx);
                    }
                    Err(err) => this.tell(Tone::Problem, format!("Could not delete: {err}"), cx),
                }
            })
            .ok();
        })
        .detach();
    }

    /// Change one signature the way the website's row does, then show the map as it is now.
    fn edit_signature(&mut self, key: i64, edit: SignatureEdit, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        if self.sigs.read(cx).busy {
            return;
        }
        let done = match &edit {
            SignatureEdit::Group(_) => "Category set",
            SignatureEdit::Type(_) => "Type set",
            SignatureEdit::Link(_) => "Linked to the connection",
            SignatureEdit::Unlink => "Unlinked from its connection",
            SignatureEdit::Mass(_) => "Mass set",
            SignatureEdit::Time(_) => "Lifetime set",
            SignatureEdit::PreserveMass { on: true, .. } => "Preserving mass on the connection",
            SignatureEdit::PreserveMass { on: false, .. } => "No longer preserving mass",
        };
        self.set_busy(true, cx);
        let work = cx.background_spawn(async move {
            client.edit_signature(map_id, key, &edit)?;
            client.snapshot(map_id)
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                this.set_busy(false, cx);
                match result {
                    Ok(after) => {
                        this.sigs.update(cx, |sigs, cx| {
                            sigs.snapshot = Some(Arc::new(after));
                            cx.notify();
                        });
                        this.tell(Tone::Done, done.into(), cx);
                    }
                    Err(err) => this.tell(Tone::Problem, format!("Could not change it: {err}"), cx),
                }
            })
            .ok();
        })
        .detach();
    }

    /// Copy the bookmark for where a signature's hole goes, the way the website's copy
    /// button does: the far system once someone has been, else the class its type promises.
    /// A hole with no name yet gets the next one in the chain, written onto its far end so
    /// the next copy takes the one after.
    fn copy_signature_bookmark(&mut self, key: i64, cx: &mut Context<Self>) {
        let sigs = self.sigs.read(cx);
        let (Some(snapshot), Some(here)) = (sigs.snapshot.clone(), sigs.system().cloned()) else {
            return;
        };
        let Some(sig) = snapshot.signatures.iter().find(|s| s.id == key).cloned() else {
            return;
        };
        let ty = sig
            .signature_type_id
            .and_then(|id| sigs.catalog.types.iter().find(|t| t.id == id))
            .cloned();
        // The connection's other end: a placed system, or a ghost nobody has been through.
        let end = sig
            .connection_id
            .and_then(|c| snapshot.connection(c))
            .map(|c| {
                if c.from_system == here.id {
                    c.to_system
                } else {
                    c.from_system
                }
            });
        let far = end.and_then(|id| snapshot.systems.iter().find(|s| s.id == id));
        let end_alias = end.and_then(|id| {
            far.map(|s| s.alias.clone())
                .or_else(|| {
                    snapshot
                        .ghosts
                        .iter()
                        .find(|g| g.id == id)
                        .map(|g| g.alias.clone())
                })
                .flatten()
                .filter(|a| !a.trim().is_empty())
        });
        let class = far
            .and_then(|s| s.wormhole_class_id)
            .or(ty.as_ref().and_then(|t| t.target_class));
        let mut guessed = None;
        let alias = end_alias.or_else(|| {
            let alias = jump::suggest_alias(
                &here,
                class,
                far.map_or(-1.0, |s| s.security_status),
                &aliases(&snapshot),
                AliasScheme::parse(snapshot.map.naming.alias_scheme.as_deref()),
                &snapshot.map.naming.ignored_alias,
            );
            guessed = alias.clone().zip(end);
            alias
        });
        let text = format_bookmark(
            &BookmarkSystem {
                alias,
                name: far.map(|s| s.name.clone()).unwrap_or_default(),
                region: far.map(|s| s.region.clone()),
                wormhole_class_id: class,
                security: far.map(|s| s.security_status),
                occupier: far.and_then(|s| s.occupying_group.clone()),
                is_home: far.is_some_and(|s| s.is_home),
            },
            &BookmarkContext {
                current_alias: None,
                signature_id: Some(sig.signature_id.clone()),
                size: sig.size,
                mass_status: sig.mass_status,
                time_status: sig.time_status,
                wormhole_code: ty.and_then(|t| t.signature),
            },
            &BookmarkFormats::from(&snapshot.map.naming),
            here.alias.as_deref(),
        );
        if !self.copy(&text) {
            return self.tell(Tone::Problem, "Could not copy the bookmark".into(), cx);
        }
        self.tell(Tone::Done, format!("Copied “{text}”"), cx);
        if let (Some((alias, placement)), Some(client), Some(map_id)) =
            (guessed, self.client(), self.config.map_id)
        {
            let work = cx.background_spawn(async move {
                client.set_alias(map_id, placement, &alias)?;
                client.snapshot(map_id)
            });
            cx.spawn(async move |this, cx| {
                let result = work.await;
                this.update(cx, |this, cx| match result {
                    Ok(after) => this.sigs.update(cx, |sigs, cx| {
                        sigs.snapshot = Some(Arc::new(after));
                        cx.notify();
                    }),
                    Err(err) => {
                        this.tell(Tone::Problem, format!("Could not name the hole: {err}"), cx)
                    }
                })
                .ok();
            })
            .detach();
        }
    }

    // Aliases.

    /// Open the dialog that names the system you are in.
    fn open_alias(&mut self, cx: &mut Context<Self>) {
        if self.alias_window.is_some() {
            return;
        }
        let sigs = self.sigs.read(cx);
        let Some(system) = sigs.system().cloned() else {
            let why = match &sigs.here {
                Some(here) => format!("{} is not on the map, so it cannot be named", here.system),
                None => "No idea where you are yet: the chat logs have no character".into(),
            };
            return self.tell(Tone::Problem, why, cx);
        };
        // Nothing yet: offer what the chain's scheme would call it, from where it was
        // jumped to from.
        let current = system
            .alias
            .clone()
            .filter(|a| !a.trim().is_empty())
            .or_else(|| {
                let snapshot = sigs.snapshot.as_ref()?;
                let last = self.last.as_ref().filter(|l| l.here == system.name)?;
                let origin = snapshot.system_named(&last.back_to)?;
                jump::suggest_alias(
                    origin,
                    system.wormhole_class_id,
                    system.security_status,
                    &aliases(snapshot),
                    AliasScheme::parse(snapshot.map.naming.alias_scheme.as_deref()),
                    &snapshot.map.naming.ignored_alias,
                )
            })
            .unwrap_or_default();
        let this = cx.entity().downgrade();
        let (placement, name) = (system.id, system.name.clone());
        let done: alias::Done = Box::new(move |value, cx| {
            this.update(cx, |this, cx| {
                this.alias_window = None;
                if let Some(value) = value {
                    this.save_alias(placement, name.clone(), value, cx);
                }
            })
            .ok();
        });
        match alias::open(system.name.clone(), current, done, cx) {
            Ok(handle) => self.alias_window = Some(handle),
            Err(err) => self.note(
                Level::Problem,
                format!("Could not open the alias dialog: {err}"),
            ),
        }
    }

    fn save_alias(&mut self, placement: i64, name: String, alias: String, cx: &mut Context<Self>) {
        let (Some(client), Some(map_id)) = (self.client(), self.config.map_id) else {
            return;
        };
        let work = cx.background_spawn({
            let alias = alias.clone();
            async move { client.set_alias(map_id, placement, &alias) }
        });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(()) if alias.is_empty() => {
                        this.tell(Tone::Done, format!("Cleared the alias of {name}"), cx)
                    }
                    Ok(()) => this.tell(Tone::Done, format!("Named {name} {alias}"), cx),
                    Err(err) => {
                        this.tell(Tone::Problem, format!("Could not name {name}: {err}"), cx)
                    }
                }
                this.refresh(cx);
            })
            .ok();
        })
        .detach();
    }

    /// Fetch the portraits of characters the logs know an id for, once each.
    fn fetch_portraits(&mut self, cx: &mut Context<Self>) {
        for (name, _, id) in &self.characters {
            let Some(id) = *id else { continue };
            if self.portraits.contains_key(name) {
                continue;
            }
            self.portraits.insert(name.clone(), None);
            let name = name.clone();
            let fetch = cx.background_spawn(async move {
                let mut response = ureq::get(format!(
                    "https://images.evetech.net/characters/{id}/portrait?size=64"
                ))
                .call()
                .ok()?;
                let bytes = response.body_mut().read_to_vec().ok()?;
                Some(Arc::new(Image::from_bytes(ImageFormat::Jpeg, bytes)))
            });
            cx.spawn(async move |this, cx| {
                if let Some(portrait) = fetch.await {
                    this.update(cx, |this, cx| {
                        this.portraits.insert(name, Some(portrait));
                        cx.notify();
                    })
                    .ok();
                }
            })
            .detach();
        }
    }

    fn portrait(&self, name: &str) -> Option<Arc<Image>> {
        self.portraits.get(name).cloned().flatten()
    }

    // Looks and the overlay.

    fn set_appearance(
        &mut self,
        appearance: Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.config.appearance = appearance;
        self.save();
        theme::apply(appearance, window, cx);
        cx.notify();
    }

    pub fn toggle_overlay(&mut self, cx: &mut Context<Self>) {
        self.set_overlay(self.overlay.is_none(), cx);
    }

    fn set_overlay(&mut self, open: bool, cx: &mut Context<Self>) {
        if let Some(handle) = self.overlay.take() {
            handle
                .update(cx, |_, window, _| window.remove_window())
                .ok();
        }
        self.config.overlay.open = open;
        self.save();
        if open {
            match overlay::open(self.sigs.clone(), &self.config.overlay, cx) {
                Ok(handle) => self.overlay = Some(handle),
                Err(err) => {
                    self.config.overlay.open = false;
                    self.note(Level::Problem, format!("Could not open the overlay: {err}"));
                }
            }
        }
        cx.notify();
    }

    /// Change an overlay setting; an open overlay is reopened to take it.
    fn edit_overlay(
        &mut self,
        cx: &mut Context<Self>,
        edit: impl FnOnce(&mut crate::config::Overlay),
    ) {
        edit(&mut self.config.overlay);
        let settings = self.config.overlay.clone();
        self.sigs.update(cx, |sigs, cx| {
            sigs.overlay = settings;
            cx.notify();
        });
        self.apply_focus(cx);
        if self.config.overlay.open {
            self.set_overlay(true, cx);
        } else {
            self.save();
        }
        cx.notify();
    }
}

/// What the app knows that planning a jump needs, fetched once per sign-in.
#[derive(Clone)]
struct JumpContext {
    catalog: Arc<SignatureCatalog>,
    codes: Arc<HashMap<i64, String>>,
    grid: Grid,
    stargates: Arc<HashMap<i64, Vec<i64>>>,
    suggest_alias: bool,
}

/// What a jump means for the map.
#[derive(Debug)]
enum Plan {
    /// Not part of the chain: why, for the log.
    Skip(String),
    /// Already on the map, signature and all; only the way back is news.
    Known(LastJump, String),
    /// Mapped without asking, since no signature on the origin could have been it.
    Mapped(LastJump, String),
    Ask(Box<Ask>),
}

/// Every alias on the map, ghosts' too.
fn aliases(snapshot: &MapSnapshot) -> Vec<String> {
    snapshot
        .systems
        .iter()
        .filter_map(|s| s.alias.clone())
        .chain(snapshot.ghosts.iter().filter_map(|g| g.alias.clone()))
        .collect()
}

/// The website's jump prompt, minus the prompting: which system it was, whether it is part
/// of the chain at all, and if so what to ask.
fn plan_jump(
    client: &Client,
    context: &JumpContext,
    map_id: i64,
    character: &str,
    from: &str,
    to: &str,
) -> anyhow::Result<Plan> {
    let snapshot = client.snapshot(map_id)?;
    // Only a jump out of the chain extends it; flying around elsewhere maps nothing.
    let Some(origin) = snapshot.system_named(from).cloned() else {
        return Ok(Plan::Skip(format!("{from} is not on the map")));
    };
    let existing = snapshot.system_named(to).cloned();
    let target = match &existing {
        Some(s) => Target {
            solar_system_id: s.solar_system_id,
            name: s.name.clone(),
            class: s.wormhole_class_id,
            security: s.security_status,
        },
        None => match client.find_system(to)? {
            Some(hit) => Target {
                solar_system_id: hit.id,
                name: hit.name,
                class: hit.wormhole_class_id,
                security: hit.security,
            },
            None => return Ok(Plan::Skip(format!("No system called {to}"))),
        },
    };
    if context
        .stargates
        .get(&origin.solar_system_id)
        .is_some_and(|gates| gates.contains(&target.solar_system_id))
    {
        return Ok(Plan::Skip(format!("{from} to {to} was a stargate")));
    }
    // A trade hub the map keeps off stays off; placed by hand, it is somebody's choice.
    if existing.is_none() && client.ignored(map_id)?.contains(&target.solar_system_id) {
        return Ok(Plan::Skip(format!("{to} is on the map's ignore list")));
    }
    let connection = existing
        .as_ref()
        .and_then(|e| snapshot.connection_between(origin.id, e.id))
        .map(|c| c.id);
    if let Some(id) = connection
        && snapshot.signature_on(id).is_some()
        && let Some((jump, bookmark)) = landed_on(&snapshot, map_id, from, to, &context.codes)
    {
        return Ok(Plan::Known(jump, bookmark));
    }

    let types: HashMap<i64, &SignatureType> =
        context.catalog.types.iter().map(|t| (t.id, t)).collect();
    let in_origin: Vec<Signature> = snapshot
        .signatures
        .iter()
        .filter(|s| s.solar_system_id == origin.solar_system_id)
        .cloned()
        .collect();
    let ghosts = jump::ghosts_from(&origin, &snapshot);
    let unflown = ghosts.keys().copied().collect();
    let candidates = jump::candidates(&in_origin, &types, target.class, &unflown);
    let ghost_aliases = ghosts
        .iter()
        .filter_map(|(&connection, ghost)| {
            let alias = snapshot
                .ghosts
                .iter()
                .find(|g| g.id == *ghost)?
                .alias
                .clone()?;
            (!alias.trim().is_empty()).then_some((connection, alias))
        })
        .collect();
    // A system already on the map keeps the name the chain knows it by.
    let alias = match existing.as_ref().and_then(|e| e.alias.clone()) {
        Some(alias) if !alias.trim().is_empty() => Some(alias),
        _ if context.suggest_alias => jump::suggest_alias(
            &origin,
            target.class,
            target.security,
            &aliases(&snapshot),
            AliasScheme::parse(snapshot.map.naming.alias_scheme.as_deref()),
            &snapshot.map.naming.ignored_alias,
        ),
        _ => None,
    };
    let at = jump::free_position(
        &jump::positions(&snapshot.systems, &snapshot.ghosts),
        (origin.position_x, origin.position_y),
        context.grid,
    );
    let ask = Ask {
        character: character.to_string(),
        map_id,
        origin,
        target,
        options: candidates.numbered().cloned().collect(),
        likely: candidates.likely.len(),
        ghosts,
        ghost_aliases,
        alias,
        at,
    };
    if !ask.options.is_empty() {
        return Ok(Plan::Ask(Box::new(ask)));
    }
    // Nothing on the origin could have been it, so there is nothing to ask.
    if connection.is_some() {
        return Ok(
            match landed_on(&snapshot, map_id, from, to, &context.codes) {
                Some((jump, bookmark)) => Plan::Known(jump, bookmark),
                None => Plan::Skip(format!("{from} to {to} is already on the map")),
            },
        );
    }
    Ok(
        match submit(client, &context.catalog, &context.codes, &ask, None, true)? {
            Some((jump, bookmark)) => Plan::Mapped(jump, bookmark),
            None => Plan::Skip(format!("Mapped {to}, but it is not on the map now")),
        },
    )
}

/// Map the jump `ask` is about as going through `picked`, or a hole nobody scanned. With
/// `lone_ghost`, an unscanned jump off a system with exactly one unflown hole drawn is
/// taken to be that hole. One command either way, so a wrong pick is one undo.
fn submit(
    client: &Client,
    catalog: &SignatureCatalog,
    codes: &HashMap<i64, String>,
    ask: &Ask,
    picked: Option<&Signature>,
    lone_ghost: bool,
) -> anyhow::Result<Option<(LastJump, String)>> {
    let ty = picked
        .and_then(|s| s.signature_type_id)
        .and_then(|id| catalog.types.iter().find(|t| t.id == id));
    let size = ty
        .and_then(|t| jump::size_for_jump_mass(t.max_jump_mass))
        .or(picked.and_then(|s| s.size));
    let mass_status = picked.and_then(|s| s.mass_status);
    let time_status = picked.and_then(|s| s.time_status);
    let ghost = match picked {
        Some(sig) => sig.connection_id.and_then(|c| ask.ghosts.get(&c).copied()),
        None if lone_ghost && ask.ghosts.len() == 1 => ask.ghosts.values().next().copied(),
        None => None,
    };
    // A hole scanned and named before anyone flew it keeps that name.
    let alias = picked
        .and_then(|s| s.connection_id)
        .and_then(|c| ask.ghost_aliases.get(&c).cloned())
        .or_else(|| ask.alias.clone());

    if let Some(ghost) = ghost {
        client.resolve_ghost(&ResolveGhost {
            map_id: ask.map_id,
            map_solar_system_id: ghost,
            solar_system_id: ask.target.solar_system_id,
            alias,
            size,
            mass_status,
            time_status,
        })?;
    } else {
        client.track_jump(&TrackJump {
            map_id: ask.map_id,
            from_map_solar_system_id: ask.origin.id,
            to_solar_system_id: ask.target.solar_system_id,
            x: ask.at.0,
            y: ask.at.1,
            signature_pk: picked.map(|s| s.id),
            alias,
            size,
            mass_status,
            time_status,
        })?;
    }
    let snapshot = client.snapshot(ask.map_id)?;
    Ok(landed_on(
        &snapshot,
        ask.map_id,
        &ask.origin.name,
        &ask.target.name,
        codes,
    ))
}

/// The hole between two systems on the map, and the bookmark for the way back to `from`.
fn landed_on(
    snapshot: &MapSnapshot,
    map_id: i64,
    from: &str,
    to: &str,
    codes: &HashMap<i64, String>,
) -> Option<(LastJump, String)> {
    let origin = snapshot.system_named(from)?;
    let here = snapshot.system_named(to)?;
    let connection = snapshot.connection_between(origin.id, here.id)?;
    let jump = LastJump {
        map_id,
        connection_id: connection.id,
        back_to: origin.name.clone(),
        here: here.name.clone(),
    };
    let bookmark = bookmark_for(snapshot, &jump, codes).unwrap_or_default();
    Some((jump, bookmark))
}

/// The bookmark for the way back across `jump`'s connection, as the map names it now.
fn bookmark_for(
    snapshot: &MapSnapshot,
    jump: &LastJump,
    codes: &HashMap<i64, String>,
) -> Option<String> {
    let connection = snapshot.connection(jump.connection_id)?;
    let end = snapshot.system_named(&jump.back_to)?;
    let near = snapshot.system_named(&jump.here)?;
    let formats = BookmarkFormats::from(&snapshot.map.naming);
    Some(connection_bookmark(
        connection,
        end,
        near,
        &snapshot.signatures,
        |id| id.and_then(|id| codes.get(&id).cloned()),
        &formats,
    ))
}

// Rendering.

/// A page on the website at `server`, which may be typed without its scheme.
fn website(server: &str, path: &str) -> String {
    let server = server.trim().trim_end_matches('/');
    let server = if server.is_empty() {
        crate::config::DEFAULT_SERVER
    } else {
        server
    };
    if server.contains("://") {
        format!("{server}{path}")
    } else {
        format!("https://{server}{path}")
    }
}

/// A framed section with a header band.
fn panel(title: &'static str, theme: &Theme) -> Div {
    ui::section(title, theme)
}

fn icon(path: &'static str, size: f32, color: Hsla) -> Svg {
    svg()
        .path(path)
        .size(rpx(size))
        .flex_none()
        .text_color(color)
}

fn muted(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .text_sm()
        .text_color(theme.muted_foreground)
        .child(text.into())
}

/// A row of toggle buttons, one per choice, the chosen one filled.
fn segmented<T: Copy + PartialEq + 'static>(
    id: &'static str,
    choices: &[(T, &'static str)],
    current: T,
    cx: &mut Context<DesktopApp>,
    pick: impl Fn(&mut DesktopApp, T, &mut Window, &mut Context<DesktopApp>) + Clone + 'static,
) -> Div {
    let mut row = div().flex().flex_wrap().gap_1();
    for (i, &(value, label)) in choices.iter().enumerate() {
        let pick = pick.clone();
        let kind = if value == current {
            Kind::Selected
        } else {
            Kind::Normal
        };
        let theme = cx.theme().clone();
        row = row.child(
            ui::button(
                ElementId::NamedInteger(id.into(), i as u64),
                label,
                kind,
                true,
                &theme,
            )
            .on_click(cx.listener(move |this, _, window, cx| pick(this, value, window, cx))),
        );
    }
    row
}

impl DesktopApp {
    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let update_card = self.update_card(&theme, cx);
        let map_name = self
            .sigs
            .read(cx)
            .snapshot
            .as_ref()
            .map(|s| s.map.name.clone());

        let brand = div()
            .h(rpx(56.0))
            .px_4()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(theme.sidebar_border)
            .child(
                svg()
                    .path("logo.svg")
                    .size(rpx(18.0))
                    .text_color(theme.sidebar_foreground),
            )
            .child(
                div()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_sm()
                    .child("WormholeSystems"),
            )
            .child(
                div()
                    .px(rpx(5.0))
                    .py(px(1.0))
                    .border_1()
                    .border_color(theme.primary.opacity(0.5))
                    .text_color(theme.primary)
                    .text_size(rpx(9.0))
                    .child(stage()),
            );

        let mut nav = div().flex().flex_col().py_3();
        for page in Page::ALL {
            let active = self.page == page;
            nav = nav.child(
                div()
                    .id(page.label())
                    .relative()
                    .h(rpx(38.0))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .text_color(if active {
                        theme.sidebar_foreground
                    } else {
                        theme.muted_foreground
                    })
                    .when(active, |d| d.bg(theme.sidebar_accent))
                    .hover(|s| {
                        s.bg(theme.sidebar_accent.opacity(0.6))
                            .text_color(theme.sidebar_foreground)
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.page = page;
                        cx.notify();
                    }))
                    .when(active, |d| {
                        d.child(
                            div()
                                .absolute()
                                .left_0()
                                .top_0()
                                .bottom_0()
                                .w(px(2.0))
                                .bg(theme.primary),
                        )
                    })
                    .child(icon(
                        page.icon(),
                        16.0,
                        if active {
                            theme.primary
                        } else {
                            theme.muted_foreground
                        },
                    ))
                    .child(
                        div()
                            .font_family(HEADING)
                            .font_weight(FontWeight::MEDIUM)
                            .text_sm()
                            .child(page.label()),
                    ),
            );
        }

        // Who pastes and jumps go to: the tracked character, else whoever moved last.
        let pilot = self
            .config
            .tracked
            .clone()
            .or_else(|| self.active.clone())
            .map(|name| {
                let system = self
                    .characters
                    .iter()
                    .find(|(c, _, _)| *c == name)
                    .map(|(_, s, _)| s.clone());
                let face = rpx(32.0);
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pb_2()
                    .child(match self.portrait(&name) {
                        Some(portrait) => img(portrait)
                            .size(face)
                            .flex_none()
                            .rounded(px(2.0))
                            .into_any_element(),
                        None => div()
                            .size(face)
                            .flex_none()
                            .rounded(px(2.0))
                            .bg(theme.foreground.opacity(0.08))
                            .into_any_element(),
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(
                                div()
                                    .font_family(HEADING)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_sm()
                                    .truncate()
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .truncate()
                                    .text_color(theme.muted_foreground)
                                    .child(system.unwrap_or_else(|| "Not logged in".into())),
                            ),
                    )
            });

        // The account itself is under Settings; here only whether it is signed in, and the map.
        let signed_in = self.account.is_some();
        let status = div()
            .px_4()
            .py_3()
            .flex()
            .flex_col()
            .gap_1()
            .border_t_1()
            .border_color(theme.sidebar_border)
            .children(pilot)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .size(rpx(7.0))
                            .flex_none()
                            .rounded_full()
                            .bg(if signed_in {
                                theme.success
                            } else {
                                theme.muted_foreground
                            }),
                    )
                    .text_color(theme.muted_foreground)
                    .child(div().truncate().child(if signed_in {
                        map_name.unwrap_or_else(|| "No map".into())
                    } else {
                        "Signed out".into()
                    })),
            );

        let overlay_open = self.overlay.is_some();
        let appearance = self.config.appearance;
        let tool = |id: &'static str, path: &'static str, label: &'static str, on: bool| {
            let color = if on {
                theme.primary
            } else {
                theme.muted_foreground
            };
            div()
                .id(id)
                .flex_1()
                .h(rpx(40.0))
                .flex()
                .items_center()
                .justify_center()
                .gap_2()
                .cursor_pointer()
                .text_xs()
                .text_color(color)
                .hover(|s| s.bg(theme.sidebar_accent))
                .child(icon(path, 14.0, color))
                .child(label)
        };
        let tools = div()
            .flex()
            .border_t_1()
            .border_color(theme.sidebar_border)
            .child(
                tool(
                    "toggle-overlay",
                    "icons/picture-in-picture-2.svg",
                    "Overlay",
                    overlay_open,
                )
                .on_click(cx.listener(move |this, _, _, cx| this.set_overlay(!overlay_open, cx))),
            )
            .child(div().w(px(1.0)).bg(theme.sidebar_border))
            .child(
                tool("toggle-theme", appearance.icon(), appearance.label(), false).on_click(
                    cx.listener(move |this, _, window, cx| {
                        this.set_appearance(appearance.next(), window, cx)
                    }),
                ),
            );

        div()
            .w(rpx(SIDEBAR))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.sidebar_border)
            .text_color(theme.sidebar_foreground)
            .child(brand)
            .child(nav)
            .child(div().flex_1())
            .children(update_card)
            .child(status)
            .child(tools)
    }

    fn header(&self, theme: &Theme, cx: &App) -> Div {
        div()
            .flex_none()
            .h(rpx(56.0))
            .px_6()
            .flex()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_xl()
                    .child(self.page.label()),
            )
            .child(div().w(px(1.0)).h(rpx(16.0)).bg(theme.border))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(muted(self.page.blurb(), theme)),
            )
            .when(self.page != Page::Signatures, |d| {
                d.when_some(self.sigs.read(cx).notice.as_ref(), |d, n| {
                    d.child(
                        div()
                            .flex_none()
                            .max_w(rpx(420.0))
                            .child(ui::notice(n, theme)),
                    )
                })
            })
    }

    fn signatures_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        if self.account.is_none() {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(muted("Sign in under Settings to paste scans.", &theme))
                .into_any_element();
        }
        let keys = |action: Action| {
            self.config
                .hotkeys
                .get(&action)
                .filter(|k| !k.is_empty())
                .map(|k| hotkeys::display(k))
        };
        let hint = match (keys(Action::PasteScan), keys(Action::DeleteMissing)) {
            (Some(paste), Some(sweep)) => format!(
                "In the probe scanner, select all and copy, then press {paste}. {sweep} deletes what the scan no longer sees."
            ),
            (Some(paste), None) => {
                format!("In the probe scanner, select all and copy, then press {paste}.")
            }
            _ => "In the probe scanner, select all and copy, then paste here.".into(),
        };
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex_none()
                    .px_6()
                    .py_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(muted(hint, &theme)),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(sigs::panel(&self.sigs, false, cx)),
            )
            .into_any_element()
    }

    fn activity_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let last = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(match &self.last_bookmark {
                Some(b) => div()
                    .font_family(HEADING)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_xl()
                    .text_color(theme.primary)
                    .child(b.clone()),
                None => muted("Nothing copied yet. Jump through a mapped hole.", &theme),
            })
            .when_some(self.last.as_ref(), |d, last| {
                d.child(muted(
                    format!("The way back from {} to {}", last.here, last.back_to),
                    &theme,
                ))
            });

        let mut lines = panel("Log", &theme);
        if self.log.is_empty() {
            lines = lines.child(muted("Nothing yet.", &theme));
        }
        for line in &self.log {
            let (color, mark) = match line.level {
                Level::Info => (theme.muted_foreground, theme.border),
                Level::Done => (theme.foreground, theme.success),
                Level::Problem => (theme.danger, theme.danger),
            };
            lines = lines.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_sm()
                    .child(div().size(rpx(6.0)).flex_none().rounded_full().bg(mark))
                    .child(
                        div()
                            .flex_none()
                            .font_family(HEADING)
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(line.at.clone()),
                    )
                    .child(div().text_color(color).child(line.text.clone())),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .child(panel("Last bookmark", &theme).child(last))
            .child(lines)
            .into_any_element()
    }

    fn shortcuts_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let target = match &self.last {
            Some(last) => format!(
                "The hole keys act on the last hole you jumped: {} to {}.",
                last.back_to, last.here
            ),
            None => "The hole keys act on the last hole you jumped.".to_string(),
        };
        let mut list = panel("Keys", &theme).child(muted(
            "Paste and delete act on the system the tracked character is in (Settings, EVE chat logs).",
            &theme,
        ));
        for action in Action::ALL {
            if action == Action::CopyBookmark {
                list = list.child(div().pt_2().child(muted(target.clone(), &theme)));
            }
            let capturing = self.capturing == Some(action);
            let label = if capturing {
                "Press keys…".to_string()
            } else {
                self.config
                    .hotkeys
                    .get(&action)
                    .filter(|h| !h.is_empty())
                    .map(|h| hotkeys::display(h))
                    .unwrap_or_else(|| "Not set".into())
            };
            let button = ui::button(
                SharedString::from(format!("hotkey-{action:?}")),
                label,
                if capturing {
                    Kind::Selected
                } else {
                    Kind::Normal
                },
                true,
                &theme,
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.start_capture(action, window, cx)),
            );
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .py_1()
                    .border_b_1()
                    .border_color(theme.border.opacity(0.5))
                    .child(div().text_sm().child(action.label()))
                    .child(button),
            );
        }
        list = list.child(muted(
            "Click a key and press the new one. Esc cancels, Backspace clears.",
            &theme,
        ));
        let picks = panel("Picking the signature", &theme)
            .child(muted(
                "After a jump out of the chain, the overlay numbers the signatures it could have been. Press the number to map the jump through that one, or 0 for a hole nobody scanned. The keys are only taken while a jump waits for its answer.",
                &theme,
            ))
            .child(segmented(
                "pick-modifiers",
                &PICK_MODIFIERS,
                PICK_MODIFIERS
                    .iter()
                    .map(|(m, _)| *m)
                    .find(|m| *m == self.config.pick_modifiers)
                    .unwrap_or(PICK_MODIFIERS[0].0),
                cx,
                |this, modifiers, _, cx| this.set_pick_modifiers(modifiers, cx),
            ));
        if let Some(problem) = &self.hotkeys.problem {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(problem.clone()),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .max_w(rpx(720.0))
            .child(list)
            .child(picks)
            .into_any_element()
    }

    fn settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();

        let account = if let Some(name) = &self.account {
            panel("Account", &theme).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_sm().child(format!(
                        "{name} on {}",
                        self.config.server.trim_start_matches("https://")
                    )))
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .child(
                                ui::button("settings", "Manage tokens", Kind::Normal, true, &theme)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.open_url(&website(
                                            &this.config.server,
                                            "/settings/tokens",
                                        ))
                                    })),
                            )
                            .child(
                                ui::button("sign-out", "Sign out", Kind::Normal, true, &theme)
                                    .on_click(cx.listener(|this, _, _, cx| this.sign_out(cx))),
                            ),
                    ),
            )
        } else {
            panel("Account", &theme)
                .child(muted(
                    "Make a token on the website under Settings, Tokens, and paste it here.",
                    &theme,
                ))
                .child(Input::new(&self.server))
                .child(Input::new(&self.token).mask_toggle())
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            ui::button(
                                "connect",
                                if self.connecting {
                                    "Signing in…"
                                } else {
                                    "Sign in"
                                },
                                Kind::Primary,
                                !self.connecting,
                                &theme,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.connect(cx))),
                        )
                        .child(
                            ui::button("get-token", "Make a token", Kind::Normal, true, &theme)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let server = this.server.read(cx).value().to_string();
                                    cx.open_url(&website(&server, "/settings/tokens"))
                                })),
                        ),
                )
        };

        let maps = (!self.maps.is_empty()).then(|| {
            let mut row = div().flex().flex_wrap().gap_1();
            for map in &self.maps {
                let id = map.id;
                let kind = if self.config.map_id == Some(id) {
                    Kind::Selected
                } else {
                    Kind::Normal
                };
                row = row.child(
                    ui::button(
                        ElementId::Integer(id as u64),
                        map.name.clone(),
                        kind,
                        true,
                        &theme,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.pick_map(id, cx))),
                );
            }
            panel("Map", &theme).child(row)
        });

        let mut logs = panel("EVE chat logs", &theme).child(Input::new(&self.chatlog_dir));
        if self.watcher.is_some() && self.characters.is_empty() {
            logs = logs.child(muted(
                "No Local logs yet. Log in a character, and keep \"Log Chat to File\" on in EVE's chat settings.",
                &theme,
            ));
        }
        if !self.characters.is_empty() || self.config.tracked.is_some() {
            // Everyone the logs know, and the tracked one even while logged out.
            let mut names: Vec<String> =
                self.characters.iter().map(|(c, _, _)| c.clone()).collect();
            if let Some(tracked) = &self.config.tracked
                && !names.contains(tracked)
            {
                names.push(tracked.clone());
                names.sort();
            }
            let mut chips = div().flex().flex_wrap().gap_1().child(
                ui::button(
                    "track-any",
                    "Whoever moved last",
                    if self.config.tracked.is_none() {
                        Kind::Selected
                    } else {
                        Kind::Normal
                    },
                    true,
                    &theme,
                )
                .on_click(cx.listener(|this, _, _, cx| this.track(None, cx))),
            );
            for (i, name) in names.into_iter().enumerate() {
                let picked = self.config.tracked.as_ref() == Some(&name);
                chips = chips.child(
                    ui::character(
                        ElementId::NamedInteger("track-char".into(), i as u64),
                        name.clone(),
                        self.portrait(&name),
                        if picked { Kind::Selected } else { Kind::Normal },
                        &theme,
                    )
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.track(Some(name.clone()), cx)),
                    ),
                );
            }
            logs = logs.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .text_sm()
                    .child("Track")
                    .child(chips)
                    .child(muted(
                        "Pastes go to this character's system, and only their jumps are mapped.",
                        &theme,
                    )),
            );
        }
        let row = |label: &'static str, control: AnyElement| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .text_sm()
                .child(label)
                .child(control)
        };
        logs = logs.child(row(
            "Copy the bookmark back on every jump",
            Switch::new("copy-on-jump")
                .checked(self.config.copy_on_jump)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.config.copy_on_jump = *checked;
                    this.save();
                    cx.notify();
                }))
                .into_any_element(),
        ));

        let o = self.config.overlay.clone();
        let overlay = panel("Overlay", &theme)
            .child(muted(
                "The signatures in a panel that stays above the game. Drag it by its title and size it by the grip in its corner. It takes clicks but never the keyboard.",
                &theme,
            ))
            .child(row(
                "Show the overlay",
                Switch::new("overlay-open")
                    .checked(self.overlay.is_some())
                    .on_click(
                        cx.listener(|this, checked: &bool, _, cx| this.set_overlay(*checked, cx)),
                    )
                    .into_any_element(),
            ))
            .child(row(
                "Position and size",
                ui::button("overlay-reset", "Reset", Kind::Normal, true, &theme)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.edit_overlay(cx, |o| {
                            o.corner = Corner::TopRight;
                            o.size = OverlaySize::Medium;
                            (o.x, o.y, o.width, o.height) = (None, None, None, None);
                        })
                    }))
                    .into_any_element(),
            ))
            .child(row(
                "Text size",
                segmented("overlay-text-size", TEXT_SIZES, o.text_size, cx, |this, v, _, cx| {
                    this.edit_overlay(cx, |o| o.text_size = v)
                })
                .into_any_element(),
            ))
            .child(row(
                "Background",
                segmented(
                    "opacity",
                    &[(60u8, "60%"), (75, "75%"), (85, "85%"), (100, "100%")],
                    o.opacity,
                    cx,
                    |this, v, _, cx| this.edit_overlay(cx, |o| o.opacity = v),
                )
                .into_any_element(),
            ))
            .child(row(
                "Show it when a jump needs its signature",
                Switch::new("overlay-on-jump")
                    .checked(o.show_on_jump)
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        let checked = *checked;
                        this.edit_overlay(cx, |o| o.show_on_jump = checked)
                    }))
                    .into_any_element(),
            ))
            .child(row(
                "Show it when a scan is pasted",
                Switch::new("overlay-on-paste")
                    .checked(o.show_on_paste)
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        let checked = *checked;
                        this.edit_overlay(cx, |o| o.show_on_paste = checked)
                    }))
                    .into_any_element(),
            ));

        let mut overlay = overlay.child(row(
            "Only while EVE or this app has focus",
            Switch::new("overlay-focused")
                .checked(o.only_when_focused)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.edit_overlay(cx, |o| o.only_when_focused = checked)
                }))
                .into_any_element(),
        ));
        if o.only_when_focused && !focus::supported() {
            overlay = overlay.child(muted(
                "This desktop does not say which window has focus, so the overlay stays up. It works on Hyprland and Windows.",
                &theme,
            ));
        }
        if o.only_when_focused {
            // Everyone the chat logs know, and anyone picked before who is not logged in now.
            let mut names: Vec<String> =
                self.characters.iter().map(|(c, _, _)| c.clone()).collect();
            for c in &o.characters {
                if !names.contains(c) {
                    names.push(c.clone());
                }
            }
            names.sort();
            let mut chips = div().flex().flex_wrap().gap_1().child(
                ui::button(
                    "overlay-any",
                    "Any character",
                    if o.characters.is_empty() {
                        Kind::Selected
                    } else {
                        Kind::Normal
                    },
                    true,
                    &theme,
                )
                .on_click(
                    cx.listener(|this, _, _, cx| this.edit_overlay(cx, |o| o.characters.clear())),
                ),
            );
            for (i, name) in names.into_iter().enumerate() {
                let picked = o.characters.contains(&name);
                chips = chips.child(
                    ui::character(
                        ElementId::NamedInteger("overlay-char".into(), i as u64),
                        name.clone(),
                        self.portrait(&name),
                        if picked { Kind::Selected } else { Kind::Normal },
                        &theme,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let name = name.clone();
                        this.edit_overlay(cx, |o| {
                            if let Some(i) = o.characters.iter().position(|c| *c == name) {
                                o.characters.remove(i);
                            } else {
                                o.characters.push(name);
                            }
                        })
                    })),
                );
            }
            let now = match &self.focus {
                Focus::Eve(Some(name)) => format!("In front now: {name}'s client."),
                Focus::Eve(None) => "In front now: EVE, no character yet.".into(),
                Focus::Us => "In front now: this app.".into(),
                Focus::Other => "In front now: another app, so the overlay is hidden.".into(),
                Focus::Unknown => String::new(),
            };
            overlay = overlay
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .text_sm()
                        .child("For these characters")
                        .child(chips),
                )
                .when(!now.is_empty(), |d| d.child(muted(now, &theme)));
        }

        let looks = panel("Appearance", &theme)
            .child(row(
                "Theme",
                segmented(
                    "appearance",
                    &[
                        (Appearance::System, "System"),
                        (Appearance::Light, "Light"),
                        (Appearance::Dark, "Dark"),
                    ],
                    self.config.appearance,
                    cx,
                    |this, a, window, cx| this.set_appearance(a, window, cx),
                )
                .into_any_element(),
            ))
            .child(row(
                "Text size",
                segmented(
                    "text-size",
                    TEXT_SIZES,
                    self.config.text_size,
                    cx,
                    |this, v, _, cx| {
                        this.config.text_size = v;
                        this.save();
                        theme::set_text_size(v, cx);
                        cx.notify();
                    },
                )
                .into_any_element(),
            ));

        let status = if self.checking_update {
            "Checking…".to_string()
        } else if let Some(available) = &self.update {
            format!("Version {} is out.", available.version)
        } else {
            self.update_note.clone().unwrap_or_default()
        };
        let about = panel("Version", &theme).child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_4()
                .text_sm()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(format!("WormholeSystems {}", crate::update::VERSION))
                        .when(!status.is_empty(), |d| d.child(muted(status, &theme))),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            ui::button("source", "Source code", Kind::Normal, true, &theme)
                                .on_click(|_, _, cx| {
                                    cx.open_url(&format!(
                                        "https://github.com/{}",
                                        crate::update::REPO
                                    ))
                                }),
                        )
                        .child(
                            ui::button(
                                "update-check",
                                "Check now",
                                Kind::Normal,
                                !self.checking_update && !self.installing_update,
                                &theme,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.check_for_update(cx))),
                        ),
                ),
        );

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .max_w(rpx(720.0))
            .child(account)
            .children(maps)
            .child(logs)
            .child(overlay)
            .child(looks)
            .child(about)
            .into_any_element()
    }
}

impl Render for DesktopApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let page = match self.page {
            Page::Signatures => self.signatures_page(cx),
            Page::Activity => self.activity_page(cx),
            Page::Shortcuts => self.shortcuts_page(cx),
            Page::Settings => self.settings_page(cx),
        };
        let body = if self.page == Page::Signatures {
            page
        } else {
            div()
                .id("page-scroll")
                .flex_1()
                .overflow_y_scroll()
                .child(page)
                .into_any_element()
        };

        div()
            .id("root")
            .track_focus(&self.capture_focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| this.captured(event, cx)))
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme::UI)
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(self.header(&theme, cx))
                    .child(body),
            )
    }
}

/// The sidebar badge: the release stage while there is one (`0.1.0-alpha.1` is an alpha),
/// else just that this is the desktop app.
fn stage() -> &'static str {
    let version = env!("CARGO_PKG_VERSION");
    if version.contains("alpha") {
        "ALPHA"
    } else if version.contains("beta") {
        "BETA"
    } else {
        "DESKTOP"
    }
}
