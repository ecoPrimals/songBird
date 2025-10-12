================================================================================
  🚀 PHASE 2 - ERROR HANDLING ROADMAP
================================================================================

With Phase 1 complete, here's the plan for Phase 2.

================================================================================
CURRENT STATUS
================================================================================

✅ Phase 1 Complete: 76 hardcoded values eliminated
✅ Grade: C+ (70/100)
✅ All 9 working crates build successfully
✅ Zero regressions

================================================================================
PHASE 2 SCOPE: ERROR HANDLING
================================================================================

TARGET: Eliminate all unwrap/expect/panic for production readiness

CURRENT STATE:
- 451 unwrap/expect calls
- 152 potential panics
- Mix of error handling styles
- Some proper Result<T, SongbirdError> already

GOAL:
- 0 unwrap/expect in production code
- 0 panics in production code
- Consistent Result<T, SongbirdError> pattern
- Proper error context and tracing

ESTIMATED TIME: 6-8 hours

================================================================================
PATTERN TO APPLY
================================================================================

BEFORE:
```rust
let value = config.get("key").unwrap();
let parsed = value.parse::<u16>().expect("Invalid port");
```

AFTER:
```rust
let value = config.get("key")
    .ok_or_else(|| SongbirdError::config_error("Missing key"))?;
let parsed = value.parse::<u16>()
    .map_err(|e| SongbirdError::config_error(format!("Invalid port: {}", e)))?;
```

Or with tracing:
```rust
let value = config.get("key")
    .inspect_err(|e| error!("Config missing key: {}", e))
    .ok_or_else(|| SongbirdError::config_error("Missing key"))?;
```

================================================================================
PRIORITY ORDER
================================================================================

1. HIGH: Error paths that can crash services
   - Network initialization
   - Configuration loading
   - Service registration
   - Health checks

2. MEDIUM: Error paths that affect reliability
   - Discovery operations
   - Capability queries
   - Registry operations

3. LOW: Internal utilities and testing code
   - Test utilities (can keep unwrap)
   - Debug/development tools
   - Example code

================================================================================
CRATE BREAKDOWN
================================================================================

Estimated unwrap/expect per crate (needs verification):
- songbird-discovery: ~80
- songbird-universal: ~120
- songbird-config: ~60
- songbird-types: ~40
- songbird-registry: ~30
- songbird-network-federation: ~20
- songbird-observability: ~30
- songbird-test-utils: ~40 (mostly tests, can skip)
- songbird-canonical: ~30

Total: ~450

STRATEGY:
1. Start with config (foundation)
2. Move to types (dependencies)
3. Then discovery, registry, network
4. Finally universal and observability

================================================================================
SUCCESS CRITERIA
================================================================================

✅ Zero unwrap/expect in production code
✅ All error paths return Result
✅ Consistent error types
✅ Proper error context
✅ All builds passing
✅ Tests updated where needed
✅ Grade improvement: C+ → B (70 → 80/100)

================================================================================
ALTERNATIVE: PHASE 3 - TESTING
================================================================================

If you prefer to tackle testing first:

SCOPE:
- Fix test compilation issues
- Set up coverage measurement
- Write missing unit tests
- E2E test framework
- Chaos testing basics

TARGET:
- 90% test coverage
- All tests passing
- Coverage reporting
- CI/CD integration

ESTIMATED TIME: 8-10 hours

GRADE IMPACT: C+ → B+ (70 → 85/100)

================================================================================
RECOMMENDATION
================================================================================

**Phase 2 (Error Handling) first**, then Phase 3 (Testing).

REASONING:
1. Error handling is critical for production
2. Makes the codebase more testable
3. Cleaner error paths = easier to test
4. Logical progression: Config → Errors → Tests
5. Faster grade improvement path

================================================================================
NEXT COMMAND
================================================================================

Ready when you are! Just say "proceed" to start Phase 2, or specify
"phase 3" if you want to tackle testing first.

================================================================================
