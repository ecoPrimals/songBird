#!/bin/bash
# Comprehensive parenthesis fix for all Arc::new(RwLock::new patterns

cd /home/eastgate/Development/ecoPrimals/songbird

find . -name "*.rs" -not -path "./target/*" -not -path "./archive/*" -not -path "./experiments/target/*" -type f | while read file; do
    # Fix Arc::new(RwLock::new(HashMap::new()), -> Arc::new(RwLock::new(HashMap::new())),
    perl -i -pe 's/Arc::new\(RwLock::new\(HashMap::new\(\),/Arc::new(RwLock::new(HashMap::new())),/g' "$file"
    
    # Fix Arc::new(RwLock::new(Vec::new()), -> Arc::new(RwLock::new(Vec::new())),
    perl -i -pe 's/Arc::new\(RwLock::new\(Vec::new\(\),/Arc::new(RwLock::new(Vec::new())),/g' "$file"
    
    # Fix Arc::new(RwLock::new(HashMap::new() -> Arc::new(RwLock::new(HashMap::new()))
    perl -i -pe 's/Arc::new\(RwLock::new\(HashMap::new\(\)([^))])/Arc::new(RwLock::new(HashMap::new()))$1/g' "$file"
    
    # Fix Arc::new(RwLock::new(Vec::new() -> Arc::new(RwLock::new(Vec::new()))  
    perl -i -pe 's/Arc::new\(RwLock::new\(Vec::new\(\)([^))])/Arc::new(RwLock::new(Vec::new()))$1/g' "$file"
done

echo "✓ Comprehensive Arc/RwLock fixes applied"

