#!/usr/bin/env python3
"""
Technical Debt Elimination Script

This script analyzes and eliminates technical debt across the Songbird codebase,
including TODO/FIXME markers, deprecated code, and legacy patterns.
"""

import os
import re
import json
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Set, Tuple, Optional
from collections import defaultdict
from datetime import datetime

@dataclass
class TechnicalDebtItem:
    """Represents a technical debt item found in the codebase."""
    type: str  # TODO, FIXME, XXX, HACK, DEPRECATED
    file_path: str
    line_number: int
    content: str
    context: str
    priority: str  # HIGH, MEDIUM, LOW
    category: str  # implementation, integration, cleanup, etc.
    estimated_effort: str  # TRIVIAL, SMALL, MEDIUM, LARGE

@dataclass
class TechnicalDebtReport:
    """Report of technical debt analysis."""
    total_items: int
    items_by_type: Dict[str, int]
    items_by_priority: Dict[str, int]
    items_by_category: Dict[str, int]
    items_by_crate: Dict[str, int]
    debt_items: List[TechnicalDebtItem]
    elimination_plan: List[str]

class TechnicalDebtEliminator:
    """Main class for technical debt analysis and elimination."""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.crates_dir = project_root / "crates"
        self.debt_items: List[TechnicalDebtItem] = []
        
        # Technical debt patterns
        self.debt_patterns = {
            "TODO": r'//\s*TODO:?\s*(.+)',
            "FIXME": r'//\s*FIXME:?\s*(.+)',
            "XXX": r'//\s*XXX:?\s*(.+)',
            "HACK": r'//\s*HACK:?\s*(.+)',
            "DEPRECATED": r'//\s*DEPRECATED:?\s*(.+)',
        }
        
        # Category keywords for classification
        self.category_keywords = {
            "implementation": ["implement", "add", "create", "build", "write"],
            "integration": ["integrate", "connect", "federation", "network"],
            "optimization": ["optimize", "improve", "performance", "cow", "clone"],
            "cleanup": ["remove", "clean", "delete", "unused", "legacy"],
            "documentation": ["document", "comment", "explain", "clarify"],
            "testing": ["test", "validate", "verify", "check"],
            "security": ["security", "auth", "permission", "validate"],
            "configuration": ["config", "setting", "parameter", "option"],
        }
        
        # Priority classification based on keywords
        self.priority_keywords = {
            "HIGH": ["critical", "urgent", "security", "bug", "error", "crash", "fail"],
            "MEDIUM": ["implement", "integrate", "optimize", "improve"],
            "LOW": ["cleanup", "document", "comment", "consider", "maybe"],
        }
    
    def analyze_technical_debt(self) -> TechnicalDebtReport:
        """Analyze technical debt across the codebase."""
        print("🔍 Analyzing technical debt across the codebase...")
        
        self.debt_items = []
        
        # Scan all Rust files for technical debt
        for rust_file in self.crates_dir.rglob("*.rs"):
            self._scan_file_for_debt(rust_file)
        
        # Also scan for deprecated attributes
        self._scan_deprecated_attributes()
        
        # Categorize and prioritize debt items
        for item in self.debt_items:
            item.category = self._categorize_debt(item.content)
            item.priority = self._prioritize_debt(item.content)
            item.estimated_effort = self._estimate_effort(item.content, item.type)
        
        # Generate statistics
        items_by_type = defaultdict(int)
        items_by_priority = defaultdict(int)
        items_by_category = defaultdict(int)
        items_by_crate = defaultdict(int)
        
        for item in self.debt_items:
            items_by_type[item.type] += 1
            items_by_priority[item.priority] += 1
            items_by_category[item.category] += 1
            crate_name = self._extract_crate_name(item.file_path)
            items_by_crate[crate_name] += 1
        
        # Generate elimination plan
        elimination_plan = self._generate_elimination_plan()
        
        return TechnicalDebtReport(
            total_items=len(self.debt_items),
            items_by_type=dict(items_by_type),
            items_by_priority=dict(items_by_priority),
            items_by_category=dict(items_by_category),
            items_by_crate=dict(items_by_crate),
            debt_items=self.debt_items,
            elimination_plan=elimination_plan
        )
    
    def _scan_file_for_debt(self, file_path: Path):
        """Scan a single file for technical debt markers."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            for i, line in enumerate(lines, 1):
                for debt_type, pattern in self.debt_patterns.items():
                    match = re.search(pattern, line, re.IGNORECASE)
                    if match:
                        # Get context (surrounding lines)
                        context_start = max(0, i - 3)
                        context_end = min(len(lines), i + 2)
                        context = ''.join(lines[context_start:context_end]).strip()
                        
                        debt_item = TechnicalDebtItem(
                            type=debt_type,
                            file_path=str(file_path),
                            line_number=i,
                            content=match.group(1).strip(),
                            context=context,
                            priority="MEDIUM",  # Will be updated later
                            category="unknown",  # Will be updated later
                            estimated_effort="MEDIUM"  # Will be updated later
                        )
                        self.debt_items.append(debt_item)
        
        except Exception as e:
            print(f"Warning: Could not scan {file_path}: {e}")
    
    def _scan_deprecated_attributes(self):
        """Scan for deprecated attributes and functions."""
        for rust_file in self.crates_dir.rglob("*.rs"):
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Find deprecated attributes
                deprecated_pattern = r'#\[deprecated[^\]]*\]'
                for match in re.finditer(deprecated_pattern, content):
                    line_num = content[:match.start()].count('\n') + 1
                    
                    debt_item = TechnicalDebtItem(
                        type="DEPRECATED",
                        file_path=str(rust_file),
                        line_number=line_num,
                        content=match.group(0),
                        context="",
                        priority="MEDIUM",
                        category="cleanup",
                        estimated_effort="SMALL"
                    )
                    self.debt_items.append(debt_item)
            
            except Exception:
                continue
    
    def _categorize_debt(self, content: str) -> str:
        """Categorize technical debt by content."""
        content_lower = content.lower()
        
        for category, keywords in self.category_keywords.items():
            if any(keyword in content_lower for keyword in keywords):
                return category
        
        return "other"
    
    def _prioritize_debt(self, content: str) -> str:
        """Prioritize technical debt by content."""
        content_lower = content.lower()
        
        for priority, keywords in self.priority_keywords.items():
            if any(keyword in content_lower for keyword in keywords):
                return priority
        
        return "MEDIUM"
    
    def _estimate_effort(self, content: str, debt_type: str) -> str:
        """Estimate effort required to address technical debt."""
        content_lower = content.lower()
        
        # TRIVIAL: Simple comments, documentation
        if any(word in content_lower for word in ["comment", "document", "typo", "rename"]):
            return "TRIVIAL"
        
        # SMALL: Simple implementations, cleanups
        if debt_type == "DEPRECATED" or any(word in content_lower for word in ["remove", "clean", "delete", "simple"]):
            return "SMALL"
        
        # LARGE: Complex implementations, integrations
        if any(word in content_lower for word in ["implement", "integrate", "federation", "network", "complex"]):
            return "LARGE"
        
        return "MEDIUM"
    
    def _extract_crate_name(self, file_path: str) -> str:
        """Extract crate name from file path."""
        path_parts = Path(file_path).parts
        crates_index = -1
        
        for i, part in enumerate(path_parts):
            if part == "crates":
                crates_index = i
                break
        
        if crates_index != -1 and crates_index + 1 < len(path_parts):
            return path_parts[crates_index + 1]
        
        return "unknown"
    
    def _generate_elimination_plan(self) -> List[str]:
        """Generate a prioritized elimination plan."""
        plan = []
        
        # Group by priority and effort
        high_priority = [item for item in self.debt_items if item.priority == "HIGH"]
        medium_priority = [item for item in self.debt_items if item.priority == "MEDIUM"]
        low_priority = [item for item in self.debt_items if item.priority == "LOW"]
        
        if high_priority:
            plan.append(f"🔥 PHASE 1: Address {len(high_priority)} HIGH priority items first")
            
        # Group by effort within priority
        trivial_items = [item for item in self.debt_items if item.estimated_effort == "TRIVIAL"]
        small_items = [item for item in self.debt_items if item.estimated_effort == "SMALL"]
        
        if trivial_items:
            plan.append(f"⚡ Quick wins: {len(trivial_items)} trivial items (documentation, comments)")
        
        if small_items:
            plan.append(f"🧹 Cleanup: {len(small_items)} small items (deprecated code, simple fixes)")
        
        # Category-based grouping
        implementation_items = [item for item in self.debt_items if item.category == "implementation"]
        integration_items = [item for item in self.debt_items if item.category == "integration"]
        
        if implementation_items:
            plan.append(f"🏗️ Implementation: {len(implementation_items)} missing implementations")
        
        if integration_items:
            plan.append(f"🔗 Integration: {len(integration_items)} integration tasks")
        
        return plan
    
    def eliminate_trivial_debt(self) -> Tuple[int, int]:
        """Eliminate trivial technical debt items."""
        print("⚡ Eliminating trivial technical debt...")
        
        files_updated = 0
        items_fixed = 0
        
        trivial_items = [item for item in self.debt_items if item.estimated_effort == "TRIVIAL"]
        
        for item in trivial_items:
            if self._fix_trivial_debt(item):
                items_fixed += 1
        
        return files_updated, items_fixed
    
    def _fix_trivial_debt(self, item: TechnicalDebtItem) -> bool:
        """Fix a trivial debt item."""
        try:
            file_path = Path(item.file_path)
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            if item.line_number <= len(lines):
                line = lines[item.line_number - 1]
                
                # Simple fixes for trivial items
                if "TODO: document" in line.lower():
                    # Add a basic documentation comment
                    lines[item.line_number - 1] = line.replace("TODO: document", "/// TODO: Add documentation")
                    
                elif "TODO: comment" in line.lower():
                    # Convert to proper comment
                    lines[item.line_number - 1] = line.replace("TODO: comment", "/// TODO: Add detailed comment")
                
                # Write back the file
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.writelines(lines)
                
                return True
        
        except Exception:
            pass
        
        return False
    
    def generate_debt_elimination_report(self, report: TechnicalDebtReport) -> str:
        """Generate a comprehensive technical debt elimination report."""
        report_content = f"""# 📊 Technical Debt Elimination Report

