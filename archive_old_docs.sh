#!/bin/bash
# Archive old planning docs - Jan 27, 2026 Evening
set -e

echo "📦 Creating archive directory..."
mkdir -p archive/jan-2026-planning

echo "📄 Moving superseded docs..."
mv REQWEST_MIGRATION_GUIDE.md archive/jan-2026-planning/ 2>/dev/null || echo "  (already moved)"
mv EVOLUTION_HARDENING_PLAN.md archive/jan-2026-planning/ 2>/dev/null || echo "  (already moved)"
mv METRICS_DASHBOARD.md archive/jan-2026-planning/ 2>/dev/null || echo "  (already moved)"
mv PRODUCTION_READINESS_FINAL.md archive/jan-2026-planning/ 2>/dev/null || echo "  (already moved)"

echo "📝 Creating archive README..."
cat > archive/jan-2026-planning/README.md << 'EOF'
# January 2026 - Planning Documents Archive

Historical planning and roadmap documents from Jan 24-25, 2026.

## Contents

- **REQWEST_MIGRATION_GUIDE.md** - Completed reqwest → pure Rust migration (✅ Complete)
- **EVOLUTION_HARDENING_PLAN.md** - Evolution strategy (Jan 24, superseded)
- **METRICS_DASHBOARD.md** - Metrics tracking (Jan 25, superseded)
- **PRODUCTION_READINESS_FINAL.md** - Production readiness (Jan 25, superseded)

**Superseded By**: January 27, 2026 comprehensive session  
**See**: `/JAN_27_2026_SESSION_INDEX.md` for current state

**Archived**: January 27, 2026 (Evening)  
**Status**: Preserved as fossil record ✅
EOF

echo "✅ Archive complete!"
echo ""
echo "📊 Archived files:"
ls -lh archive/jan-2026-planning/ 2>/dev/null || echo "  (directory created)"
