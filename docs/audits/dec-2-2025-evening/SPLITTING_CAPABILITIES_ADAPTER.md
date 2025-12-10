# Splitting capabilities/adapter.rs (1120 lines → <1000)

## Current Structure (1120 lines)
- RegistrationHandle (~50 lines)
- UniversalCapabilityAdapter (~980 lines)
  - Discovery methods (~380 lines)
  - Connection management (~180 lines)
  - Service registry (~270 lines)
- Simple helper types (~50 lines)
- Test utilities (~30 lines)

## New Structure

### adapter.rs (Core - ~250 lines)
- RegistrationHandle struct + impl
- UniversalCapabilityAdapter struct + new()
- Re-exports from submodules
- Core public API

### adapter/discovery.rs (~400 lines)
- discover_primal_capabilities()
- find_capability_providers()
- discover_capability_providers_from_env()
- discover_capability_providers_from_network()
- infer_capability_providers()
- get_best_primal_for_capability()
- query_primal_capabilities()

### adapter/connections.rs (~200 lines)
- connect_to_primal()
- test_primal_health()
- get_active_connections()
- disconnect_from_primal()
- update_connection_health()

### adapter/registry.rs (~270 lines)
- register_service()
- deregister_service()
- discover_capability_providers()
- update_service_health()
- get_service_health()
- execute_capability_request()

### adapter/types.rs (~50 lines)
- SimpleServiceHealth
- SimpleCapabilityRequest
- SimpleCapabilityResponse
- SimpleServiceInfo
- ServiceLike trait

## Status: IN PROGRESS

This split will maintain 100% API compatibility while improving maintainability.

