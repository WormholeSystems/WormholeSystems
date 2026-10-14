//! Reading and writing the `.env` the stack is configured from.
//!
//! Values are patched into the existing file rather than the file being rewritten, so the
//! comments explaining each setting survive, and anything the operator added by hand stays
//! where they put it.

use std::collections::BTreeMap;

/// Replace the values of matching `KEY=...` lines and keep everything else verbatim. Keys
/// the file does not already have are appended. A key that is there twice has both lines
/// replaced: compose reads the last one, a person reading the file finds the first, and
/// the two should not disagree.
pub fn patch(current: &str, values: &BTreeMap<String, String>) -> String {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = String::new();

    for line in current.lines() {
        let patched = parse(line).and_then(|(export, key, _)| {
            let value = values.get(key)?;
            seen.insert(key.to_string());
            Some(format!("{export}{key}={}", quote(value)))
        });
        out.push_str(patched.as_deref().unwrap_or(line));
        out.push('\n');
    }

    let remaining: Vec<_> = values.iter().filter(|(k, _)| !seen.contains(*k)).collect();
    if !remaining.is_empty() {
        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str("# Added by wsctl\n");
        for (key, value) in remaining {
            out.push_str(&format!("{key}={}\n", quote(value)));
        }
    }

    out
}

/// The value of a `KEY=...` line as compose reads it: the last one when there are several,
/// quotes and escapes undone.
pub fn get(env: &str, key: &str) -> Option<String> {
    env.lines()
        .rev()
        .filter_map(parse)
        .find(|(_, k, _)| *k == key)
        .map(|(_, _, value)| unquote(value))
}

/// Drop a key entirely, every line of it. Discord is read as all-or-nothing by the server,
/// so an empty value left behind would read as "configured" and stop it starting.
pub fn remove(current: &str, key: &str) -> String {
    current
        .lines()
        .filter(|line| parse(line).is_none_or(|(_, k, _)| k != key))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// A `KEY=value` line as (`export ` or nothing, key, raw value). Comments and anything else
/// are None.
fn parse(line: &str) -> Option<(&'static str, &str, &str)> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return None;
    }
    let (export, rest) = match trimmed.strip_prefix("export ") {
        Some(rest) => ("export ", rest.trim_start()),
        None => ("", trimmed),
    };
    let (key, value) = rest.split_once('=')?;
    let key = key.trim();
    (!key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .then_some((export, key, value))
}

/// Undo `quote`, and read what a person wrote the way compose does. Single quotes are
/// literal. Inside double quotes a backslash escapes whatever follows it. Bare, a ` #`
/// starts a comment. Outside single quotes `$$` is one `$`.
fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
        let bare = trimmed
            .split_once(" #")
            .map_or(trimmed, |(v, _)| v)
            .trim_end();
        return bare.replace("$$", "$");
    }
    let mut out = String::new();
    let mut chars = trimmed[1..trimmed.len() - 1].chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push(chars.next().unwrap_or('\\')),
            '$' if chars.peek() == Some(&'$') => {
                chars.next();
                out.push('$');
            }
            other => out.push(other),
        }
    }
    out
}

