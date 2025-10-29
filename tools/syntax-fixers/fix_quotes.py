#!/usr/bin/env python3
"""Fix Unicode quotes in Rust source files."""

with open('crates/songbird-primal-sdk/src/capability_orchestrator.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace various Unicode quotes with ASCII quotes
replacements = [
    ('\u2018', "'"),  # LEFT SINGLE QUOTATION MARK
    ('\u2019', "'"),  # RIGHT SINGLE QUOTATION MARK
    ('\u201c', '"'),  # LEFT DOUBLE QUOTATION MARK
    ('\u201d', '"'),  # RIGHT DOUBLE QUOTATION MARK
]

for old, new in replacements:
    count = content.count(old)
    if count > 0:
        print(f"Replacing {count} instances of {repr(old)} with {repr(new)}")
        content = content.replace(old, new)

with open('crates/songbird-primal-sdk/src/capability_orchestrator.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print("Fixed Unicode quotes")

