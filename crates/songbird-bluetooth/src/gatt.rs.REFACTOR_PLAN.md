# GATT Client Refactoring Plan

## Current Issues:
1. `discover_services()` has cognitive complexity 26/25
2. Should be split into smaller, focused functions

## Refactoring Strategy:

### Extract Helper Functions:
1. `discover_service_batch()` - Discover one batch of services
2. `should_continue_discovery()` - Check if discovery should continue
3. `update_discovery_state()` - Update start handle for next iteration

This maintains the same logic but makes it more maintainable and testable.

## Next Steps After Clippy:
- Refactor into helper functions
- Add unit tests for each helper
- Ensure no behavioral changes

