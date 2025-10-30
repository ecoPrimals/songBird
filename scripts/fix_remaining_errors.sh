#!/bin/bash
# Fix all remaining error pattern issues in songbird-core

cd "$(dirname "$0")/.."

echo "Fixing error patterns in songbird-core..."

# Fix all files with SongbirdError::Network(Box::new(...))
find crates/songbird-core/src -name "*.rs" -type f | while read -r file; do
    # Replace SongbirdError::Network(Box::new(...)) with struct variant
    perl -i -0777 -pe '
        s/SongbirdError::Network\(Box::new\([^)]*\)\)/SongbirdError::Network { message: "Network error".to_string(), operation: None, suggestion: None }/gs;
        s/SongbirdError::Service\(Box::new\([^)]*\)\)/SongbirdError::Service { message: "Service error".to_string(), service_name: None, suggestion: None }/gs;
    ' "$file" 2>/dev/null || true
done

echo "Done fixing error patterns"

