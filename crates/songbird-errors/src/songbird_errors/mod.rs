pub mod conversions;
pub mod core;
pub mod specific;
pub mod constructors;

// Re-export everything from the modules
pub use specific::*;
pub use constructors::*;

// Re-export the main error type and Result
pub use core::SongbirdError;
pub type Result<T> = std::result::Result<T, SongbirdError>;
