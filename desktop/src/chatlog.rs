//! Jumps, read from EVE's Local chat logs. The client writes a line the moment Local changes,
//! which is the moment you land, well before ESI would say so.
//!
//! The files are `Local_<date>_<time>[_<character id>].txt` in the `Chatlogs` folder, UTF-16
//! with a byte order mark. The header names the character (`Listener:`), and every system
//! change is a line like `[ 2026.10.16 13:00:00 ] EVE System > Channel changed to Local : Rens`.
//! The game only writes them with "Log Chat to File" on, which is the default.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const CHANGED: &str = "Channel changed to Local : ";
const LISTENER: &str = "Listener:";

/// Logs untouched for longer than this belong to sessions long gone.
const STALE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    /// Where a character is, learnt without a jump: a log that was already there when the
    /// app started, or the first line of a new session.
    Located { character: String, system: String },
    Jumped {
        character: String,
        from: String,
        to: String,
    },
}

/// Where the game keeps its chat logs on this machine, the first that exists.
pub fn default_dir() -> Option<PathBuf> {
    candidates().into_iter().find(|p| p.is_dir())
}

fn candidates() -> Vec<PathBuf> {
    let tail = Path::new("EVE").join("logs").join("Chatlogs");
    let mut out = Vec::new();
    if let Some(docs) = dirs::document_dir() {
        out.push(docs.join(&tail));
    }
    if let Some(home) = dirs::home_dir() {
        // Windows with Documents in OneDrive.
        out.push(home.join("OneDrive").join("Documents").join(&tail));
        // macOS, the native client.
        out.push(
            home.join("Library/Application Support/EVE Online/p_drive/User/My Documents")
                .join(&tail),
        );
        // Linux, Steam's Proton prefix for EVE (app 8500).
        for steam in [
            ".steam/steam",
            ".local/share/Steam",
            ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        ] {
            out.push(
                home.join(steam)
                    .join("steamapps/compatdata/8500/pfx/drive_c/users/steamuser/Documents")
                    .join(&tail),
            );
        }
    }
    out
}

/// The system a log line moves Local to, if it is such a line.
pub fn parse_change(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("EVE System > ")?;
    let system = rest.strip_prefix(CHANGED)?.trim();
    (!system.is_empty()).then_some(system)
}

fn parse_listener(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix(LISTENER)?.trim();
    (!rest.is_empty()).then_some(rest)
}

/// The character id newer clients put at the end of the file name:
/// `Local_20261016_120000_90000001.txt`.
pub fn character_id(path: &Path) -> Option<i64> {
    let stem = path.file_stem()?.to_str()?;
    let parts: Vec<&str> = stem.split('_').collect();
    match parts.as_slice() {
        [_, _, _, id] => id.parse().ok(),
        _ => None,
    }
}

fn is_local_log(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("Local_") && n.ends_with(".txt"))
}

#[derive(Default)]
struct LogFile {
    offset: u64,
    /// A byte left over when a read ended halfway through a UTF-16 unit.
    odd: Option<u8>,
    /// Text after the last newline, waiting for the rest of its line.
    partial: String,
    listener: Option<String>,
}

impl LogFile {
    /// What was appended since the last read, as complete lines.
    fn read_lines(&mut self, path: &Path) -> std::io::Result<Vec<String>> {
        let mut file = File::open(path)?;
        let len = file.metadata()?.len();
        if len < self.offset {
            // Truncated or replaced: start over.
            *self = LogFile::default();
        }
        if len == self.offset {
            return Ok(Vec::new());
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut fresh = Vec::with_capacity((len - self.offset) as usize);
        file.read_to_end(&mut fresh)?;
        let at_start = self.offset == 0;
        self.offset += fresh.len() as u64;
        let mut bytes: Vec<u8> = self.odd.take().into_iter().collect();
        bytes.extend_from_slice(&fresh);
        let bom = at_start && bytes.starts_with(&[0xFF, 0xFE]);
        Ok(self.decode(&bytes[if bom { 2 } else { 0 }..]))
    }

    fn decode(&mut self, bytes: &[u8]) -> Vec<String> {
        let even = bytes.len() & !1;
        if even < bytes.len() {
            self.odd = Some(bytes[even]);
        }
        let units = bytes[..even]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair));
        let text: String = char::decode_utf16(units)
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        let mut text = std::mem::take(&mut self.partial) + &text;
        let mut lines = Vec::new();
        while let Some(end) = text.find('\n') {
            let line: String = text.drain(..=end).collect();
            lines.push(
                line.trim_end_matches(['\r', '\n'])
                    .trim_start_matches('\u{feff}')
                    .to_string(),
            );
        }
        self.partial = text;
        lines
    }
}

/// Follows every recent Local log in one folder.
pub struct Watcher {
    dir: PathBuf,
    files: HashMap<PathBuf, LogFile>,
    /// Each character's current system, by the name in the log header.
    location: HashMap<String, String>,
    /// Character ids, from the file names, for portraits.
    ids: HashMap<String, i64>,
    /// Polled before: the logs found on the first poll were already there.
    primed: bool,
}

