#!/bin/bash
# Fix all the missing closing parentheses that were removed by the overly aggressive sed script

echo "🔧 Fixing missing closing parentheses..."

# Fix .clamp(0.0, 1.0); -> .clamp(0.0, 1.0));
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.clamp(0\.0, 1\.0);$/\.clamp(0.0, 1.0));/g'

# Fix .into(); -> .into());
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/prerequisite\.into();$/prerequisite.into());/g'
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/suggestion\.into();$/suggestion.into());/g'

# Fix .to_string(); -> .to_string()));
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/outcome\.to_string();$/outcome.to_string());/g'

# Fix write!(f patterns
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/write!(f, ", service: {}",/write!(f, ", service: {}",/g'

echo "✅ Fixed closing parentheses. Checking compilation..."
cargo build --workspace 2>&1 | grep "^error:" | wc -l

