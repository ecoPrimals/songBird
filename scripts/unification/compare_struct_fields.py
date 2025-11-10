#!/usr/bin/env python3
"""
Field-Level Struct Comparison Tool
Identifies TRUE duplicates (identical fields) vs domain-specific variants
"""

import os
import re
import sys
import hashlib
from collections import defaultdict
from pathlib import Path
from datetime import datetime

def extract_struct_definition(file_path, struct_name):
    """Extract the complete struct definition including fields"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Pattern to match struct definition
        # Matches: pub struct Name { ... } or pub struct Name<T> { ... }
        pattern = rf'\bpub\s+struct\s+{re.escape(struct_name)}\b[^{{]*\{{([^}}]*)\}}'
        
        match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
        if match:
            return match.group(1)
        return None
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return None

def extract_fields(struct_body):
    """Extract field definitions from struct body"""
    if not struct_body:
        return []
    
    fields = []
    # Match lines like: pub field_name: Type,
    # Handles various forms: pub field: Type, field: Type, pub field: Vec<Type>,
    for line in struct_body.split('\n'):
        line = line.strip()
        # Skip empty lines, comments, attributes
        if not line or line.startswith('//') or line.startswith('#[') or line.startswith('///'):
            continue
        
        # Match field declarations: pub? name: type,?
        match = re.match(r'(pub\s+)?([a-z_][a-z0-9_]*)\s*:\s*(.+?)(?:,\s*)?$', line)
        if match:
            field_name = match.group(2)
            field_type = match.group(3).strip().rstrip(',')
            fields.append((field_name, field_type))
    
    return sorted(fields)  # Sort for consistent comparison

def field_signature(fields):
    """Generate a signature hash for field comparison"""
    field_str = '|'.join([f"{name}:{typ}" for name, typ in fields])
    return hashlib.md5(field_str.encode()).hexdigest()

def find_struct_files(project_root, struct_name):
    """Find all files containing a struct definition"""
    crates_dir = Path(project_root) / 'crates'
    files = []
    
    for rs_file in crates_dir.rglob('*.rs'):
        try:
            with open(rs_file, 'r', encoding='utf-8') as f:
                content = f.read()
                if re.search(rf'\bstruct\s+{re.escape(struct_name)}\b', content):
                    files.append(str(rs_file.relative_to(project_root)))
        except Exception:
            continue
    
    return sorted(files)

def compare_single_struct(project_root, struct_name):
    """Compare all definitions of a single struct"""
    print(f"\n🔍 Analyzing: {struct_name}")
    print("=" * 60)
    
    files = find_struct_files(project_root, struct_name)
    
    if not files:
        print(f"❌ No definitions found for {struct_name}")
        return None
    
    print(f"Found {len(files)} definitions\n")
    
    # Extract fields from each definition
    definitions = {}
    for file_path in files:
        full_path = Path(project_root) / file_path
        struct_body = extract_struct_definition(full_path, struct_name)
        if struct_body:
            fields = extract_fields(struct_body)
            if fields:
                definitions[file_path] = fields
    
    if not definitions:
        print(f"⚠️  Could not extract fields from any definitions")
        return None
    
    # Group by field signature
    sig_to_files = defaultdict(list)
    for file_path, fields in definitions.items():
        sig = field_signature(fields)
        sig_to_files[sig].append((file_path, fields))
    
    num_variants = len(sig_to_files)
    
    result = {
        'struct_name': struct_name,
        'total_definitions': len(definitions),
        'num_variants': num_variants,
        'is_true_duplicate': num_variants == 1,
        'variants': []
    }
    
    if num_variants == 1:
        print(f"✅ TRUE DUPLICATE - All {len(definitions)} definitions are IDENTICAL\n")
        print("Locations:")
        for file_path, fields in list(sig_to_files.values())[0]:
            print(f"  - {file_path}")
        
        print("\nFields:")
        fields = list(sig_to_files.values())[0][0][1]
        for name, typ in fields:
            print(f"  {name}: {typ}")
        
        print("\n✅ CONSOLIDATION: SAFE - Replace all with re-exports to canonical\n")
        
        result['variants'].append({
            'files': [f for f, _ in list(sig_to_files.values())[0]],
            'fields': fields
        })
    else:
        print(f"⚠️  DOMAIN-SPECIFIC VARIANTS - {num_variants} different implementations\n")
        
        for idx, (sig, file_fields_list) in enumerate(sig_to_files.items(), 1):
            print(f"Variant {idx}:")
            for file_path, _ in file_fields_list:
                print(f"  - {file_path}")
            
            print("\n  Fields:")
            fields = file_fields_list[0][1]
            for name, typ in fields:
                print(f"    {name}: {typ}")
            print()
            
            result['variants'].append({
                'files': [f for f, _ in file_fields_list],
                'fields': fields
            })
        
        print("⚠️  CONSOLIDATION: REVIEW NEEDED - Determine if variants are legitimate\n")
    
    return result

def analyze_all_duplicates(project_root):
    """Analyze all identified duplicate config names"""
    print("🔍 Field-Level Struct Comparison Tool")
    print("=" * 60)
    print("\nAnalyzing all identified duplicate config names...\n")
    
    # Read duplicate names from previous report
    report_path = Path(project_root) / 'DUPLICATE_DEFINITIONS_REPORT.md'
    if not report_path.exists():
        print("❌ DUPLICATE_DEFINITIONS_REPORT.md not found.")
        print("   Run 04_find_duplicates.sh first.")
        sys.exit(1)
    
    # Extract struct names from report (format: "  19 HealthCheckConfig")
    struct_names = set()
    with open(report_path, 'r') as f:
        in_summary = False
        for line in f:
            if '## Config Struct Duplicates' in line or '### Summary' in line:
                in_summary = True
                continue
            if in_summary:
                # Match lines like "     19 HealthCheckConfig"
                match = re.match(r'\s+\d+\s+(\w+)', line)
                if match:
                    struct_names.add(match.group(1))
                elif line.startswith('##') and 'Summary' not in line:
                    # Hit next section
                    break
    
    struct_names = sorted(struct_names)
    print(f"Found {len(struct_names)} duplicate config names to analyze\n")
    
    results = []
    true_duplicates = []
    domain_variants = []
    
    for struct_name in struct_names:
        result = compare_single_struct(project_root, struct_name)
        if result:
            results.append(result)
            if result['is_true_duplicate']:
                true_duplicates.append(result)
            else:
                domain_variants.append(result)
        print("-" * 60)
    
    # Generate report
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    output_file = f"FIELD_COMPARISON_REPORT_{timestamp}.md"
    
    with open(Path(project_root) / output_file, 'w') as f:
        f.write("# Field-Level Struct Comparison Report\n\n")
        f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        
        f.write("## Executive Summary\n\n")
        f.write(f"- **Total Analyzed**: {len(results)} structs\n")
        f.write(f"- **✅ True Duplicates**: {len(true_duplicates)} (safe to consolidate)\n")
        f.write(f"- **⚠️  Domain Variants**: {len(domain_variants)} (need review)\n")
        if len(results) > 0:
            f.write(f"- **Consolidation Rate**: {len(true_duplicates) * 100 // len(results)}% can be safely consolidated\n")
        f.write("\n---\n\n")
        
        if true_duplicates:
            f.write("## ✅ True Duplicates (Safe to Consolidate)\n\n")
            for result in true_duplicates:
                f.write(f"### {result['struct_name']}\n\n")
                f.write(f"**{result['total_definitions']} identical definitions**\n\n")
                f.write("**Locations:**\n")
                for file_path in result['variants'][0]['files']:
                    f.write(f"- `{file_path}`\n")
                f.write("\n**Fields:**\n```rust\n")
                for name, typ in result['variants'][0]['fields']:
                    f.write(f"{name}: {typ}\n")
                f.write("```\n\n")
                f.write("**Action:** Replace all with re-exports to canonical location\n\n")
                f.write("---\n\n")
        
        if domain_variants:
            f.write("## ⚠️  Domain-Specific Variants (Review Needed)\n\n")
            for result in domain_variants:
                f.write(f"### {result['struct_name']}\n\n")
                f.write(f"**{result['num_variants']} different implementations** across {result['total_definitions']} definitions\n\n")
                
                for idx, variant in enumerate(result['variants'], 1):
                    f.write(f"**Variant {idx}:**\n")
                    for file_path in variant['files']:
                        f.write(f"- `{file_path}`\n")
                    f.write("\n**Fields:**\n```rust\n")
                    for name, typ in variant['fields']:
                        f.write(f"{name}: {typ}\n")
                    f.write("```\n\n")
                
                f.write("**Action:** Review to determine if:\n")
                f.write("- Variants should be unified (accidental divergence)\n")
                f.write("- Variants should be renamed for clarity (legitimate differences)\n\n")
                f.write("---\n\n")
        
        f.write("## Recommendations\n\n")
        f.write("### Immediate Actions (True Duplicates)\n")
        f.write(f"1. Consolidate the {len(true_duplicates)} TRUE duplicates marked with ✅\n")
        f.write("2. Each consolidation: ~30 minutes (proven process)\n")
        f.write("3. Replace all occurrences with re-exports to canonical\n\n")
        
        f.write("### Review Actions (Domain Variants)\n")
        f.write(f"1. Review each of the {len(domain_variants)} domain-specific variants\n")
        f.write("2. Determine if differences are legitimate or accidental\n")
        f.write("3. Either:\n")
        f.write("   - Unify if differences are accidental\n")
        f.write("   - Rename for clarity if legitimate (e.g., NetworkConfig → EdgeNetworkConfig)\n\n")
    
    print(f"\n✅ Analysis complete!")
    print(f"📄 Report: {output_file}")
    print(f"\nResults:")
    print(f"  ✅ True Duplicates: {len(true_duplicates)}")
    print(f"  ⚠️  Domain Variants: {len(domain_variants)}")
    print(f"  📊 Safe Consolidation Rate: {len(true_duplicates) * 100 // len(results) if results else 0}%")

def main():
    project_root = Path(__file__).parent.parent.parent.absolute()
    
    if len(sys.argv) > 1:
        # Single struct analysis
        struct_name = sys.argv[1]
        compare_single_struct(str(project_root), struct_name)
    else:
        # Full analysis
        analyze_all_duplicates(str(project_root))

if __name__ == '__main__':
    main()

