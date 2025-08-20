#!/bin/bash
# Fix all Result<T, SongbirdError> patterns that should just be Result<T>

echo "🔧 Fixing all Result type signatures..."

# Fix Result<T, songbird_errors::SongbirdError> to Result<T>
find crates -name "*.rs" -type f -exec sed -i 's/Result<\([^,>]*\), songbird_errors::SongbirdError>/Result<\1>/g' {} \;

# Fix Result<T, SongbirdError> to Result<T> (without namespace)
find crates -name "*.rs" -type f -exec sed -i 's/Result<\([^,>]*\), SongbirdError>/Result<\1>/g' {} \;

echo "✅ All Result type signatures standardized!"
