#!/bin/bash
# Fix Result<T, E, E> patterns in federation crate

echo "🔧 Fixing Result type signatures in federation crate..."

# Fix the most common pattern: Result<T, SongbirdError, SongbirdError>
find crates/songbird-federation/src -name "*.rs" -type f -exec sed -i 's/Result<\([^,]*\), songbird_errors::SongbirdError, songbird_errors::SongbirdError>/Result<\1, songbird_errors::SongbirdError>/g' {} \;

# Fix patterns with SongbirdError without namespace
find crates/songbird-federation/src -name "*.rs" -type f -exec sed -i 's/Result<\([^,]*\), SongbirdError, songbird_errors::SongbirdError>/Result<\1, SongbirdError>/g' {} \;

# Fix mixed patterns
find crates/songbird-federation/src -name "*.rs" -type f -exec sed -i 's/Result<\([^,]*\), songbird_errors::SongbirdError, SongbirdError>/Result<\1, songbird_errors::SongbirdError>/g' {} \;

# Fix triple error patterns
find crates/songbird-federation/src -name "*.rs" -type f -exec sed -i 's/Result<\([^,]*\), SongbirdError, songbird_errors::SongbirdError, songbird_errors::SongbirdError>/Result<\1, SongbirdError>/g' {} \;

echo "✅ Result type signatures fixed!"
