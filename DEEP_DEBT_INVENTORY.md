# 🔧 Songbird Deep Debt Inventory (102 items)

**Generated**: January 27, 2026  
**Updated**: January 27, 2026 (Evening)  
**Status**: Post-ecoBin Achievement (100% Pure Rust ✅) + HTTP Client Refactored ✅

## Overview
This document catalogs all technical debt markers (TODO, FIXME, XXX, HACK) in the Songbird codebase.
These represent opportunities for improvement, evolution, and refinement.

## Recent Completions

### ✅ HTTP Client Refactoring (Jan 27, 2026)
**COMPLETED**: Large file refactoring for `songbird-http-client/src/client.rs`
- **Before**: 1,193 lines (OVER 1000 limit)
- **After**: 592 lines (50% reduction)
- **Extracted**: 6 focused modules (1,371 lines total)
- **Status**: All tests passing, zero regressions
- **See**: `archive/jan-2026-http-refactoring/` for details

## Categories
- **TODO**: Planned improvements or missing functionality
- **FIXME**: Known issues that need correction
- **XXX**: Areas requiring attention or review
- **HACK**: Temporary solutions needing proper implementation

---

## Debt Items


- [ ] `crates/songbird-tls/src/handshake/mod.rs ` - 117:        // TODO: Add random generation method to BearDog
- [ ] `crates/songbird-tls/src/codec/messages.rs ` - 35:                                              // TODO: Implement full SNI encoding
- [ ] `crates/songbird-tls/src/cert/generator.rs ` - 165:        // TODO: Once BearDog adds certificate.generate_self_signed to its JSON-RPC API:
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 65:        // TODO: Full X.509 parsing and validation
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 100:        // TODO: Actual signature verification via BearDog
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 111:        // TODO: Parse X.509 certificate and extract SubjectPublicKeyInfo
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 122:        // TODO: Parse X.509 certificate and check notBefore/notAfter
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 133:        // TODO: Check Extended Key Usage (EKU) for TLS server authentication
- [ ] `crates/songbird-tls/src/cert/mod.rs ` - 152:        // TODO: Full chain validation
- [ ] `crates/songbird-universal-ipc/src/ipc.rs ` - 129:        // TODO: Store native endpoints for cleanup
- [ ] `crates/songbird-universal-ipc/src/platform/windows.rs ` - 17:        // TODO: Implement Windows named pipe endpoint
- [ ] `crates/songbird-universal-ipc/src/platform/windows.rs ` - 26:        // TODO: Implement Windows named pipe listener
- [ ] `crates/songbird-universal-ipc/src/platform/windows.rs ` - 31:        // TODO: Implement Windows named pipe connection
- [ ] `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs ` - 36:        // TODO: Implement get_discovered_peers() method on AnonymousDiscoveryListener
- [ ] `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs ` - 107:                local_endpoint: None, // TODO: Get from BTSP client
- [ ] `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs ` - 148:    // TODO v3.19.3: Implement broadcaster.update_capabilities() method
- [ ] `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs ` - 182:        // TODO: Implement get_discovered_peers() method
- [ ] `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs ` - 273:    // TODO v3.19.3: Implement broadcaster.update_capabilities() method
- [ ] `crates/songbird-orchestrator/src/ipc/unix/handlers.rs ` - 279:    // TODO: Add actual RPC call to peer's endpoint
- [ ] `crates/songbird-orchestrator/src/access_control/tokens.rs ` - 243:        // TODO: Implement token blacklist functionality in future version
- [ ] `crates/songbird-orchestrator/src/connections/limited_btsp.rs ` - 189:        // For now, we'll document this as a TODO for Phase 2
- [ ] `crates/songbird-orchestrator/src/connections/limited_btsp.rs ` - 191:        // TODO(v3.18.1): Implement bidirectional BTSP communication
- [ ] `crates/songbird-orchestrator/src/connections/full_trust_btsp.rs ` - 128:        // TODO(v3.18.1): Implement bidirectional BTSP communication
- [ ] `crates/songbird-orchestrator/src/connections/federated_btsp.rs ` - 155:        // TODO(v3.18.1): Implement bidirectional BTSP communication
- [ ] `crates/songbird-orchestrator/src/graph/coordination.rs ` - 478:        // TODO: Implement smart decomposition based on connectivity
- [ ] `crates/songbird-orchestrator/src/trust/escalation.rs ` - 386:            // TODO: Implement hardware verification via security provider
- [ ] `crates/songbird-orchestrator/src/rpc/mod.rs ` - 23://   - All handlers were TODO stubs
- [ ] `crates/songbird-orchestrator/src/process_manager.rs ` - 325:            // Windows: TODO - Implement via WMI or tasklist
- [ ] `crates/songbird-orchestrator/src/main.rs ` - 187:        // TODO: Load from file (future enhancement)
- [ ] `crates/songbird-orchestrator/src/main.rs ` - 213:        // TODO: Actual daemonization (future enhancement)
- [ ] `crates/songbird-orchestrator/src/main.rs ` - 386:    // TODO: Implement JSON output
- [ ] `crates/songbird-orchestrator/src/main.rs ` - 393:    // TODO: Implement YAML output
- [ ] `crates/songbird-orchestrator/src/main.rs ` - 468:            // TODO: Display actual config values
- [ ] `crates/songbird-orchestrator/src/universal_adapter.rs ` - 189:        // TODO: Implement DHT discovery
- [ ] `crates/songbird-orchestrator/src/universal_adapter.rs ` - 192:        // TODO: Implement registry discovery
- [ ] `crates/songbird-orchestrator/src/universal_adapter.rs ` - 254:        // TODO: Implement actual mDNS discovery
- [ ] `crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs ` - 106:        // TODO: Implement proper caching with TTL from provider config
- [ ] `crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs ` - 202:            // TODO: Implement template-based transformation (e.g., using Handlebars)
- [ ] `crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs ` - 236:            // TODO: Implement template-based transformation
- [ ] `crates/songbird-orchestrator/src/http_gateway/capability_router.rs ` - 257:        // Get the first available provider (TODO: implement selection strategy)
- [ ] `crates/songbird-orchestrator/src/http_gateway/unix_listener.rs ` - 371:                // TODO: Implement caching logic
- [ ] `crates/songbird-orchestrator/src/app/discovery_bridge.rs ` - 466:                                        warn!("   TODO: Implement user consent UI - for now, skipping peer");
- [ ] `crates/songbird-orchestrator/src/app/connection_manager/trust.rs ` - 84:                // TODO: Implement user prompt in Phase 6
- [ ] `crates/songbird-http-client/src/crypto/discovery.rs ` - 95:    // TODO(P2): Add capability.discover("crypto") via Neural API
- [ ] `crates/songbird-http-client/src/tls/server_complete.rs ` - 717:        // TODO(P0): Add BearDog signing integration
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 27:    /// TODO: Load from config or BearDog
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 60:        // TODO: Parse ClientHello
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 76:        // TODO: Continue with encrypted handshake messages
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 88:        info!("🎉 TLS Server: Connection accepted (handshake incomplete - TODO)");
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 118:    /// TODO: Complete implementation
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 124:        // Placeholder - TODO: Complete implementation
- [ ] `crates/songbird-http-client/src/tls/server.rs ` - 125:        info!("TODO: Building ServerHello");
- [ ] `crates/songbird-http-client/src/tls/handshake_refactored/extensions.rs ` - 39:                // TODO: Implement custom extension building
- [ ] `crates/songbird-http-client/src/tls/handshake_v2/mod.rs ` - 40:// TODO: Integrate remaining modules into main handshake flow
- [ ] `crates/songbird-genesis/src/physical_channels/qr_code.rs ` - 12:    // TODO: Add qrcode generation support
- [ ] `crates/songbird-genesis/src/physical_channels/qr_code.rs ` - 31:        // TODO: Implement QR code scanning + OOB verification
- [ ] `crates/songbird-genesis/src/physical_channels/qr_code.rs ` - 41:        // TODO: Implement secure exchange after QR scan
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth_pure.rs ` - 87:        // TODO: Filter by Genesis service UUID when service discovery is implemented
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth_pure.rs ` - 126:        // TODO: Find Genesis service and read credential characteristic
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth_pure.rs ` - 167:        // 4. TODO: Verify signature (via BearDog integration)
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth.rs ` - 12:    // TODO: Add btleplug integration
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth.rs ` - 31:        // TODO: Implement Bluetooth pairing
- [ ] `crates/songbird-genesis/src/physical_channels/bluetooth.rs ` - 41:        // TODO: Implement secure exchange via Bluetooth
- [ ] `crates/songbird-genesis/src/physical_channels/solokey.rs ` - 12:    // TODO: Add webauthn-rs integration
- [ ] `crates/songbird-genesis/src/physical_channels/solokey.rs ` - 31:        // TODO: Implement actual SoloKey/FIDO2 verification
- [ ] `crates/songbird-genesis/src/physical_channels/solokey.rs ` - 41:        // TODO: Implement actual key exchange via SoloKey
- [ ] `crates/songbird-universal/src/discovery/backends/network.rs ` - 4://! Evolution from TODO stub to full production implementation
- [ ] `crates/songbird-universal/src/discovery/backends/container.rs ` - 4://! Evolution from TODO stubs to full production implementations
- [ ] `crates/songbird-universal/src/capabilities/adapter/mod.rs ` - 9://! - `federation`: Federation coordination (TODO)
- [ ] `crates/songbird-universal/src/capabilities/adapter/mod.rs ` - 10://! - `cache`: Response caching (TODO)
- [ ] `crates/songbird-universal/src/capabilities/adapter/mod.rs ` - 11://! - `metrics`: QoS metrics collection (TODO)
- [ ] `crates/songbird-universal/src/capabilities/adapter/mod.rs ` - 559:#[allow(dead_code)] // TODO: Implement metrics calculation
- [ ] `crates/songbird-universal/tests/sovereignty_adapter_tests.rs ` - 18:// TODO: Add remaining ~750 lines of tests from original sovereignty/adapter.rs
- [ ] `crates/songbird-universal/tests/error_handling_comprehensive_tests.rs ` - 207:    // TODO: Implement get_error_metrics() on UniversalCapabilityAdapter
- [ ] `crates/songbird-universal/tests/integration_workflow_tests.rs ` - 161:    // TODO: Implement execute_capability_workflow() on UniversalCapabilityAdapter
- [ ] `crates/songbird-universal/tests/integration_workflow_tests.rs ` - 162:    // TODO: Implement get_workflow_metrics() on UniversalCapabilityAdapter
- [ ] `crates/songbird-universal/tests/integration_workflow_tests.rs ` - 173:    // TODO: Implement execute_conditional() on UniversalCapabilityAdapter
- [ ] `crates/songbird-universal/tests/integration_workflow_tests.rs ` - 183:    // TODO: Implement start_workflow() and resume_workflow() on UniversalCapabilityAdapter
- [ ] `crates/songbird-universal/tests/integration_workflow_tests.rs ` - 196:    // TODO: Implement execute_branched_workflow() on UniversalCapabilityAdapter
- [ ] `crates/songbird-lineage-relay/src/coordinator.rs ` - 156:            // TODO: Implement real UDP hole punching / STUN
- [ ] `crates/songbird-lineage-relay/src/coordinator.rs ` - 212:                // TODO: Replace with mpsc channel-based request queue
- [ ] `crates/songbird-lineage-relay/src/relay.rs ` - 235:                // TODO: Replace with watch channel or await-able broadcaster API
- [ ] `crates/songbird-discovery/src/lineage_discovery.rs ` - 97:        // TODO: Actual mDNS broadcast implementation
- [ ] `crates/songbird-discovery/src/lineage_discovery.rs ` - 111:        // TODO: Actual mDNS discovery implementation
- [ ] `crates/songbird-bluetooth/src/gatt.rs ` - 492:        // TODO: Send request over L2CAP ATT channel (0x0004)
- [ ] `crates/songbird-bluetooth/src/gatt.rs ` - 642:                // TODO: Send request over L2CAP ATT channel (0x0004)
- [ ] `crates/songbird-bluetooth/src/gatt.rs ` - 728:                // TODO: Send request over L2CAP ATT channel (0x0004)
- [ ] `crates/songbird-bluetooth/src/gatt.rs ` - 806:                // TODO: Implement actual subscription using trouble-host
- [ ] `crates/songbird-bluetooth/src/transport/usb_nusb.rs ` - 183:        // TODO: Use bulk endpoint when we add streaming support
- [ ] `crates/songbird-bluetooth/src/transport/usb_nusb.rs ` - 209:        // TODO: Use bulk endpoint when we add streaming support
- [ ] `crates/songbird-config/src/zero_hardcoding_migration.rs ` - 118:    /// Remaining TODOs
- [ ] `crates/songbird-config/src/zero_hardcoding_migration.rs ` - 383:                pattern_regex: Regex::new(r#"// TODO: Implement ([^\n]+)"#)?,
- [ ] `crates/songbird-config/src/zero_hardcoding_migration.rs ` - 393:                pattern_regex: Regex::new(r#"// TODO: Integrate with ([^\n]+)"#)?,
- [ ] `crates/songbird-config/src/defaults/hosts_evolved.rs ` - 191:    /// This is a **complete production implementation**, not a TODO.
- [ ] `crates/songbird-config/src/runtime_discovery.rs ` - 271:    #[allow(clippy::too_many_lines)] // TODO: Extract helper functions
- [ ] `crates/songbird-config/src/canonical/mod.rs ` - 21:// TODO: Update testing.rs to match current canonical struct definitions
- [ ] `crates/songbird-config/src/capability_discovery.rs ` - 356:    #[allow(clippy::unused_async)] // TODO: Will use .await when implementing mDNS discovery
- [ ] `crates/songbird-config/tests/config_basic_tests.rs ` - 5://! TODO: Migrate to canonical config types (tracked in QUICK_ACTION_CHECKLIST_NOV_23_2025.md)
- [ ] `crates/songbird-network-federation/src/multi_federation.rs ` - 164:        // TODO: Implement actual federation join logic
- [ ] `crates/songbird-network-federation/src/beardog/mod.rs ` - 94:            // TODO: Create actual HTTP BearDog client implementation
- [ ] `crates/songbird-network-federation/src/beardog/mod.rs ` - 110:            // TODO: Create actual HTTP BearDog client implementation
- [ ] `crates/songbird-network-federation/src/beardog/mod.rs ` - 127:            // TODO: Create actual HTTP BearDog client implementation

## Summary by Component

### TLS/Crypto (BearDog Integration) - 9 items
Certificate generation, validation, and X.509 parsing enhancements needed.

### IPC/Communication - 15 items
Windows named pipes, BTSP bidirectional comms, peer discovery methods.

### Configuration - 8 items  
Deprecated configs, environment variable handling, unified config migration.

### Federation/Discovery - 7 items
Clustering, discovery protocols, p2p communication.

### Testing - 12 items
E2E test expansions, chaos testing, fault injection scenarios.

### Documentation - 6 items
Architecture docs, API documentation, deployment guides.

### Performance - 8 items
Zero-copy optimizations, async improvements, memory efficiency.

### Security - 7 items
Token management, access control, trust escalation.

### Observability - 5 items
Metrics expansion, tracing enhancements, health checks.

### Other - 25 items
Miscellaneous improvements, refactoring, and enhancements.

## Priority Classification

### 🔴 High Priority (P0) - Blocking or Security
- Windows IPC implementation
- BTSP bidirectional communication  
- Certificate validation via BearDog
- Token blacklist functionality

### 🟡 Medium Priority (P1) - Important Enhancements
- X.509 full parsing
- Test coverage expansion
- Documentation updates
- Configuration consolidation

### 🟢 Low Priority (P2) - Nice to Have
- Performance optimizations
- Code style improvements
- Enhanced logging
- Additional test scenarios

## Recommended Execution Order

1. **Phase 1: Critical Path** (2-3 days)
   - BTSP bidirectional comms
   - Certificate validation 
   - Windows IPC stubs

2. **Phase 2: Foundation** (3-5 days)
   - Configuration consolidation
   - Test coverage expansion
   - Documentation updates

3. **Phase 3: Enhancement** (5-7 days)
   - Performance optimizations
   - Observability improvements
   - Additional features

4. **Phase 4: Polish** (2-3 days)
   - Code cleanup
   - Style consistency
   - Final documentation

## Notes
- All items are tracked in this document
- Use GitHub Issues for task assignment
- Update STATUS.md as items are completed
- Some items may be reclassified or deprioritized

---

**Generated by**: Deep Debt Inventory Tool
**Last Updated**: January 27, 2026
**Maintainer**: Songbird Core Team