/// Quote anything that would not survive a bare `KEY=value` line. dotenv rejects an
/// unquoted value with a space in it, and a character name is two words. Compose expands
/// `$` inside double quotes, so a value with one goes in single quotes, which it leaves
/// alone; one with a single quote in it as well has the `$` escaped instead.
fn quote(value: &str) -> String {
    let plain = !value.is_empty()
        && !value
            .chars()
            .any(|c| c.is_whitespace() || "|#\"'$`\\".contains(c));
    if plain {
        value.to_string()
    } else if value.contains('$') && !value.contains('\'') {
        format!("'{value}'")
    } else {
        format!(
            "\"{}\"",
            value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn replaces_a_value_and_leaves_the_comments_alone() {
        let current = "# what this is for\nWS_DOMAIN=old.example.com\nHTTP_PORT=80\n";
        let out = patch(current, &values(&[("WS_DOMAIN", "map.example.com")]));
        assert_eq!(
            out,
            "# what this is for\nWS_DOMAIN=map.example.com\nHTTP_PORT=80\n"
        );
    }

    // A character name is two words, and dotenv refuses a bare value with a space in it.
    #[test]
    fn quotes_anything_a_bare_line_would_not_survive() {
        let out = patch(
            "WS_CONTACT_NAME=\n",
            &values(&[("WS_CONTACT_NAME", "Nicolas Kion")]),
        );
        assert_eq!(out, "WS_CONTACT_NAME=\"Nicolas Kion\"\n");
    }

    #[test]
    fn escapes_a_quote_inside_a_value() {
        let out = patch("K=\n", &values(&[("K", "a\"b")]));
        assert_eq!(out, "K=\"a\\\"b\"\n");
        assert_eq!(get(&out, "K").as_deref(), Some("a\"b"));
    }

    #[test]
    fn appends_keys_the_file_does_not_have_yet() {
        let out = patch("EXISTING=1\n", &values(&[("NEW", "2")]));
        assert!(out.starts_with("EXISTING=1\n"));
        assert!(out.contains("# Added by wsctl\nNEW=2\n"));
    }

    #[test]
    fn reads_plain_and_quoted_values_and_ignores_comments() {
        let env = "# WS_DOMAIN=commented.example.com\nWS_DOMAIN=\"map.example.com\"\nPORT=80\n";
        assert_eq!(get(env, "WS_DOMAIN").as_deref(), Some("map.example.com"));
        assert_eq!(get(env, "PORT").as_deref(), Some("80"));
        assert_eq!(get(env, "ABSENT"), None);
    }

    #[test]
    fn removing_a_key_takes_the_whole_line() {
        let env = "A=1\nDISCORD_CLIENT_ID=abc\nB=2\n";
        assert_eq!(remove(env, "DISCORD_CLIENT_ID"), "A=1\nB=2\n");
    }

    // Compose reads the last of two, so that is the one that counts, and patching one has
    // to patch both or the file says two things.
    #[test]
    fn a_duplicate_key_is_read_last_and_patched_everywhere() {
        let env = "K=first\nK=last\n";
        assert_eq!(get(env, "K").as_deref(), Some("last"));
        assert_eq!(patch(env, &values(&[("K", "new")])), "K=new\nK=new\n");
        assert_eq!(remove(env, "K"), "");
    }

    #[test]
    fn an_exported_key_keeps_its_export() {
        let env = "export K=old\n";
        assert_eq!(get(env, "K").as_deref(), Some("old"));
        assert_eq!(patch(env, &values(&[("K", "new")])), "export K=new\n");
        assert_eq!(remove(env, "K"), "");
    }

    #[test]
    fn values_are_read_the_way_compose_reads_them() {
        let env = "A='p$ss\\w\"rd'\nB=\"p\\$ss it's\"\nC=bare # a comment\nD=\"x$$y\"\n";
        assert_eq!(get(env, "A").as_deref(), Some("p$ss\\w\"rd"));
        assert_eq!(get(env, "B").as_deref(), Some("p$ss it's"));
        assert_eq!(get(env, "C").as_deref(), Some("bare"));
        assert_eq!(get(env, "D").as_deref(), Some("x$y"));
    }

    // Inside double quotes compose would expand `$ss` to whatever that variable is, which
    // for a password is nothing.
    #[test]
    fn a_dollar_is_never_left_for_compose_to_expand() {
        assert_eq!(quote("pa$s word"), "'pa$s word'");
        assert_eq!(quote("it's $5"), "\"it's \\$5\"");
    }

    // Round-tripping is the property that matters: whatever is written can be read back.
    #[test]
    fn every_value_survives_a_round_trip() {
        for value in [
            "simple",
            "two words",
            "with#hash",
            "with$dollar",
            "both $ and '",
            "a $$ pair",
            "trailing \\",
            "with\"quote",
            "with\\backslash",
            "",
        ] {
            let out = patch("K=\n", &values(&[("K", value)]));
            assert_eq!(get(&out, "K").as_deref(), Some(value), "value {value:?}");
        }
    }
}
