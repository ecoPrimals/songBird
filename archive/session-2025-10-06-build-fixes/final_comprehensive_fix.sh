#!/bin/bash
# Final comprehensive fix for ALL remaining syntax patterns

cd /home/eastgate/Development/ecoPrimals/songbird

echo "Fixing all .insert() double closing parens..."
find crates -name "*.rs" -type f -exec perl -i -pe 's/(\.insert\([^;]+\))([);])/$1$2/g unless /\)\);/' {} \;

echo "Fixing all .to_string(); patterns..."
find crates -name "*.rs" -type f -exec perl -i -pe 's/\.to_string\(\);$/\.to_string\(\)\);/g if /insert.*to_string\(\);$/' {} \;

echo "Fixing remaining edge cases..."
# Fix patterns like: .insert(key, value));
find crates -name "*.rs" -type f -exec sed -i 's/\(\.insert([^)]*)\));$/\1);/g' {} \;

# Fix patterns like: ], when should be ),
find crates -name "*.rs" -type f -exec sed -i 's/], (/, (/g' {} \;

echo "Complete!"

