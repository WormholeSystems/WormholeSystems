use super::{EsiClient, Result};

impl EsiClient {
    /// `destination_id` is a solar system, station, or structure id.
    pub async fn set_waypoint(
        &self,
        token: &str,
        destination_id: i64,
        add_to_beginning: bool,
        clear_other_waypoints: bool,
    ) -> Result<()> {
        // Values are plain ints/bools, so no escaping is needed.
        let path = format!(
            "/ui/autopilot/waypoint?destination_id={destination_id}\
             &add_to_beginning={add_to_beginning}&clear_other_waypoints={clear_other_waypoints}"
        );
        // The waypoint endpoint returns an empty body, so we just check the status.
        self.send_checked(
            self.request(reqwest::Method::POST, &path, Some(token)),
            Some(token),
        )
        .await?;
        Ok(())
    }
}
