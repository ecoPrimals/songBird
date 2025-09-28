# 📊 Configuration Consolidation Report

**Generated**: 2025-09-26 16:12:22
**Total Configurations Found**: 612

## 🔍 Configuration Distribution by Category

- **Other**: 241 configs (39.4%)
- **Observability**: 64 configs (10.5%)
- **Discovery**: 63 configs (10.3%)
- **Network**: 54 configs (8.8%)
- **Security**: 48 configs (7.8%)
- **Performance**: 43 configs (7.0%)
- **Primal**: 36 configs (5.9%)
- **Gaming**: 25 configs (4.1%)
- **System**: 14 configs (2.3%)
- **Testing**: 13 configs (2.1%)
- **Federation**: 11 configs (1.8%)

## 🏗️ Configuration Distribution by Crate

- **songbird-config**: 172 configs
- **songbird-types**: 165 configs
- **songbird-orchestrator**: 91 configs
- **songbird-universal-primals**: 46 configs
- **songbird-discovery**: 39 configs
- **songbird-canonical**: 30 configs
- **songbird-universal**: 23 configs
- **songbird-test-utils**: 22 configs
- **songbird-cli**: 10 configs
- **songbird-network-federation**: 8 configs

## 🎯 Consolidation Opportunities (112)

1. Consolidate 5 variants of 'PrimalConfig' from crates: songbird-universal-primals, songbird-test-utils, songbird-config, songbird-types
2. Consolidate 10 variants of 'PerformanceConfig' from crates: songbird-config, songbird-types, songbird-orchestrator, songbird-test-utils, songbird-canonical, songbird-network-federation
3. Consolidate 2 variants of 'IntegrationConfig' from crates: songbird-test-utils
4. Consolidate 3 variants of 'ExecutionConfig' from crates: songbird-orchestrator, songbird-test-utils, songbird-discovery
5. Consolidate 5 variants of 'FederationConfig' from crates: songbird-test-utils, songbird-config, songbird-network-federation, songbird-types
6. Consolidate 7 variants of 'EnvironmentConfig' from crates: songbird-canonical, songbird-test-utils, songbird-config, songbird-types
7. Consolidate 12 variants of 'NetworkConfig' from crates: songbird-config, songbird-types, songbird-canonical, songbird-test-utils, songbird-network-federation, songbird-discovery
8. Consolidate 8 variants of 'SecurityConfig' from crates: songbird-config, songbird-cli, songbird-types, songbird-test-utils, songbird-universal-primals, songbird-universal
9. Consolidate 3 variants of 'ConsensusConfig' from crates: songbird-test-utils, songbird-config, songbird-types
10. Consolidate 5 variants of 'HealthConfig' from crates: songbird-canonical, songbird-test-utils, songbird-types
11. Consolidate 3 variants of 'CleanupConfig' from crates: songbird-orchestrator, songbird-test-utils, songbird-discovery
12. Consolidate 7 variants of 'TimeoutConfig' from crates: songbird-config, songbird-types, songbird-orchestrator, songbird-test-utils, songbird-universal-primals
13. Consolidate 2 variants of 'AuthConfig' from crates: songbird-test-utils, songbird-config
14. Consolidate 7 variants of 'EncryptionConfig' from crates: songbird-universal-primals, songbird-test-utils, songbird-config, songbird-types
15. Consolidate 2 variants of 'Config' from crates: songbird-test-utils, songbird-cli
16. Consolidate 6 variants of 'ServiceConfig' from crates: songbird-config, songbird-types
17. Consolidate 2 variants of 'OrchestrationConfig' from crates: songbird-canonical, songbird-types
18. Consolidate 2 variants of 'FederationDiscoveryConfig' from crates: songbird-discovery, songbird-types
19. Consolidate 3 variants of 'ResourceManagementConfig' from crates: songbird-orchestrator, songbird-discovery, songbird-types
20. Consolidate 3 variants of 'PortConfig' from crates: songbird-canonical, songbird-config, songbird-types

## 🔄 Migration Mappings (229)

- `TestPrimalConfig` → `CanonicalTestConfig`
- `PerformanceTestConfig` → `CanonicalPerformanceConfig`
- `IntegrationTestConfig` → `CanonicalTestConfig`
- `ErrorTestConfig` → `CanonicalTestConfig`
- `TestExecutionConfig` → `CanonicalTestConfig`
- `TestFederationConfig` → `CanonicalFederationConfig`
- `ChaosTestConfig` → `CanonicalTestConfig`
- `TestEnvironmentConfig` → `CanonicalSystemConfig`
- `TestNetworkConfig` → `CanonicalNetworkConfig`
- `TestSecurityConfig` → `CanonicalSecurityConfig`
- `TestDataConfig` → `CanonicalTestConfig`
- `TestConsensusConfig` → `CanonicalFederationConfig`
- `TestHealthConfig` → `CanonicalObservabilityConfig`
- `MetricsCollectionConfig` → `CanonicalObservabilityConfig`
- `TestCleanupConfig` → `CanonicalTestConfig`
- `TestTimeoutConfig` → `CanonicalTestConfig`
- `TestAuthConfig` → `CanonicalSecurityConfig`
- `TestEncryptionConfig` → `CanonicalSecurityConfig`
- `TestConfig` → `CanonicalTestConfig`
- `CanonicalNetworkConfig` → `CanonicalNetworkConfig`
