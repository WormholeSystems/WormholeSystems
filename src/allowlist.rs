//! Instance-wide access rules: which EVE characters, corporations and alliances an
//! operator lets in, and which of them may create maps.
//!
//! Both rules have the same shape, a list of ids where an empty list means everyone, so
//! an install that sets nothing behaves as if the rule did not exist.

use std::sync::Arc;

/// A list of EVE character, corporation or alliance ids. A character matches when any of
/// its three ids is on the list; an empty list matches everybody.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AffiliationAllowlist {
    ids: Arc<[i64]>,
}

/// What the parse refused: the entry that is not an id. Silently dropping it would be
/// worse than failing to start, since a list whose only entry is mistyped reads as empty,
/// and empty means everyone.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not an EVE id")]
pub struct InvalidId(pub String);

impl AffiliationAllowlist {
    pub fn new(ids: impl IntoIterator<Item = i64>) -> Self {
        AffiliationAllowlist {
            ids: ids.into_iter().collect(),
        }
    }

    /// Parse a comma-separated list. Blank entries (a trailing comma, a doubled one) are
    /// skipped; anything else that is not a positive number is an error.
    pub fn parse(value: &str) -> Result<Self, InvalidId> {
        let mut ids = Vec::new();
        for entry in value.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            match entry.parse::<i64>() {
                Ok(id) if id > 0 => ids.push(id),
                _ => return Err(InvalidId(entry.to_string())),
            }
        }
        Ok(AffiliationAllowlist::new(ids))
    }

    /// Whether anything is configured. When not, callers can skip resolving the character
    /// at all.
    pub fn is_enforced(&self) -> bool {
        !self.ids.is_empty()
    }

    /// Whether a character with these ids is let through.
    pub fn allows(
        &self,
        character_id: i64,
        corporation_id: Option<i64>,
        alliance_id: Option<i64>,
    ) -> bool {
        if !self.is_enforced() {
            return true;
        }
        [Some(character_id), corporation_id, alliance_id]
            .into_iter()
            .flatten()
            .any(|id| self.ids.contains(&id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(ids: &[i64]) -> AffiliationAllowlist {
        AffiliationAllowlist::new(ids.iter().copied())
    }

    #[test]
    fn an_empty_list_lets_everyone_through() {
        assert!(!list(&[]).is_enforced());
        assert!(list(&[]).allows(42, Some(2000), Some(3000)));
        assert!(list(&[123]).is_enforced());
    }

    #[test]
    fn any_of_the_three_ids_matches() {
        assert!(list(&[42]).allows(42, Some(2000), Some(3000)));
        assert!(list(&[2000]).allows(42, Some(2000), Some(3000)));
        assert!(list(&[3000]).allows(42, Some(2000), Some(3000)));
        assert!(list(&[111, 2000, 999]).allows(42, Some(2000), Some(3000)));
    }

    #[test]
    fn nothing_matching_is_refused() {
        assert!(!list(&[111, 222]).allows(42, Some(2000), Some(3000)));
        assert!(!list(&[3000]).allows(42, Some(2000), None));
        assert!(!list(&[2000]).allows(42, None, None));
    }

    #[test]
    fn parses_a_comma_separated_list() {
        assert_eq!(
            AffiliationAllowlist::parse(" 99000001, 98000002 ,,90000003,").unwrap(),
            list(&[99000001, 98000002, 90000003])
        );
        assert_eq!(AffiliationAllowlist::parse("").unwrap(), list(&[]));
        assert_eq!(AffiliationAllowlist::parse(" , ").unwrap(), list(&[]));
    }

    // A typo must not quietly turn a closed instance into an open one.
    #[test]
    fn an_entry_that_is_not_an_id_is_an_error() {
        assert_eq!(
            AffiliationAllowlist::parse("99000001,Goonswarm"),
            Err(InvalidId("Goonswarm".into()))
        );
        assert!(AffiliationAllowlist::parse("-5").is_err());
        assert!(AffiliationAllowlist::parse("0").is_err());
    }
}
