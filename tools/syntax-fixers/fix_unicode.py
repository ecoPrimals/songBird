#!/usr/bin/env python3
"""Fix all Unicode quotes and special characters."""

with open('crates/songbird-primal-sdk/src/capability_orchestrator.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Check line 325 (index 324)
if len(lines) >= 325:
    line = lines[324]
    print(f"Line 325: {repr(line)}")
    print(f"Bytes: {line.encode('utf-8').hex()}")
    
    # Find all non-ASCII characters
    for i, ch in enumerate(line):
        if ord(ch) > 127:
            print(f"  Position {i}: {repr(ch)} (U+{ord(ch):04X})")

