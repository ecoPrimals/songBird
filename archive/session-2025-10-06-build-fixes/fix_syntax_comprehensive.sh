#!/bin/bash
# Comprehensive syntax fix script for Songbird

cd /home/eastgate/Development/ecoPrimals/songbird

# Fix missing closing parentheses after .to_string()
find crates/songbird-types -name "*.rs" -exec sed -i 's/\.to_string()$/\.to_string()/g' {} \;
find crates/songbird-types -name "*.rs" -exec sed -i 's/unwrap_or_else(|_| "\([^"]*\)"\.to_string($/unwrap_or_else(|_| "\1".to_string())/g' {} \;

# Fix missing commas after .to_string() when followed by a field
# This regex looks for .to_string() followed by newline and whitespace, then a word (field name)
find crates/songbird-types -name "*.rs" -type f -print0 | xargs -0 perl -i -pe 's/\.to_string\(\)\s*\n(\s+)([a-z_][a-z0-9_]*:)/\.to_string\(\),\n$1$2/g'

echo "Syntax fixes applied"

