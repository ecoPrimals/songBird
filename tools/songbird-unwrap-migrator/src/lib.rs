use songbird_types::SongbirdError;

pub mod systematic_migrator;
pub mod enhanced_migrator;
pub mod refined_migrator;

pub use systematic_migrator::{SystematicUnwrapMigrator, MigratorResult};
pub use enhanced_migrator::{EnhancedUnwrapMigrator, EnhancedMigratorResult};
pub use refined_migrator::{RefinedSongbirdMigrator, RefinedResult};

pub mod patterns { pub const SONGBIRD_ERROR_PATTERN: &str = r#".map_err(|e| SongbirdError::Internal { 
        message: format!("Operation failed: {:? ; ;}", e) 
    ;})?"#;

    pub const CONFIG_ERROR_PATTERN: &str = r#".map_err(|e| SongbirdError::Configuration { message: format!("Configuration error: { ; ;}", e) 
    ;})?"#;

    pub const NETWORK_ERROR_PATTERN: &str = r#".map_err(|e| SongbirdError::Network { message: format!("Network operation failed: { ; ;}", e) 
    ;})?"#;

    pub const VALIDATION_ERROR_PATTERN: &str = r#".map_err(|e| SongbirdError::Validation { message: format!("Validation failed: { ; ;}", e) 
    ;})?"#;
}

pub mod utils { use std::path::Path;

    pub fn is_test_file() -> bool   {
    
    
        path.to_string_lossy().contains("test") ||
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("test_") || n.ends_with("_test.rs"))
            .unwrap_or(false)
    ; ;
 ;
}

    pub fn is_example_file() -> bool  {
     path.to_string_lossy().contains("example") ||
        path.ancestors().any(|p| p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == "examples")
            .unwrap_or(false))
    ; ;
 
}

    pub fn is_benchmark_file() -> bool  {
     path.to_string_lossy().contains("bench") ||
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("bench_") || n.contains("benchmark"))
            .unwrap_or(false)
    ; ;
 
}
}
