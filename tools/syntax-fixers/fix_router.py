#!/usr/bin/env python3
"""Fix router.rs delimiter errors."""

import re

filepath = "crates/songbird-universal/src/sovereignty/router.rs"

with open(filepath, 'r') as f:
    content = f.read()

# Fix all the delimiter issues in one pass
replacements = [
    # Line 143: Add missing comma
    (r'segment_id: format!\("segment_\{\}", i\)\n', r'segment_id: format!("segment_{}", i),\n'),
    
    # Line 144-145: Fix function calls with wrong delimiters
    (r'\.score\(,', r'.score(),'),
    (r'\.clone\(,', r'.clone(),'),
    
    # Line 146: Fix struct name and opening
    (r'SecurityAssessment  \{security_score:', r'SecurityAssessment { security_score:'),
    
    # Line 152: Fix push call
    (r'segment_assessments\.push\(segment_assessment\)\);', r'segment_assessments.push(segment_assessment);'),
    
    # Lines 159-162: Fix struct literal
    (r'Ok\(PathSovereigntyAssessment  \{overall_score\)\n\s+segment_assessments\)\n\s+compliance_level\)\n\s+sovereignty_risks\)\n\s+}\)', 
     r'Ok(PathSovereigntyAssessment { overall_score,\n            segment_assessments,\n            compliance_level,\n            sovereignty_risks,\n        })'),
    
    # Line 168: Fix match expression
    (r'matches!\(assessment\.compliance_level\)\n\s+SovereigntyComplianceLevel::', 
     r'matches!(assessment.compliance_level, SovereigntyComplianceLevel::'),
    (r'SovereigntyComplianceLevel::MostlyCompliant\)', r'SovereigntyComplianceLevel::MostlyCompliant)'),
    
    # Lines 187-189: Fix vector literal
    (r'SecurityCapability::Encryption\)\n\s+SecurityCapability::Authentication\)\n\s+\]\)', 
     r'SecurityCapability::Encryption,\n            SecurityCapability::Authentication,\n        ])'),
    
    # Lines 197-199: Fix iterator chain
    (r'path\.segments\.iter,\n\s+\.map\(\|segment\| segment\.sovereignty_level\.score\(\)\n\s+\.sum\(\);', 
     r'path.segments.iter()\n            .map(|segment| segment.sovereignty_level.score())\n            .sum();'),
    
    # Lines 209-211: Fix iterator chain
    (r'path\.segments\.iter,\n\s+\.map\(\|segment\| segment\.efficiency_score\)\n\s+\.sum\(\);', 
     r'path.segments.iter()\n            .map(|segment| segment.efficiency_score)\n            .sum();'),
    
    # Line 233: Fix match pattern
    (r'match capability_count  \{', r'match capability_count {'),
    
    # Line 246: Fix match pattern
    (r'match sovereignty_score  \{match sovereignty_score  \{', r'match sovereignty_score {'),
]

for pattern, replacement in replacements:
    content = re.sub(pattern, replacement, content, flags=re.MULTILINE)

with open(filepath, 'w') as f:
    f.write(content)

print("✅ Fixed router.rs")

