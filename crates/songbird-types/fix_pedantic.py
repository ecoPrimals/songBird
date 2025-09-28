#!/usr/bin/env python3
import re
import os

# Add Copy derives to eligible structs
files = ["src/unified_constants.rs", "src/generated_unified_constants.rs"]
for file_path in files:
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Add Copy to ConstantsSet
        content = re.sub(r'pub struct ConstantsSet \{', '#[derive(Debug, Clone, Copy)]\npub struct ConstantsSet {', content)
        
        # Add Debug and Copy to UnifiedConstantsFactory
        content = re.sub(r'pub struct UnifiedConstantsFactory;', '#[derive(Debug, Clone, Copy)]\npub struct UnifiedConstantsFactory;', content)
        
        with open(file_path, 'w') as f:
            f.write(content)

print("✅ Fixed Copy derives and Debug implementations")
