# 📊 Constants Consolidation Report

**Generated**: 2025-09-26 16:15:48
**Total Constants Found**: 452

## 🔍 Constants Distribution by Category

- **Network**: 106 constants (23.5%)
- **Timeout**: 101 constants (22.3%)
- **Size**: 96 constants (21.2%)
- **Other**: 90 constants (19.9%)
- **Discovery**: 18 constants (4.0%)
- **System**: 14 constants (3.1%)
- **Testing**: 8 constants (1.8%)
- **Gaming**: 8 constants (1.8%)
- **Performance**: 6 constants (1.3%)
- **Security**: 4 constants (0.9%)
- **Federation**: 1 constants (0.2%)

## 🏗️ Constants Distribution by Crate

- **songbird-config**: 217 constants
- **songbird-types**: 115 constants
- **songbird-test-utils**: 70 constants
- **songbird-orchestrator**: 35 constants
- **songbird-cli**: 12 constants
- **songbird-universal-primals**: 3 constants

## 🔄 Duplicate Constants (22)

### `DEFAULT_BIND_ADDRESS` (5 definitions)
- **songbird-test-utils**: `"127.0.0.1"` (&str)
- **songbird-types**: `"0.0.0.0"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)

### `DEFAULT_LOCALHOST` (4 definitions)
- **songbird-test-utils**: `"localhost"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)
- **songbird-config**: `"127.0.0.1"` (&str)

### `_` (23 definitions)
- **songbird-types**: `assert!(CanonicalNetworkLimits::MAX_CONNECTIONS > 0)` (())
- **songbird-types**: `assert!(CanonicalNetworkLimits::MAX_CONNECTIONS <= 10000)` (())
- **songbird-types**: `assert!(CanonicalNetworkLimits::CONNECTION_TIMEOUT_SECONDS > 0)` (())
- **songbird-types**: `assert!(CanonicalNetworkLimits::CONNECTION_TIMEOUT_SECONDS <= 300)` (())
- **songbird-types**: `assert!(CanonicalNetworkLimits::MAX_REQUEST_SIZE > 0)` (())
- **songbird-types**: `assert!(CanonicalNetworkLimits::MAX_REQUEST_SIZE <= 10_485_760)` (())
- **songbird-types**: `assert!(CanonicalResourceDefaults::DEFAULT_MEMORY_LIMIT > 0)` (())
- **songbird-types**: `assert!(CanonicalResourceDefaults::DEFAULT_MEMORY_LIMIT >= 1_073_741_824)` (())
- **songbird-types**: `assert!(CanonicalResourceDefaults::DEFAULT_CPU_LIMIT > 0.0)` (())
- **songbird-types**: `assert!(CanonicalResourceDefaults::DEFAULT_CPU_LIMIT <= 100.0)` (())
- **songbird-types**: `assert!(CanonicalResourceDefaults::DEFAULT_DISK_THRESHOLD > 0)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_RESPONSE_TIMEOUT_MS > 0)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_RESPONSE_TIMEOUT_MS <= 60000)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_RETRY_ATTEMPTS > 0)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_RETRY_ATTEMPTS <= 10)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_BATCH_SIZE > 0)` (())
- **songbird-types**: `assert!(CanonicalPerformanceDefaults::DEFAULT_BATCH_SIZE <= 1000)` (())
- **songbird-types**: `assert!(CanonicalDiscoveryDefaults::DISCOVERY_INTERVAL_SECONDS > 0)` (())
- **songbird-types**: `assert!(CanonicalDiscoveryDefaults::DISCOVERY_INTERVAL_SECONDS <= 300)` (())
- **songbird-types**: `assert!(CanonicalDiscoveryDefaults::HEALTH_CHECK_INTERVAL_SECONDS > 0)` (())
- **songbird-types**: `assert!(CanonicalDiscoveryDefaults::HEALTH_CHECK_INTERVAL_SECONDS <= 60)` (())
- **songbird-config**: `assert!(GAMING_BUFFER_SIZE > 0)` (())
- **songbird-config**: `assert!(MAX_GAMING_PACKET_SIZE <= 1500)` (())

### `DEFAULT_DISCOVERY_PORT` (3 definitions)
- **songbird-types**: `8001` (u16)
- **songbird-config**: `8081` (u16)
- **songbird-config**: `8001` (u16)

### `DEFAULT_DASHBOARD_PORT` (3 definitions)
- **songbird-types**: `8003` (u16)
- **songbird-config**: `3000` (u16)
- **songbird-config**: `8003` (u16)

### `DEFAULT_FEDERATION_PORT` (3 definitions)
- **songbird-types**: `8005` (u16)
- **songbird-config**: `8082` (u16)
- **songbird-config**: `8005` (u16)

### `DEFAULT_CONNECTION_TIMEOUT` (3 definitions)
- **songbird-types**: `Duration::from_secs(30)` (Duration)
- **songbird-config**: `Duration::from_millis(30000)` (Duration)
- **songbird-config**: `Duration::from_secs(30)` (Duration)

### `DEFAULT_RETRY_DELAY` (3 definitions)
- **songbird-types**: `Duration::from_secs(1)` (Duration)
- **songbird-config**: `Duration::from_millis(1000)` (Duration)
- **songbird-config**: `Duration::from_secs(1)` (Duration)

### `DEFAULT_HEALTH_CHECK_TIMEOUT` (2 definitions)
- **songbird-types**: `Duration::from_secs(10)` (Duration)
- **songbird-config**: `Duration::from_secs(5)` (Duration)

### `DEFAULT_PLAYER_TIMEOUT` (2 definitions)
- **songbird-types**: `std::time::Duration::from_secs(300)` (std::time::Duration)
- **songbird-config**: `Duration::from_secs(300)` (Duration)


## 🎯 Consolidation Opportunities (13)

1. Consolidate 106 network constants into unified_network_constants module
2. Consolidate 101 timeout constants into unified_timeout_constants module
3. Consolidate 18 discovery constants into unified_discovery_constants module
4. Consolidate 8 testing constants into unified_testing_constants module
5. Consolidate 96 size constants into unified_size_constants module
6. Consolidate 8 gaming constants into unified_gaming_constants module
7. Consolidate 14 system constants into unified_system_constants module
8. Consolidate 6 performance constants into unified_performance_constants module
9. Centralize 70 scattered constants from songbird-test-utils crate
10. Centralize 35 scattered constants from songbird-types crate
11. Centralize 35 scattered constants from songbird-orchestrator crate
12. Centralize 12 scattered constants from songbird-cli crate
13. Centralize 217 scattered constants from songbird-config crate
