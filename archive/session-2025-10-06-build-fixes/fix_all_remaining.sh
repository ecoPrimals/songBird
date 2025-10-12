#!/bin/bash
# Fix all remaining syntax errors comprehensively

cd /home/eastgate/Development/ecoPrimals/songbird/crates/songbird-types/src

# Fix missing commas after .to_string() in struct initialization
find . -name "*.rs" -exec perl -i -pe 's/\.to_string\(\)\s*\n(\s+)([a-z_][a-z0-9_]*:)/\.to_string\(\),\n$1$2/g' {} \;

# Fix specific patterns in primal.rs
sed -i 's/instance_id: "\([^"]*\)"\.to_string()$/instance_id: "\1".to_string(),/g' primal.rs
sed -i 's/version: "\([^"]*\)"\.to_string()$/version: "\1".to_string(),/g' primal.rs
sed -i 's/status: "\([^"]*\)"\.to_string()$/status: "\1".to_string(),/g' primal.rs

# Fix response.rs
sed -i 's/self\.metadata = Some(HashMap::new();/self.metadata = Some(HashMap::new());/g' response.rs
sed -i 's/Err("\([^"]*\)"\.to_string($/Err("\1".to_string())/g' response.rs
sed -i 's/Some("\([^"]*\)"\.to_string();/Some("\1".to_string()));/g' response.rs

# Fix assert_eq patterns
sed -i 's/assert_eq!()$/assert_eq!()/g' primal.rs
sed -i 's/\.to_string()\.to_string($/\.to_string()),/g' primal.rs
sed -i 's/"\([^"]*\)"\.to_string()$/"\1".to_string(),/g' primal.rs

echo "Comprehensive fixes applied"

