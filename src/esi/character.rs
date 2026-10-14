use serde::Deserialize;

use super::{EsiClient, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterLocation {
    pub solar_system_id: i64,
    /// Set only when docked in a station.
    pub station_id: Option<i64>,
    /// Set only when docked in a structure.
    pub structure_id: Option<i64>,
}

impl CharacterLocation {
    pub fn is_docked(&self) -> bool {
        self.station_id.is_some() || self.structure_id.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterShip {
    pub ship_name: String,
    pub ship_type_id: i64,
    /// The hull itself. Two Lokis are one type and two items, which is what tells a reship
    /// apart from a pilot who has been sitting in the same ship for an hour.
    pub ship_item_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterOnline {
    pub online: bool,
    pub last_login: Option<String>,
    pub last_logout: Option<String>,
    pub logins: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterPublic {
    pub name: String,
    pub corporation_id: i64,
    pub alliance_id: Option<i64>,
    pub faction_id: Option<i64>,
    pub birthday: String,
    pub gender: String,
    pub race_id: i64,
    pub bloodline_id: i64,
    pub security_status: Option<f64>,
    pub description: Option<String>,
}

impl EsiClient {
    pub async fn character_location(
        &self,
        token: &str,
        character_id: i64,
    ) -> Result<CharacterLocation> {
        self.get_json_polled(&format!("/characters/{character_id}/location"), token)
            .await
    }

    pub async fn character_ship(&self, token: &str, character_id: i64) -> Result<CharacterShip> {
        self.get_json_polled(&format!("/characters/{character_id}/ship"), token)
            .await
    }

    pub async fn character_online(
        &self,
        token: &str,
        character_id: i64,
    ) -> Result<CharacterOnline> {
        self.get_json_polled(&format!("/characters/{character_id}/online"), token)
            .await
    }

    pub async fn character_public(&self, character_id: i64) -> Result<CharacterPublic> {
        self.get_json(&format!("/characters/{character_id}"), None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docked_detection() {
        let in_space = CharacterLocation {
            solar_system_id: 30000142,
            station_id: None,
            structure_id: None,
        };
        let at_station = CharacterLocation {
            solar_system_id: 30000142,
            station_id: Some(60003760),
            structure_id: None,
        };
        let at_structure = CharacterLocation {
            solar_system_id: 30000142,
            station_id: None,
            structure_id: Some(1234),
        };
        assert!(!in_space.is_docked());
        assert!(at_station.is_docked());
        assert!(at_structure.is_docked());
    }
}
