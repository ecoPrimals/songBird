# Ecosystem Primal Integration

**SOVEREIGNTY PRINCIPLE**: Songbird does not "know" other primals exist.
All inter-primal coordination uses capability-based discovery at runtime.

## Production Code

The real implementation lives in `crates/songbird-universal/src/adapters/`:

- `ComputeAdapter` — generic compute capability (not provider-specific)
- `SecurityAdapter` — generic security capability (not provider-specific)
- `StorageAdapter` — generic storage capability (not provider-specific)
- `AIAdapter` — generic AI capability (not provider-specific)

Each adapter discovers its provider via `find_primals_with_capability()` at
runtime using environment-driven socket paths and XDG resolution. No compile-time
import of any other primal's code.
