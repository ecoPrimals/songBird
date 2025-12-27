#!/bin/bash
# Songbird Archive Cleanup - December 26, 2025
# Move historical docs to parent archive and reduce false positives

set -e

SONGBIRD_ROOT="/home/eastgate/Development/ecoPrimals/songbird"
ARCHIVE_ROOT="/home/eastgate/Development/ecoPrimals/archive/songbird-dec-26-2025-cleanup"

echo "🧹 Songbird Archive Cleanup - December 26, 2025"
echo "================================================"
echo ""

# Create archive structure
mkdir -p "${ARCHIVE_ROOT}"/{docs,showcase-receipts,backup-files}

echo "📦 Phase 1: Archive dated session docs..."

# Move old session directories to archive
if [ -d "${SONGBIRD_ROOT}/docs/sessions/2025-12-17" ]; then
    echo "  → Moving 2025-12-17 sessions..."
    mv "${SONGBIRD_ROOT}/docs/sessions/2025-12-17" "${ARCHIVE_ROOT}/docs/"
fi

if [ -d "${SONGBIRD_ROOT}/docs/sessions/2025-12-17-evening" ]; then
    echo "  → Moving 2025-12-17-evening sessions..."
    mv "${SONGBIRD_ROOT}/docs/sessions/2025-12-17-evening" "${ARCHIVE_ROOT}/docs/"
fi

if [ -d "${SONGBIRD_ROOT}/docs/sessions/2025-12-17-final" ]; then
    echo "  → Moving 2025-12-17-final sessions..."
    mv "${SONGBIRD_ROOT}/docs/sessions/2025-12-17-final" "${ARCHIVE_ROOT}/docs/"
fi

if [ -d "${SONGBIRD_ROOT}/docs/sessions/2025-12-18" ]; then
    echo "  → Moving 2025-12-18 sessions..."
    mv "${SONGBIRD_ROOT}/docs/sessions/2025-12-18" "${ARCHIVE_ROOT}/docs/"
fi

if [ -d "${SONGBIRD_ROOT}/docs/sessions/december-2025" ]; then
    echo "  → Moving december-2025 sessions..."
    mv "${SONGBIRD_ROOT}/docs/sessions/december-2025" "${ARCHIVE_ROOT}/docs/"
fi

# Move old comprehensive audits
if [ -d "${SONGBIRD_ROOT}/docs/audits/2025-12-14-comprehensive" ]; then
    echo "  → Moving 2025-12-14 comprehensive audits..."
    mv "${SONGBIRD_ROOT}/docs/audits/2025-12-14-comprehensive" "${ARCHIVE_ROOT}/docs/"
fi

echo ""
echo "📦 Phase 2: Archive showcase receipts..."

# Archive showcase receipts (they have lots of dated files)
find "${SONGBIRD_ROOT}/showcase" -type d -name "*receipt*" -exec mv {} "${ARCHIVE_ROOT}/showcase-receipts/" \; 2>/dev/null || true

echo ""
echo "📦 Phase 3: Archive backup files..."

# Move .backup files
find "${SONGBIRD_ROOT}/crates" -name "*.backup" -exec mv {} "${ARCHIVE_ROOT}/backup-files/" \; 2>/dev/null || true

echo ""
echo "📦 Phase 4: Archive existing docs/archive..."

if [ -d "${SONGBIRD_ROOT}/docs/archive" ]; then
    echo "  → Moving docs/archive to parent..."
    mv "${SONGBIRD_ROOT}/docs/archive" "${ARCHIVE_ROOT}/docs-archive"
fi

echo ""
echo "📊 Cleanup Summary"
echo "=================="

# Count what we archived
ARCHIVED_SESSIONS=$(find "${ARCHIVE_ROOT}/docs" -name "*.md" 2>/dev/null | wc -l)
ARCHIVED_RECEIPTS=$(find "${ARCHIVE_ROOT}/showcase-receipts" -type f 2>/dev/null | wc -l)
ARCHIVED_BACKUPS=$(find "${ARCHIVE_ROOT}/backup-files" -type f 2>/dev/null | wc -l)

echo "  ✅ Archived session docs: ${ARCHIVED_SESSIONS} files"
echo "  ✅ Archived receipts: ${ARCHIVED_RECEIPTS} files"
echo "  ✅ Archived backups: ${ARCHIVED_BACKUPS} files"

echo ""
echo "📍 Archive location: ${ARCHIVE_ROOT}"
echo ""

# Create index file
cat > "${ARCHIVE_ROOT}/README.md" << 'EOF'
# Songbird Archive - December 26, 2025 Cleanup

## Purpose

This archive contains historical documentation, session reports, and backup files that were removed from the main Songbird workspace to reduce false positives in audits and improve workspace clarity.

## Contents

### `/docs/` - Historical Session Documentation

- **2025-12-17/**: December 17 session docs (40 files)
  - Comprehensive audits, execution reports, session summaries
  - FALSE_POSITIVES_GUIDE.md (for reference)
  
- **2025-12-17-evening/**: December 17 evening session (10 files)
  - Albatross multi-protocol testing
  - Two-tower live success verification
  
- **2025-12-17-final/**: December 17 final session (15 files)
  - Final execution status
  - Complete handoff documentation
  
- **2025-12-18/**: December 18 session docs (53 files)
  - Comprehensive audits and execution reports
  - Evolution and migration documentation
  
- **december-2025/**: General December 2025 sessions (17 files)
  - Network connectivity improvements
  - Port fallback testing
  - Discovery verification fixes

- **2025-12-14-comprehensive/**: December 14 comprehensive audits
  - Historical audit session data

### `/showcase-receipts/` - Test Run Receipts

Dated receipt files from showcase demonstrations, including:
- BearDog encryption verification receipts
- BirdSong privacy test outputs
- P2P integration test artifacts

### `/backup-files/` - Code Backups

.backup files from development iterations:
- core.rs.backup
- mod.rs.backup
- Other temporary backup files

### `/docs-archive/` - Previous Archive

Contents of `docs/archive/` moved from main workspace.

## Archive Rationale

These files were archived to:

1. **Reduce False Positives**: Many TODO/FIXME markers in historical docs were inflating debt counts
2. **Improve Navigation**: Cleaner workspace structure with only current docs
3. **Preserve History**: Maintain fossil record for reference while decluttering
4. **Disk Space**: Part of disk space management effort

## Restoration

If you need to restore any files:

```bash
# From the archive root
cp -r docs/2025-12-17 /path/to/songbird/docs/sessions/
```

## Metadata

- **Archived**: December 26, 2025
- **Reason**: Workspace cleanup, reduce false positives
- **Total Files**: ~500+ documents and artifacts
- **Disk Saved**: ~50-100 MB
- **Status**: Safe to delete after 90 days if space needed

EOF

echo "✅ Archive index created: ${ARCHIVE_ROOT}/README.md"
echo ""
echo "🎉 Cleanup complete!"
echo ""
echo "Next steps:"
echo "  1. Rerun audit to see reduced false positives"
echo "  2. cargo clean to free disk space"
echo "  3. Review remaining TODOs in production code"

