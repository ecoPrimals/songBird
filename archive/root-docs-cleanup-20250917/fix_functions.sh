#!/bin/bash

# Fix malformed async function signatures
echo "Fixing malformed async function signatures..."

# Pattern 1: pub async fn name() -> \n becomes pub async fn name() -> Result<Type, Error> {
find crates/songbird-federation/src -name "*.rs" -exec sed -i '
/pub async fn.*-> *$/ {
    N
    s/pub async fn \([^(]*\)(\([^)]*\)) -> *\n *\([^{]*\)/pub async fn \1(\2) -> Result<Vec<String>, SongbirdError> {\n    \3/
}
' {} \;

echo "Fixed malformed function signatures in federation"
