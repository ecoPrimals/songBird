pub mod conversions;
pub mod specific;
pub mod constructors;

// Re-export everything from the modules
pub use specific::*;
pub use constructors::*;

// Re-export the canonical error type from songbird-types
pub use songbird_types::errors::{SongbirdError, SongbirdResult};
pub type Result<T> = SongbirdResult<T>;
