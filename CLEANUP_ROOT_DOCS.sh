#!/bin/bash
# Root Documentation Cleanup Script
# Created: December 2, 2025 Evening
# Purpose: Organize root documentation into logical structure

set -e

echo "🧹 Cleaning up root documentation..."

# Create archive directories
mkdir -p docs/audits/dec-2-2025-morning
mkdir -p docs/audits/dec-2-2025-evening  
mkdir -p docs/status-historical
mkdir -p docs/sessions

echo "📂 Moving morning audit files..."
# Morning audit files (superseded by evening reports)
mv -n 00_AUDIT_INDEX_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_README.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_COMPLETE_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_EXECUTIVE_SUMMARY_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025_FINAL.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_FINDINGS_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_NEXT_STEPS_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_ACTION_PLAN_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n AUDIT_FINAL_REPORT.txt docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n TODO_REPORT_DEC_2_2025.txt docs/audits/dec-2-2025-morning/ 2>/dev/null || true

echo "📂 Keeping evening audit files in root (current)..."
# Evening audit files stay in root (these are current):
# - 00_AUDIT_EXECUTIVE_SUMMARY_DEC_2_EVENING.md
# - 00_READY_FOR_DEPLOYMENT.md
# - DEPLOYMENT_FINAL_STATUS_DEC_2.md
# - SESSION_COMPLETE_DEC_2_EVENING.md
# - FINAL_EXECUTION_REPORT_DEC_2_EVENING.md
# - COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025_EVENING.md
# - FEDERATION_TESTS_ANALYSIS_DEC_2.md
# - IMMEDIATE_ACTION_CHECKLIST_DEC_2_EVENING.md
# - PROGRESS_REPORT_DEC_2_EVENING.md

echo "📂 Moving historical session files..."
# Session summaries (historical)
mv -n SESSION_COMPLETE_AUDIT_AND_CLEANUP_DEC_2_2025.md docs/sessions/ 2>/dev/null || true
mv -n SESSION_COMPLETE_DEC2_FINAL.md docs/sessions/ 2>/dev/null || true
mv -n SESSION_COMPLETE_EXTENDED_DEC2.md docs/sessions/ 2>/dev/null || true
mv -n COMPLETE_SESSION_REPORT_DEC_2_2025.md docs/sessions/ 2>/dev/null || true
mv -n EXECUTION_SUMMARY_DEC_2_2025.md docs/sessions/ 2>/dev/null || true
mv -n WEEK1_COMPLETE_DEC2.md docs/sessions/ 2>/dev/null || true
mv -n WEEK1_HANDOFF_FINAL.md docs/sessions/ 2>/dev/null || true

echo "📂 Moving historical status files..."
# Status documents (historical)
mv -n ROOT_STATUS_DEC_2_2025.md docs/status-historical/ 2>/dev/null || true
mv -n PRIMAL_SDK_STATUS_DEC_2_2025.md docs/status-historical/ 2>/dev/null || true
mv -n ROOT_DOCS_STATUS.md docs/status-historical/ 2>/dev/null || true
mv -n SPRINT_STATUS.md docs/status-historical/ 2>/dev/null || true

echo "📂 Moving cleanup logs..."
# Cleanup logs
mv -n DOCUMENTATION_CLEANUP_COMPLETE.md docs/status-historical/ 2>/dev/null || true
mv -n ROOT_DOCS_CLEANED.txt docs/status-historical/ 2>/dev/null || true

echo "📂 Moving technical plans..."
# Technical plans (keep some, archive others)
mv -n PRIMAL_SDK_CLEANUP_PLAN.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n SPLITTING_CAPABILITIES_ADAPTER.md docs/audits/dec-2-2025-evening/ 2>/dev/null || true
mv -n ASYNC_AWAIT_AUDIT_DEC2.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n E2E_SLEEP_ELIMINATION_COMPLETE.md docs/sessions/ 2>/dev/null || true

echo "📂 Moving superseded start files..."
# Superseded start files
mv -n 00_START_HERE_DEC_2_2025.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n 00_START_HERE.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true
mv -n 00_READ_ME_FIRST.md docs/audits/dec-2-2025-morning/ 2>/dev/null || true

echo "📂 Creating index files..."
# Create index in each directory
cat > docs/audits/dec-2-2025-morning/README.md << 'EOF'
# Morning Audit - December 2, 2025

**Status**: Historical - Superseded by evening audit  
**Purpose**: Reference only

## Files
- Initial comprehensive audit
- Morning session reports
- Superseded by evening reports in root

## Current Reports
See root directory for latest:
- `00_AUDIT_EXECUTIVE_SUMMARY_DEC_2_EVENING.md`
- `COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025_EVENING.md`
EOF

cat > docs/audits/dec-2-2025-evening/README.md << 'EOF'
# Evening Execution - December 2, 2025

**Status**: Current - See root directory  
**Purpose**: Current audit reports are in root

## Current Files (in root)
- `00_AUDIT_EXECUTIVE_SUMMARY_DEC_2_EVENING.md`
- `00_READY_FOR_DEPLOYMENT.md`
- `DEPLOYMENT_FINAL_STATUS_DEC_2.md`
- `SESSION_COMPLETE_DEC_2_EVENING.md`
- `FINAL_EXECUTION_REPORT_DEC_2_EVENING.md`
- `COMPREHENSIVE_AUDIT_REPORT_DEC_2_2025_EVENING.md`
- `FEDERATION_TESTS_ANALYSIS_DEC_2.md`
- `IMMEDIATE_ACTION_CHECKLIST_DEC_2_EVENING.md`
- `PROGRESS_REPORT_DEC_2_EVENING.md`
EOF

cat > docs/sessions/README.md << 'EOF'
# Historical Session Summaries

**Status**: Historical reference  
**Purpose**: Archive of completed sessions

See root directory for current session:
- `SESSION_COMPLETE_DEC_2_EVENING.md`
EOF

cat > docs/status-historical/README.md << 'EOF'
# Historical Status Documents

**Status**: Historical reference  
**Purpose**: Archive of status reports

See root directory for current status:
- `STATUS.md`
- `CURRENT_STATUS.md`
EOF

echo "✅ Root documentation cleanup complete!"
echo ""
echo "📊 Summary:"
echo "  - Kept in root: ~15-20 current files"
echo "  - Archived: ~30 historical files"
echo "  - Structure:"
echo "    - docs/audits/dec-2-2025-morning/ (morning audit)"
echo "    - docs/audits/dec-2-2025-evening/ (index only)"
echo "    - docs/sessions/ (historical sessions)"
echo "    - docs/status-historical/ (old status docs)"
echo ""
echo "📂 Root now contains:"
echo "  - 00_START_HERE_LATEST.md (NEW - primary entry)"
echo "  - Current deployment guides (3 files)"
echo "  - Latest audit reports (9 files)"
echo "  - Core project files (README, CONTRIBUTING, etc.)"
echo ""
echo "🎯 Next: Review 00_START_HERE_LATEST.md"

