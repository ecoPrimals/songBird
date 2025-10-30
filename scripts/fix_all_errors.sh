#!/bin/bash
# Comprehensive error pattern fixer

cd "$(dirname "$0")/.."

# Fix patterns in biomeos/client.rs
perl -i -pe '
s/SongbirdError::Service\(Box::new\(\s*ServiceError::new\(\s*"([^"]+)",\s*format!\(([^)]+)\)\s*\),?\s*\)\)/SongbirdError::Service {\n                        message: format!($2),\n                        service_name: Some("$1".to_string()),\n                        suggestion: None,\n                    }/gs;
s/SongbirdError::Network\(Box::new\(\s*NetworkError::new\(([^)]+)\)\s*\)\)/SongbirdError::Network {\n                    message: $1,\n                    operation: None,\n                    suggestion: None,\n                }/gs;
' crates/songbird-core/src/biomeos/client.rs

# Fix substrate/clients.rs
perl -i -pe '
s/SongbirdError::Service\(Box::new\(ServiceError::new\([^)]+\)\)\)/SongbirdError::Service { message: "Service error".to_string(), service_name: None, suggestion: None }/gs;
s/SongbirdError::Network\(Box::new\(NetworkError::new\([^)]+\)\)\)/SongbirdError::Network { message: "Network error".to_string(), operation: None, suggestion: None }/gs;
' crates/songbird-core/src/substrate/clients.rs

echo "Fixed error patterns"

