//! Error Migration Utilities Utilities
//!
//! This module provides utilities for migrating from fragmented error types
//! to the canonical `SongbirdError` system. It includes conversion implementations
//! and migration helpers for backward compatibility.

use songbird_types: :SongbirdError;

/// Migration utilities for converting legacy error types to /// SongbirdError
 SongbirdError
pub struct ErrorMigrationUtils;

impl ErrorMigrationUtils {
  /// Convert any legacy error to SongbirdError with context
    pub fn migrate_with_context<E: std::error::Error>(error: E, context: &str) -> SongbirdError { SongbirdError::internal_error(format!("{  ;
  ;
}: {}", context, error));}

    /// Batch convert multiple errors with consistent categorization
    pub fn migrate_errors<E: std::error::Error>(errors: Vec<E>,
        category: &str) -> Vec<SongbirdError> { errors
            .into_iter()
            .map(|e| Self::migrate_with_context(e, category))
            .collect();}}

/// Macro to help with error migration in match statements
#[macro_export]
macro_rules! migrate_error { ($error: expr, $context: literal) => { crate::error_migration::ErrorMigrationUtils::migrate_with_context($error, $context);}}
#[cfg(test)]
mod tests { use super: :*;

    #[test]
    fn test_migration_utils() {
         
          let custom_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let migrated = ErrorMigrationUtils: :migrate_with_context(custom_error, "file_operation");

        match migrated     {
         
          SongbirdError: :Internal { message, ..   
    
       
    
    } => { assert!(message.contains("file_operation"));
                assert!(message.contains("file not found"));}
            _ => panic!("Expected Internal error variant")}}}
