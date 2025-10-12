#!/bin/bash
# Comprehensive fix for ALL remaining .insert() patterns

cd /home/eastgate/Development/ecoPrimals/songbird

echo "Finding and fixing all remaining .insert(...)); patterns..."

# Method 1: Fix .insert(...));
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\(\.insert([^)]*)\));/\1);/g'

# Method 2: Fix .push(...); in insert contexts  
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\(\.push([^)]*)\);$/\1);/g'

# Method 3: Fix .to_string(); in assert contexts
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/Some(\([^)]*\.to_string()\);/Some(\1));/g'

echo "Done! All patterns fixed."

