//! The identity every outbound request carries.
//!
//! CCP, zKillboard and EVE Ref all ask that a client says who is running it and how to
//! reach them: an anonymous client gets rate limited, and a misbehaving one that nobody can
//! contact gets blocked. That contact has to be whoever deployed this copy, not whoever
//! wrote it, so it comes from the configuration and the process refuses to run without it.

use std::sync::OnceLock;

use crate::config::Contact;

static AGENT: OnceLock<String> = OnceLock::new();

/// Set the identity from the configuration. Called once at startup, before anything builds
/// a client; a second call keeps the first.
pub fn init(contact: &Contact) {
    let _ = AGENT.set(agent(contact));
}

/// `wormholesystems/0.1.0 (Some Pilot; someone@example.com; +https://github.com/wormholesystems/wormholesystems)`
///
/// Without [`init`] (a test, a tool built on the library) the contact is read from the
/// environment the way the configuration reads it. Panics if it is not there. That is
/// deliberate: a deploy that talks to ESI anonymously is one that gets throttled and then
/// banned, and failing at the first request with a message naming the two variables is
/// kinder than finding out from CCP.
pub fn get() -> &'static str {
    AGENT.get_or_init(|| match Contact::from_env() {
        Ok(contact) => agent(&contact),
        Err(_) => panic!(
            "{} and {} must be set: every request to ESI, zKillboard and EVE Ref carries \
             them so those services can tell who is running this and reach them. Use the \
             in-game name of the character who administers this install and an address you \
             read.",
            Contact::NAME_VAR,
            Contact::EMAIL_VAR,
        ),
    })
}

fn agent(contact: &Contact) -> String {
    format!(
        "wormholesystems/{} ({}; {}; +https://github.com/wormholesystems/wormholesystems)",
        env!("CARGO_PKG_VERSION"),
        contact.name,
        contact.email,
    )
}

/// The client that identifies itself. Every outbound request in the app goes through this
/// one or the blocking one below, so there is one place the identity can be wrong. Shared:
/// a clone is a handle onto the same connection pool, so asking for it per request costs
/// nothing and reuses connections. A caller that wants a tighter deadline sets one on the
/// request.
///
/// Without timeouts a connection that goes quiet hangs its request forever, and the pollers
/// awaiting it with it. The read timeout resets on every chunk, so large bodies still work.
pub fn client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent(get())
                .connect_timeout(std::time::Duration::from_secs(10))
                .read_timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("http client")
        })
        .clone()
}

pub fn blocking_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(get())
        .build()
        .expect("http client")
}

#[cfg(test)]
mod tests {
    /// The format is what the services read, so it is worth pinning: product/version first,
    /// then who to contact.
    #[test]
    fn names_the_product_and_the_contact() {
        let agent = super::agent(&crate::config::Contact {
            name: "Zvi Sarok".into(),
            email: "admin@example.com".into(),
        });
        assert!(agent.starts_with("wormholesystems/"), "{agent}");
        assert!(agent.contains("(Zvi Sarok; admin@example.com;"), "{agent}");
        assert!(
            agent.contains("+https://github.com/wormholesystems/wormholesystems"),
            "{agent}"
        );
    }
}
