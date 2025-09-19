#!/usr/bin/env python3
"""
Songbird Unwrap Migration Tool
Converts .unwrap() and .expect() calls to proper error handling patterns.
"""

import os
import re
import argparse
import sys
from pathlib import Path
from typing import List, Dict, Tuple

class UnwrapMigrator:
    def __init__(self, dry_run: bool = True):
        self.dry_run = dry_run
        self.stats = {
            'files_processed': 0,
            'unwrap_calls_found': 0,
            'expect_calls_found': 0,
            'migrations_applied': 0,
            'files_modified': 0
        }
        
        # Common patterns for unwrap/expect migration
        self.patterns = {
            # Simple unwrap patterns
            'config_unwrap': {
                'pattern': r'\.parse\(\)\.unwrap\(\)',
                'replacement': r'.parse().map_err(|e| SongbirdError::Configuration { message: format!("Parse error: {}", e) })?',
                'context': 'Configuration parsing'
            },
            'option_unwrap': {
                'pattern': r'\.get\([^)]+\)\.unwrap\(\)',
                'replacement': r'.get($1).ok_or_else(|| SongbirdError::NotFound { message: "Item not found".to_string() })?',
                'context': 'Option unwrapping'
            },
            'result_unwrap': {
                'pattern': r'\.unwrap\(\)',
                'replacement': r'.map_err(|e| SongbirdError::Internal { message: format!("Operation failed: {:?}", e) })?',
                'context': 'Result unwrapping'
            },
            
            # Expect patterns
            'config_expect': {
                'pattern': r'\.parse\(\)\.expect\("([^"]+)"\)',
                'replacement': r'.parse().map_err(|e| SongbirdError::Configuration { message: format!("$1: {}", e) })?',
                'context': 'Configuration parsing with message'
            },
            'general_expect': {
                'pattern': r'\.expect\("([^"]+)"\)',
                'replacement': r'.map_err(|e| SongbirdError::Internal { message: format!("$1: {:?}", e) })?',
                'context': 'General expect with message'
            }
        }
    
    def find_rust_files(self, directory: Path) -> List[Path]:
        """Find all Rust files in the directory tree."""
        rust_files = []
        for root, dirs, files in os.walk(directory):
            # Skip target directories and test files for now
            dirs[:] = [d for d in dirs if d not in ['target', '.git']]
            
            for file in files:
                if file.endswith('.rs'):
                    file_path = Path(root) / file
                    # Skip test files and examples for initial migration
                    if not any(part in str(file_path) for part in ['test', 'example', 'bench']):
                        rust_files.append(file_path)
        
        return rust_files
    
    def analyze_file(self, file_path: Path) -> Dict:
        """Analyze a file for unwrap/expect patterns."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            return {'error': f"Could not read file: {e}"}
        
        analysis = {
            'unwrap_count': len(re.findall(r'\.unwrap\(\)', content)),
            'expect_count': len(re.findall(r'\.expect\(', content)),
            'patterns_found': [],
            'content': content
        }
        
        # Find specific patterns
        for pattern_name, pattern_info in self.patterns.items():
            matches = re.finditer(pattern_info['pattern'], content)
            for match in matches:
                analysis['patterns_found'].append({
                    'name': pattern_name,
                    'match': match.group(),
                    'start': match.start(),
                    'end': match.end(),
                    'context': pattern_info['context']
                })
        
        return analysis
    
    def migrate_file(self, file_path: Path) -> bool:
        """Migrate unwrap/expect calls in a single file."""
        analysis = self.analyze_file(file_path)
        
        if 'error' in analysis:
            print(f"❌ Error processing {file_path}: {analysis['error']}")
            return False
        
        content = analysis['content']
        original_content = content
        modifications_made = 0
        
        # Apply migrations in reverse order to preserve positions
        patterns_found = sorted(analysis['patterns_found'], key=lambda x: x['start'], reverse=True)
        
        for pattern in patterns_found:
            pattern_info = self.patterns[pattern['name']]
            
            # Simple replacement for now - more sophisticated logic could be added
            if pattern['name'] in ['result_unwrap', 'general_expect']:
                # These are the safest to replace automatically
                old_text = pattern['match']
                new_text = pattern_info['replacement']
                
                if pattern['name'] == 'general_expect':
                    # Extract the expect message
                    expect_match = re.search(r'\.expect\("([^"]+)"\)', old_text)
                    if expect_match:
                        message = expect_match.group(1)
                        new_text = f'.map_err(|e| SongbirdError::Internal {{ message: format!("{message}: {{:?}}", e) }})?'
                
                content = content[:pattern['start']] + new_text + content[pattern['end']:]
                modifications_made += 1
        
        if modifications_made > 0 and not self.dry_run:
            try:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"✅ Modified {file_path}: {modifications_made} migrations applied")
                return True
            except Exception as e:
                print(f"❌ Error writing {file_path}: {e}")
                return False
        elif modifications_made > 0:
            print(f"🔍 Would modify {file_path}: {modifications_made} migrations")
            return True
        
        return False
    
    def run_migration(self, directory: Path) -> None:
        """Run the migration on all Rust files in the directory."""
        rust_files = self.find_rust_files(directory)
        print(f"📁 Found {len(rust_files)} Rust files to process")
        
        for file_path in rust_files:
            self.stats['files_processed'] += 1
            analysis = self.analyze_file(file_path)
            
            if 'error' in analysis:
                continue
            
            self.stats['unwrap_calls_found'] += analysis['unwrap_count']
            self.stats['expect_calls_found'] += analysis['expect_count']
            
            if analysis['unwrap_count'] > 0 or analysis['expect_count'] > 0:
                print(f"📄 {file_path}: {analysis['unwrap_count']} unwrap, {analysis['expect_count']} expect calls")
                
                if self.migrate_file(file_path):
                    self.stats['files_modified'] += 1
                    self.stats['migrations_applied'] += len(analysis['patterns_found'])
    
    def print_stats(self) -> None:
        """Print migration statistics."""
        print("\n📊 Migration Statistics:")
        print(f"   📁 Files processed: {self.stats['files_processed']}")
        print(f"   ⚠️  Unwrap calls found: {self.stats['unwrap_calls_found']}")
        print(f"   ⚠️  Expect calls found: {self.stats['expect_calls_found']}")
        print(f"   🔧 Migrations applied: {self.stats['migrations_applied']}")
        print(f"   📝 Files modified: {self.stats['files_modified']}")
        
        if self.dry_run:
            print("\n💡 This was a dry run. Use --apply to make changes permanent.")
        else:
            print("\n✅ Changes have been applied to your codebase")
            print("   🧪 Run tests to verify everything works correctly")

def main():
    parser = argparse.ArgumentParser(description='Songbird Unwrap Migration Tool')
    parser.add_argument('path', nargs='?', default='crates', 
                       help='Path to scan for Rust files (default: crates)')
    parser.add_argument('--dry-run', action='store_true', default=True,
                       help='Show what would be changed without applying changes')
    parser.add_argument('--apply', action='store_true',
                       help='Apply the migration changes to files')
    parser.add_argument('--stats-only', action='store_true',
                       help='Show statistics without performing migration')
    
    args = parser.parse_args()
    
    # Determine mode
    dry_run = not args.apply
    if args.stats_only:
        dry_run = True
    
    print(f"🔄 Songbird Unwrap Migration Tool")
    print(f"📂 Scanning: {args.path}")
    print(f"🔍 Mode: {'Statistics Only' if args.stats_only else 'Dry Run' if dry_run else 'Apply Changes'}")
    
    migrator = UnwrapMigrator(dry_run=dry_run)
    
    try:
        directory = Path(args.path)
        if not directory.exists():
            print(f"❌ Directory {directory} does not exist")
            sys.exit(1)
        
        migrator.run_migration(directory)
        migrator.print_stats()
        
    except KeyboardInterrupt:
        print("\n⚠️ Migration interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Migration failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 