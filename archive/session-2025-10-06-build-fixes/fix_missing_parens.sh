#!/bin/bash
# Fix all missing closing parentheses from .to_string() and similar patterns

echo "🔧 Fixing missing closing parentheses..."

# Fix .to_string(); -> .to_string());
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.to_string();$/\.to_string());/g'

# Fix .insert(key, value); -> .insert(key, value));
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/context\.insert(\([^,]*\), \([^)]*\));$/context.insert(\1, \2));/g'

# Fix DomainError patterns
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/DomainError::\([A-Za-z]*\)("\([^"]*\)"\.to_string();$/DomainError::\1("\2".to_string());/g'

# Fix Some patterns
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/Some(&"\([^"]*\)"\.to_string();$/Some(\&"\1".to_string());/g'

# Fix *var = Some patterns  
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/= Some(\([^)]*\)\.to_string();$/= Some(\1.to_string());/g'

echo "✅ Fixed patterns. Building..."
cargo build --lib -p songbird-canonical -p songbird-errors 2>&1 | tail -10

