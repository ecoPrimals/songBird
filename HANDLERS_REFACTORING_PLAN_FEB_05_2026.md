# Handlers Refactoring Plan - Phase 5B Continuation

**Date**: February 5, 2026  
**Target**: `handlers.rs` (1,132 lines)  
**Status**: 🚧 **IN PROGRESS**

---

## Analysis Summary

Current file: `crates/songbird-orchestrator/src/ipc/unix/handlers.rs` (1,132 lines)

### Handler Categories (8 groups, 23 functions)

| Category | Lines | Functions | Priority |
|----------|-------|-----------|----------|
| Primal Registration | ~142 | 5 | High |
| Health & Diagnostics | ~75 | 3 | High |
| Peer Discovery | ~117 | 6 | High |
| Standard Methods | ~151 | 3 | High |
| Capability Discovery | ~13 | 1 | Low (tiny) |
| Encryption Wrappers | ~161 | 2 | Medium |
| Network Methods | ~360 | 3 | Medium |
| HTTP Delegation | ~81 | 1 | Low |

---

## Refactoring Structure

```
handlers.rs (1,132 lines) →handlers/

handlers/
├── mod.rs                    (~100 lines) - Module docs + re-exports
├── primal_registration.rs    (~150 lines) - 5 handlers
│   ├── handle_primal_register
│   ├── handle_primal_unregister
│   ├── handle_get_provider
│   ├── handle_list_providers
│   └── handle_list_all_primals
│
├── health.rs                 (~90 lines)  - 3 handlers
│   ├── handle_health
│   ├── handle_health_standard
│   └── handle_ping
│
├── peer_discovery.rs         (~120 lines) - 6 handlers
│   ├── handle_discovery_list_peers
│   ├── handle_discovery_peer_count
│   ├── handle_discovery_rejected_peers
│   ├── handle_peer_ping
│   └── handle_discovery_status
│
├── standard_methods.rs       (~160 lines) - 3 handlers
│   ├── handle_identity
│   └── handle_rpc_discover
│
├── encryption.rs             (~170 lines) - 2 handlers + 1 legacy
│   ├── handle_encrypt_discovery
│   ├── handle_decrypt_discovery
│   └── handle_discover_capabilities (legacy)
│
├── network.rs                (~370 lines) - 3 handlers
│   ├── handle_beacon_exchange
│   ├── handle_network_broadcast
│   └── handle_network_listen
│
└── http_delegation.rs        (~90 lines)  - 1 handler
    └── handle_http_request
```

**Total**: ~1,250 lines across 8 modules (from 1,132-line monolith)

---

## Shared Dependencies

All handler modules will share:
```rust
use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::super::jsonrpc::JsonRpcError;
use crate::app::connection_manager::ConnectionManager;
use crate::ipc::primal_registry::PrimalRegistry;
use songbird_http_client::SongbirdHttpClient;
```

---

## Implementation Steps

1. ✅ Create `handlers/` directory
2. ⏳ Extract primal_registration module
3. ⏳ Extract health module
4. ⏳ Extract peer_discovery module
5. ⏳ Extract standard_methods module
6. ⏳ Extract encryption module
7. ⏳ Extract network module
8. ⏳ Extract http_delegation module
9. ⏳ Create mod.rs with re-exports
10. ⏳ Update parent module imports
11. ⏳ Remove old handlers.rs
12. ⏳ Verify build passes
13. ⏳ Run tests
14. ⏳ Commit and push

---

## Expected Benefits

✅ **Modularity**: 8 focused modules vs 1 monolith  
✅ **Largest Module**: ~370 lines (vs 1,132 before)  
✅ **Clear Responsibilities**: Each module handles one category  
✅ **Better Navigation**: Easy to find specific handlers  
✅ **Easier Testing**: Tests can be co-located per module  
✅ **Deep Debt Impact**: +0.1% (99.5% → 99.6%)

---

## Risk Assessment

**Low Risk** - Clean boundaries, no shared state, pure functions

- All handlers are async functions with clear signatures
- No complex interdependencies
- Straightforward extraction
- All tests should continue passing

---

**Status**: 🚧 **Ready to proceed with extraction**  
**Next**: Extract primal_registration.rs module
