//! Network connection to Tor relays
//!
//! Handles TCP + TLS connections and the Tor link protocol.

mod link;
mod tls;

pub use link::TorConnection;
pub use tls::TlsConnector;
