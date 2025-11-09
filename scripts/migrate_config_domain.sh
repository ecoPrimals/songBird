#!/bin/bash
# migrate_config_domain.sh - Create canonical config for a domain
# Part of the Unification & Modernization initiative
#
# Usage: ./migrate_config_domain.sh <domain>
# Example: ./migrate_config_domain.sh security

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <domain>"
  echo "Example: $0 security"
  echo ""
  echo "Common domains: network, security, discovery, service, gaming, orchestration"
  exit 1
fi

DOMAIN=$1
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CANONICAL_DIR="$PROJECT_ROOT/crates/songbird-config/src/canonical"
REPORT_DIR="$PROJECT_ROOT/reports"

# Create necessary directories
mkdir -p "$CANONICAL_DIR"
mkdir -p "$REPORT_DIR"

echo "🔄 Migrating ${DOMAIN} configuration to canonical pattern"
echo "========================================================"
echo ""

# Find all config structs for this domain
echo "📊 Finding ${DOMAIN} config structs..."
DOMAIN_PATTERN="${DOMAIN^}.*Config|${DOMAIN,,}.*Config"
CONFIGS_FILE="$REPORT_DIR/${DOMAIN}_configs.txt"

grep -rn "struct.*${DOMAIN^}.*Config\|struct.*${DOMAIN,,}.*Config" \
  "$PROJECT_ROOT/crates"/*/src --include="*.rs" > "$CONFIGS_FILE" || true

CONFIG_COUNT=$(wc -l < "$CONFIGS_FILE")
echo "   Found $CONFIG_COUNT config structs"

if [ "$CONFIG_COUNT" -eq 0 ]; then
  echo "⚠️  No config structs found for domain: $DOMAIN"
  exit 1
fi

echo ""
echo "📄 Config structs to consolidate:"
cat "$CONFIGS_FILE" | sed 's|'"$PROJECT_ROOT"'/||' | sed 's/^/   /'

# Create canonical config file
CANONICAL_FILE="$CANONICAL_DIR/${DOMAIN}.rs"
DOMAIN_TITLE="${DOMAIN^}"

echo ""
echo "📝 Creating canonical config: $CANONICAL_FILE"

cat > "$CANONICAL_FILE" <<EOF
//! Canonical ${DOMAIN_TITLE} Configuration - Unified Modern Implementation
//!
//! This module provides the single, canonical \`${DOMAIN_TITLE}Config\` definition that replaces
//! all fragmented and deprecated ${DOMAIN,,} configuration structs across the codebase.
//!
//! ## Consolidated From
//! This unifies configurations from:
$(awk -F: '{print "//! - " $1}' "$CONFIGS_FILE" | sed 's|'"$PROJECT_ROOT"'/||' | sort | uniq)
//!
//! ## Migration Guide
//! - Replace all uses of old ${DOMAIN,,} configs with \`Canonical${DOMAIN_TITLE}Config\`
//! - Use \`from_env()\` for environment-based configuration
//! - Use \`validate()\` before using the configuration

use serde::{Deserialize, Serialize};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};

/// **CANONICAL**: ${DOMAIN_TITLE} Configuration
///
/// Unified ${DOMAIN,,} configuration that consolidates multiple definitions
/// from across the codebase into a single, maintainable structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canonical${DOMAIN_TITLE}Config {
    // TODO: Consolidate fields from the ${CONFIG_COUNT} config structs
    // Review $CONFIGS_FILE for field definitions
    
    /// Enable ${DOMAIN,,} features
    pub enabled: bool,
    
    // Add additional fields based on consolidation analysis
}

impl Default for Canonical${DOMAIN_TITLE}Config {
    fn default() -> Self {
        Self {
            enabled: true,
        }
    }
}

impl Canonical${DOMAIN_TITLE}Config {
    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - \`SONGBIRD_${DOMAIN^^}_ENABLED\` - Enable ${DOMAIN,,} features (default: true)
    ///
    /// # Errors
    /// Returns error if required environment variables are missing or invalid
    pub fn from_env() -> SongbirdResult<Self> {
        Ok(Self {
            enabled: SafeEnv::get_or_default(
                "SONGBIRD_${DOMAIN^^}_ENABLED",
                "true".to_string()
            ).parse().unwrap_or(true),
        })
    }
    
    /// Validate configuration values
    ///
    /// # Errors
    /// Returns error if configuration is invalid
    pub fn validate(&self) -> SongbirdResult<()> {
        // Add validation logic
        Ok(())
    }
}

/// Type alias for backward compatibility
pub type ${DOMAIN_TITLE}Config = Canonical${DOMAIN_TITLE}Config;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Canonical${DOMAIN_TITLE}Config::default();
        assert!(config.enabled);
    }
    
    #[test]
    fn test_from_env() {
        let config = Canonical${DOMAIN_TITLE}Config::from_env()
            .expect("Should load from environment");
        assert!(config.validate().is_ok());
    }
}
EOF

echo "✅ Created canonical config skeleton"

# Update mod.rs if it exists
MOD_FILE="$CANONICAL_DIR/mod.rs"
if [ -f "$MOD_FILE" ]; then
  if ! grep -q "pub mod ${DOMAIN};" "$MOD_FILE"; then
    echo "pub mod ${DOMAIN};" >> "$MOD_FILE"
    echo "✅ Updated $MOD_FILE"
  fi
else
  echo "pub mod ${DOMAIN};" > "$MOD_FILE"
  echo "✅ Created $MOD_FILE"
fi

# Generate migration checklist
CHECKLIST_FILE="$REPORT_DIR/${DOMAIN}_migration_checklist.md"
cat > "$CHECKLIST_FILE" <<EOF
# ${DOMAIN_TITLE} Configuration Migration Checklist

**Generated**: $(date)  
**Domain**: ${DOMAIN}  
**Config Count**: ${CONFIG_COUNT}

## Phase 1: Analysis (Day 1)

- [ ] Review all ${CONFIG_COUNT} config structs in: \`$CONFIGS_FILE\`
- [ ] Identify common fields across configurations
- [ ] Identify domain-specific fields
- [ ] Document field purposes and default values
- [ ] Check for conflicting field names or types

## Phase 2: Consolidation (Days 2-3)

- [ ] Update \`$CANONICAL_FILE\` with unified fields
- [ ] Implement \`from_env()\` with SafeEnv
- [ ] Implement \`validate()\` with proper checks
- [ ] Add comprehensive documentation
- [ ] Add unit tests for all scenarios

## Phase 3: Migration (Days 4-5)

### For each old config struct:

$(awk -F: '{print "- [ ] " $1 " (line " $2 ")"}' "$CONFIGS_FILE" | sed 's|'"$PROJECT_ROOT"'/||')

### Migration steps per file:
1. Replace import: \`use songbird_config::canonical::${DOMAIN}::Canonical${DOMAIN_TITLE}Config;\`
2. Update struct usage: \`Old${DOMAIN_TITLE}Config\` → \`Canonical${DOMAIN_TITLE}Config\`
3. Update field access if names changed
4. Test thoroughly
5. Mark old config as deprecated
6. Create PR with changes

## Phase 4: Cleanup (Week 2)

- [ ] Remove all deprecated ${DOMAIN,,} configs
- [ ] Update documentation
- [ ] Update examples
- [ ] Run full test suite
- [ ] Verify no references to old configs remain

## Phase 5: Validation

- [ ] \`cargo check --workspace\` passes
- [ ] \`cargo test --workspace\` passes
- [ ] \`cargo clippy --workspace\` passes
- [ ] All usages migrated to canonical config
- [ ] Documentation updated

## Notes

- Config structs found: ${CONFIG_COUNT}
- Canonical file: \`$CANONICAL_FILE\`
- Review file: \`$CONFIGS_FILE\`

## Commands

\`\`\`bash
# Review config structs
cat $CONFIGS_FILE

# Edit canonical config
\$EDITOR $CANONICAL_FILE

# Test changes
cargo test --package songbird-config -- ${DOMAIN}

# Find remaining references
grep -r "Old${DOMAIN_TITLE}Config" crates/ --include="*.rs"
\`\`\`
EOF

echo "✅ Created migration checklist: $CHECKLIST_FILE"
echo ""

echo "🎯 NEXT STEPS:"
echo "   1. Review config structs: cat $CONFIGS_FILE"
echo "   2. Consolidate fields in: $CANONICAL_FILE"
echo "   3. Follow checklist: $CHECKLIST_FILE"
echo "   4. Test: cargo test --package songbird-config -- ${DOMAIN}"
echo ""
echo "📚 REFERENCE:"
echo "   - Example canonical config: $PROJECT_ROOT/crates/songbird-config/src/canonical/network.rs"
echo "   - SafeEnv patterns: grep -r SafeEnv::get_or_default crates/songbird-config/"
echo ""

