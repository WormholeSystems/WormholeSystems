# WormholeSystems desktop

A companion app for Windows, Linux and macOS that keeps mapping out of the browser. Copy the
probe scanner, press a key, and the scan lands on the map in the system you are in. An
overlay above the game shows that system's signatures and what the paste changed, and
deletes what the scan no longer sees with one click or key. When you jump out of the
chain, the overlay numbers the signatures it could have been; press the number and the
jump is mapped, with the bookmark for the way back on your clipboard. Keys mark the hole
you just flew and name the system you are in.

## How it works

- Where you are comes from EVE's Local chat log, which the game writes the moment you
  land. Keep "Log Chat to File" on in EVE's chat settings (it is by default). With several
  characters logged in, the one who moved last counts, unless Settings tracks one: then
  only that character's system and jumps are used. The app finds the `Chatlogs`
  folder in Documents, OneDrive, the macOS client's folder and Steam's Proton prefix; set
  it by hand if yours is elsewhere.
- Pasting reads the clipboard like the website's signature panel does (`scan.rs` is a port
  of `parseScan` in `frontend/src/lib/map/signatures.ts`, localized clients included;
  change both together). It never deletes: rows the scan no longer sees are marked, and
  "Delete missing" removes them with the website's cascade. The system has to be on the
  map.
- A jump out of a system on the map is mapped the way the website's jump prompt does it
  (`jump.rs` ports the parts that decide; change both together). The wormholes and unknown
  signatures in the system left are offered, the ones whose type could lead to the new
  system's class first. A number key (Ctrl+Alt+1 to 9 by default, 0 for a hole nobody
  scanned) or a click picks one, and one `track-jump` places the system, connects it and
  links the signature, so a wrong pick is one undo. A hole already drawn as a ghost is
  named instead. The number keys are only taken while a jump waits, which is why bare
  digits are a choice too. Gate jumps and the map's ignore list are left alone, a hole
  already mapped is not asked about, and with nothing on the origin to pick from the jump
  is mapped unscanned. Aliases follow the website's "suggest alias" setting and the map's
  scheme.
- Once the jump is on the map, the bookmark for the far end goes to the clipboard, seen
  from where you are: in Rens after jumping from Jita, it names Jita. The naming is a port
  of `frontend/src/lib/naming/bookmark.ts`. If someone maps the jump on the website first,
  the question goes away and the bookmark is copied all the same.
- Ctrl+Alt+A opens a small window to name the system you are in. It takes the keyboard
  (on Wayland as a layer-shell surface, so it shows over the game): Enter saves, Esc
  closes.
- Hole keys act on the last hole jumped and copy the updated bookmark afterwards, so a
  `{mass}` or `{life}` token in your format stays right.
- The overlay sits in a screen corner and takes clicks, never the keyboard. On Wayland it
  is a layer-shell surface, which compositors keep above everything (GNOME has no layer
  shell and gets a pop-up window); elsewhere it is a pop-up window.

Default keys: Ctrl+Alt+V pastes the scan, Ctrl+Alt+D deletes what it no longer sees,
Ctrl+Alt+O shows or hides the overlay, Ctrl+Alt+A names the system, Ctrl+Alt+B copies the
bookmark again, and Ctrl+Alt+R, C and E mark the hole reduced, critical or end of life.
The hole keys wait until a jump's signature is picked.

Closing the window keeps the app running in the tray, overlay and keys included. Click the
tray icon to open the window again; Quit is in its menu. On Linux that takes a
StatusNotifier tray (Waybar, KDE, GNOME with the AppIndicator extension); without one,
closing the window quits.

Sign in with a personal access token from Settings, Tokens on the website. Settings are
kept in `wormholesystems/desktop.toml` in the system's config folder. One that does not
parse is moved to `desktop.toml.broken` instead of being overwritten.

On Linux, global shortcuts go through X11. Under Wayland they fire while an X11 window has
focus, which EVE under Proton is.

## Look

The dark theme follows EVE's Photon UI: near-black frames, a cyan accent, flat outlined
buttons. The light one keeps the website's palette. Both are in `assets/theme.json`, and
the shared pieces (buttons, frames, notices) in `src/ui.rs`. Saira Semi Condensed stands in
for the game's condensed face; it and Inter are bundled from Google Fonts (OFL, licenses
next to them). Class, signature group and hole colors are in `src/theme.rs`, copied from
the website.

## Building

```sh
cargo run --release
```

Linux needs the usual GPUI libraries: on Debian or Ubuntu, `libxkbcommon-dev
libxkbcommon-x11-dev libwayland-dev libxcb1-dev libfontconfig-dev libfreetype-dev
libvulkan-dev`.

It is its own Cargo workspace so the server's builds never need any of that.

## Releasing

CI (`.github/workflows/desktop.yml`) builds and packages Linux x86_64, Windows x86_64 and
macOS (Apple silicon and Intel) on every change here, with cargo-packager (settings under
`[package.metadata.packager]` in `Cargo.toml`). To release, bump `version` in `Cargo.toml`,
merge, then tag that commit:

```sh
git tag desktop-v0.1.0-alpha.1 && git push origin desktop-v0.1.0-alpha.1
```

The tag has to match the version, and a version with a pre-release part (`-alpha.1`) goes
out as a GitHub pre-release. The release gets a setup exe and a portable exe for Windows, a
DMG per Mac, an AppImage and a plain binary for Linux, `SHA256SUMS`, and `latest.json`. It
is never marked as the repo's latest release: that one stays wsctl's, which `install.sh`
downloads.

The app updates itself from those releases (`src/update.rs`). It asks GitHub for the newest
`desktop-v*` release, newer pre-releases included while it is one itself, and offers the
update in the sidebar. Where it runs from the installed exe, the `.app` or the AppImage, it
downloads the signed bundle listed in `latest.json`, checks it against the public key built
in, installs it and restarts; anywhere else it opens the release page. The key pair lives in
the repository's `DESKTOP_UPDATER_PRIVATE_KEY` and `DESKTOP_UPDATER_PRIVATE_KEY_PASSWORD`
secrets and the `DESKTOP_UPDATER_PUBKEY` variable. Losing the private key means existing
installs can no longer update themselves, so keep a copy somewhere safe.

Code is not signed. Windows SmartScreen asks once ("More info", "Run anyway"). On macOS the
app is only ad-hoc signed, so the first start is refused; allow it under System Settings,
Privacy & Security, "Open Anyway".