impl Watcher {
    pub fn new(dir: PathBuf) -> Watcher {
        Watcher {
            dir,
            files: HashMap::new(),
            location: HashMap::new(),
            ids: HashMap::new(),
            primed: false,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The characters seen so far, where they are, and their id when the file name had it.
    pub fn locations(&self) -> impl Iterator<Item = (&str, &str, Option<i64>)> {
        self.location
            .iter()
            .map(|(c, s)| (c.as_str(), s.as_str(), self.ids.get(c).copied()))
    }

    /// Read whatever the logs gained since the last call.
    pub fn poll(&mut self) -> std::io::Result<Vec<ChatEvent>> {
        let now = SystemTime::now();
        let mut found = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if !is_local_log(&path) {
                continue;
            }
            let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else {
                continue;
            };
            if now.duration_since(modified).unwrap_or_default() > STALE {
                continue;
            }
            found.push((path, modified));
        }
        // Oldest first, so a character's newer session wins.
        found.sort_by_key(|(_, modified)| *modified);

        let mut events = Vec::new();
        for (path, _) in found {
            // A log that was already there is history: it says where everyone is, not that
            // they just jumped. Told by the first poll rather than by timestamps, which some
            // filesystems keep coarser than the clock.
            let history = !self.primed && !self.files.contains_key(&path);
            let file = self.files.entry(path.clone()).or_default();
            let Ok(lines) = file.read_lines(&path) else {
                continue;
            };
            for line in lines {
                if file.listener.is_none() {
                    if let Some(name) = parse_listener(&line) {
                        file.listener = Some(name.to_string());
                        if let Some(id) = character_id(&path) {
                            self.ids.insert(name.to_string(), id);
                        }
                    }
                    continue;
                }
                let (Some(character), Some(to)) = (file.listener.clone(), parse_change(&line))
                else {
                    continue;
                };
                let to = to.to_string();
                let from = self.location.insert(character.clone(), to.clone());
                match from {
                    Some(from) if !history && from != to => {
                        events.push(ChatEvent::Jumped {
                            character,
                            from,
                            to,
                        });
                    }
                    Some(_) => {}
                    None => events.push(ChatEvent::Located {
                        character,
                        system: to,
                    }),
                }
            }
        }
        self.primed = true;
        // Forget files that went stale, so the map does not grow forever.
        self.files.retain(|path, _| path.exists());
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn utf16(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn header(name: &str) -> String {
        format!(
            "\r\n\r\n        ---------------------------------------------------------------\r\n\
             \r\n          Channel ID:      local\r\n          Channel Name:    Local\r\n\
             \x20         Listener:        {name}\r\n          Session started: 2026.10.16 12:00:00\r\n\
             \x20       ---------------------------------------------------------------\r\n\r\n"
        )
    }

    fn line(system: &str) -> String {
        format!("[ 2026.10.16 12:00:01 ] EVE System > Channel changed to Local : {system}\r\n")
    }

    fn append(path: &Path, text: &str) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        if file.metadata().unwrap().len() == 0 {
            file.write_all(&[0xFF, 0xFE]).unwrap();
        }
        file.write_all(&utf16(text)).unwrap();
    }

    #[test]
    fn parses_a_local_change() {
        assert_eq!(parse_change(&line("Rens")), Some("Rens"));
        assert_eq!(parse_change(&line("J122515")), Some("J122515"));
        assert_eq!(
            parse_change("[ 2026.10.16 12:00:01 ] Someone > Channel changed to Local : Rens"),
            None
        );
        assert_eq!(
            parse_change("[ 2026.10.16 12:00:01 ] EVE System > Channel MOTD: hi"),
            None
        );
    }

    #[test]
    fn a_line_split_across_reads_comes_out_whole() {
        let mut log = LogFile::default();
        let bytes = utf16(&line("Rens"));
        // Cut through the middle of a UTF-16 unit.
        let first = log.decode(&bytes[..7]);
        assert!(first.is_empty());
        let mut rest = vec![log.odd.take().unwrap()];
        rest.extend_from_slice(&bytes[7..]);
        let lines = log.decode(&rest);
        assert_eq!(lines.len(), 1);
        assert_eq!(parse_change(&lines[0]), Some("Rens"));
    }

    #[test]
    fn follows_jumps_and_ignores_history() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Local_20261016_110000_90000001.txt");
        append(
            &old,
            &(header("Old Timer") + &line("Jita") + &line("Perimeter")),
        );

        let mut watcher = Watcher::new(dir.path().to_path_buf());
        // Logs from before the start only say where everyone is.
        assert_eq!(
            watcher.poll().unwrap(),
            vec![ChatEvent::Located {
                character: "Old Timer".into(),
                system: "Jita".into()
            }]
        );
        assert_eq!(
            watcher.locations().collect::<Vec<_>>(),
            vec![("Old Timer", "Perimeter", Some(90000001))]
        );

        append(&old, &line("Rens"));
        assert_eq!(
            watcher.poll().unwrap(),
            vec![ChatEvent::Jumped {
                character: "Old Timer".into(),
                from: "Perimeter".into(),
                to: "Rens".into()
            }]
        );
        assert!(watcher.poll().unwrap().is_empty());

        // A second client logging in after the start.
        let new = dir.path().join("Local_20261016_120000_90000002.txt");
        append(&new, &(header("Scout") + &line("J122515")));
        append(&new, &line("Amarr"));
        assert_eq!(
            watcher.poll().unwrap(),
            vec![
                ChatEvent::Located {
                    character: "Scout".into(),
                    system: "J122515".into()
                },
                ChatEvent::Jumped {
                    character: "Scout".into(),
                    from: "J122515".into(),
                    to: "Amarr".into()
                }
            ]
        );
    }
}
