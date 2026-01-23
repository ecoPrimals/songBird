# Outdated Scripts Archive

**Purpose**: Preserve historical test and utility scripts that are no longer functional due to codebase evolution.

## Why Archive These?

These scripts were created for specific purposes during development but are no longer compatible with the current codebase. They're preserved for historical reference and to understand past testing approaches.

## Files in This Archive

### test_phase1_parsing.sh
- **Purpose**: Test Phase 1 trust parsing (integer vs string format compatibility)
- **Date**: January 6, 2026 (last modified)
- **Issue**: Tests `songbird-orchestrator --test-parse-trust` command which no longer exists
- **Context**: Was used to verify BearDog/Songbird trust level format compatibility
- **Status**: Functionality superseded by comprehensive test suite

## Why No Longer Needed?

1. **test_phase1_parsing.sh**:
   - Command `test-parse-trust` removed from codebase
   - Trust parsing now covered by comprehensive unit tests
   - Modern test infrastructure (114 tests, 100% passing)
   - See: `crates/songbird-orchestrator/src/trust/` for current implementation

## Active Scripts (Root Directory)

For comparison, these scripts remain active and operational:
- `archive_old_sessions.sh` - Utility for session documentation management
- `check-tower.sh` - Tower Atomic verification
- `start-tower.sh` - Development environment startup
- `stop-tower.sh` - Clean shutdown

## Note

The testing principles from these scripts are preserved in the current test suite:
- `crates/songbird-orchestrator/tests/` - Integration tests
- `crates/songbird-http-client/tests/` - HTTP client tests (114 tests)
- `tests/` - Workspace-level tests

Historical context preserved, outdated code removed.

---

**Archive Date**: January 23, 2026  
**Archived By**: Session 23 cleanup  
**Reason**: Commands no longer exist in codebase

