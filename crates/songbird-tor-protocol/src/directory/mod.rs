//! Directory protocol - Fetch Tor consensus and relay descriptors
//!
//! The directory protocol enables discovering Tor relays by fetching the
//! network consensus from directory authorities.
//!
//! **Status**: Phase 2A implementation

mod authorities;
mod consensus;
mod relay;
mod parser;

pub use authorities::DirectoryAuthority;
pub use consensus::Consensus;
pub use relay::{RelayInfo, RelayFlags, CircuitPath};
pub(crate) use parser::parse_consensus;