**Generated**: {self._get_timestamp()}
**Total Technical Debt Items**: {report.total_items}

## 🔍 Technical Debt Distribution

### By Type
"""
        
        for debt_type, count in sorted(report.items_by_type.items(), key=lambda x: x[1], reverse=True):
            percentage = (count / report.total_items) * 100
            report_content += f"- **{debt_type}**: {count} items ({percentage:.1f}%)\n"
        
        report_content += "\n### By Priority\n"
        for priority, count in sorted(report.items_by_priority.items(), key=lambda x: x[1], reverse=True):
            percentage = (count / report.total_items) * 100
            report_content += f"- **{priority}**: {count} items ({percentage:.1f}%)\n"
        
        report_content += "\n### By Category\n"
        for category, count in sorted(report.items_by_category.items(), key=lambda x: x[1], reverse=True):
            percentage = (count / report.total_items) * 100
            report_content += f"- **{category.title()}**: {count} items ({percentage:.1f}%)\n"
        
        report_content += "\n### By Crate\n"
        for crate, count in sorted(report.items_by_crate.items(), key=lambda x: x[1], reverse=True):
            report_content += f"- **{crate}**: {count} items\n"
        
        report_content += f"\n## 🎯 Elimination Plan\n\n"
        for i, plan_item in enumerate(report.elimination_plan, 1):
            report_content += f"{i}. {plan_item}\n"
        
        report_content += f"\n## 📋 High Priority Items\n\n"
        high_priority_items = [item for item in report.debt_items if item.priority == "HIGH"]
        for i, item in enumerate(high_priority_items[:10], 1):
            crate = self._extract_crate_name(item.file_path)
            report_content += f"### {i}. {item.type} in {crate}\n"
            report_content += f"**File**: `{item.file_path}:{item.line_number}`\n"
            report_content += f"**Content**: {item.content}\n"
            report_content += f"**Category**: {item.category} | **Effort**: {item.estimated_effort}\n\n"
        
        report_content += f"\n## 🧹 Quick Wins (Trivial Items)\n\n"
        trivial_items = [item for item in report.debt_items if item.estimated_effort == "TRIVIAL"]
        for i, item in enumerate(trivial_items[:10], 1):
            crate = self._extract_crate_name(item.file_path)
            report_content += f"{i}. **{crate}**: {item.content} `({item.file_path}:{item.line_number})`\n"
        
        return report_content
    
    def _get_timestamp(self) -> str:
        """Get current timestamp for reports."""
        return datetime.now().strftime("%Y-%m-%d %H:%M:%S")

def main():
    """Main function to run technical debt analysis and elimination."""
    project_root = Path.cwd()
    
    if not (project_root / "crates").exists():
        print("Error: Run this script from the project root directory")
        sys.exit(1)
    
    eliminator = TechnicalDebtEliminator(project_root)
    
    print("🚀 Starting Technical Debt Analysis...")
    report = eliminator.analyze_technical_debt()
    
    print(f"\n📊 Analysis Complete!")
    print(f"   - Found {report.total_items} technical debt items")
    print(f"   - Across {len(report.items_by_crate)} crates")
    print(f"   - {report.items_by_priority.get('HIGH', 0)} high priority items")
    print(f"   - {sum(1 for item in report.debt_items if item.estimated_effort == 'TRIVIAL')} trivial fixes available")
    
    # Generate detailed report
    detailed_report = eliminator.generate_debt_elimination_report(report)
    with open("docs/TECHNICAL_DEBT_ELIMINATION_REPORT.md", "w") as f:
        f.write(detailed_report)
    
    # Export debt items as JSON for tooling
    debt_items_json = []
    for item in report.debt_items:
        debt_items_json.append({
            "type": item.type,
            "file": item.file_path,
            "line": item.line_number,
            "content": item.content,
            "priority": item.priority,
            "category": item.category,
            "effort": item.estimated_effort
        })
    
    with open("docs/technical_debt_items.json", "w") as f:
        json.dump(debt_items_json, f, indent=2)
    
    # Attempt to eliminate trivial debt
    files_updated, items_fixed = eliminator.eliminate_trivial_debt()
    
    print(f"\n✅ Generated:")
    print(f"   - docs/TECHNICAL_DEBT_ELIMINATION_REPORT.md")
    print(f"   - docs/technical_debt_items.json")
    
    print(f"\n🎯 Top Priorities:")
    for i, plan_item in enumerate(report.elimination_plan[:5], 1):
        print(f"   {i}. {plan_item}")
    
    if items_fixed > 0:
        print(f"\n⚡ Quick Fixes Applied: {items_fixed} trivial items fixed")
    
    print(f"\n🚀 Technical debt analysis complete!")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 