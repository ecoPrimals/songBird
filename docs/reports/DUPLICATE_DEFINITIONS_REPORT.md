# Duplicate Definitions Report
**Generated**: $(date)
**Purpose**: Identify exact duplicates for consolidation

---

## Config Struct Duplicates

### Summary

     19 HealthCheckConfig
     14 CircuitBreakerConfig
     13 DiscoveryConfig
      9 RetryConfig
      8 SecurityConfig
      8 PerformanceConfig
      8 NetworkConfig
      7 CanonicalDiscoveryConfig
      7 CacheConfig
      6 ServiceDiscoveryConfig
      5 ServiceConfig
      5 LoggingConfig
      5 LoadBalancingConfig
      5 EncryptionConfig
      4 ZeroTouchConfig
      4 TracingConfig
      4 TimeoutConfig
      4 RateLimitingConfig
      4 NetworkDiscoveryConfig
      4 MonitoringConfig
      4 HealthMonitoringConfig
      4 EnvironmentConfig
      4 ConnectionPoolConfig
      4 CanonicalNetworkConfig
      3 ZeroCostRouterConfig
      3 SessionConfig
      3 ServiceMeshConfig
      3 ScalingConfig
      3 RobustnessConfig
      3 ResourceManagementConfig
      3 ResourceConfig
      3 RateLimitConfig
      3 ProxyConfig
      3 ProviderConfig
      3 PrimalConfiguration
      3 OrchestratorConfig
      3 LoadBalancerConfig
      3 GamingSecurityConfig
      3 GamingNetworkConfig
      3 FederationConfig
      3 ErrorHandlingConfig
      3 DashboardConfig
      3 ConnectionConfig
      3 ConfigProviderInfo
      3 CliConfig
      3 CanonicalTimeoutConfig
      3 CanonicalPerformanceConfig
      3 CanonicalEnvironmentConfig
      3 CanonicalCircuitBreakerConfig
      3 BulkheadConfig
      3 BenchmarkConfig
      3 AuthenticationConfig
      2 WebSocketConfig
      2 ValidationConfig
      2 ValidationCacheConfig
      2 UniversalAdapterConfig
      2 UnifiedObservabilityConfig
      2 TrustConfig
      2 TrackingConfig
      2 TlsConfig
      2 StorageConfig
      2 ServiceFailureConfig
      2 SecurityProviderIntegrationConfig
      2 ScalabilityConfig
      2 ResourceConstraintConfig
      2 RegistryConfig
      2 ProtocolDetectionConfig
      2 PrimalHealthConfig
      2 PrimalDiscoveryEngine
      2 PrimalConfig
      2 PortConfig
      2 PerformanceDegradationConfig
      2 PerformanceAnalysisConfig
      2 ObservabilityConfig
      2 NetworkPerformanceConfig
      2 NetworkFaultConfig
      2 MonitoringConfiguration
      2 MetricsConfig
      2 LogRotationConfig
      2 LogConfig
      2 LimitsConfig
      2 IntegrationTestConfig
      2 HookSystemConfig
      2 HookConfig
      2 HealthCheckConfiguration
      2 GamingPerformanceConfig
      2 GamingConfig
      2 GamingBridgeConfig
      2 FlagMonitoringConfig
      2 FeatureFlagConfig
      2 ExperimentConfig
      2 ExecutionConfig
      2 EvaluationConfig
      2 DiscoveryTimingConfig
      2 DiscoveryPerformanceConfig
      2 DiscoveryMechanismsConfig
      2 ConsensusConfig
      2 ConfigValidator;
      2 ConfigValidationResult
      2 ConfigMetadata
      2 CleanupConfig
      2 CapabilityDiscoveryConfig
      2 CanonicalSystemConfig
      2 CanonicalSecurityConfig
      2 CanonicalRetryConfig
      2 CanonicalPrimalConfig
      2 CanonicalObservabilityConfig
      2 CanonicalMonitoringConfig
      2 CanonicalHealthMonitoringConfig
      2 CanonicalHealthConfig
      2 CanonicalHealthCheckConfig
      2 CanonicalGamingConfig
      2 CanonicalFederationConfig
      2 CanonicalConnectionPoolConfig
      2 CachingConfig
      2 ByzantineFailureConfig
      2 BootstrapConfig
      2 ApiConfig

### Detailed Locations


#### HealthCheckConfig (19 definitions)
```
crates/songbird-types/src/config/performance.rs:573:pub struct HealthCheckConfig {
crates/songbird-types/src/config/discovery_corrupted.rs:245:pub struct HealthCheckConfig {
crates/songbird-types/src/config/discovery.rs:247:pub struct HealthCheckConfig {
crates/songbird-orchestrator/src/core/robustness/config.rs:189:pub struct HealthCheckConfig {
crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs:213:pub struct HealthCheckConfiguration {
crates/songbird-orchestrator/src/core/traits/health.rs:69:pub struct HealthCheckConfig {
crates/songbird-observability/src/health/production_health.rs:60:pub struct HealthCheckConfig  {/// Check interval
crates/songbird-registry/src/types/health.rs:119:pub struct HealthCheckConfig {
crates/songbird-universal/src/types/config.rs:117:pub struct HealthCheckConfig {
crates/songbird-discovery/src/traits/health.rs:26:pub struct HealthCheckConfig {
crates/songbird-config/src/config/mod.rs:495:pub struct HealthCheckConfig {
crates/songbird-config/src/config/universal_primals.rs:183:// pub struct HealthCheckConfig { ... }
crates/songbird-config/src/config/universal_primals_clean.rs:38:pub struct HealthCheckConfig  {/// Enable health checks
crates/songbird-config/src/canonical/resilience.rs:406:pub struct HealthCheckConfig {
crates/songbird-config/src/canonical/service.rs:36:pub struct HealthCheckConfig {
crates/songbird-config/src/canonical/primals.rs:242:pub struct HealthCheckConfig {
crates/songbird-config/src/unified/robustness.rs:265:pub struct HealthCheckConfig  {/// Enable health checks
crates/songbird-config/src/unified/api.rs:225:pub struct HealthCheckConfiguration  {/// Health check endpoint path
crates/songbird-primal-sdk/src/modern_api.rs:388:pub struct HealthCheckConfig  {/// Health check endpoint path
crates/songbird-primal-sdk/src/config.rs:162:pub struct HealthCheckConfig  {/// Health check endpoint
crates/songbird-primal-sdk/src/universal_registry/config.rs:28:pub struct HealthCheckConfig  {pub path: String,
```

#### CircuitBreakerConfig (14 definitions)
```
crates/songbird-types/src/config/communication.rs:321:pub struct CircuitBreakerConfig {
crates/songbird-orchestrator/src/core/robustness/config.rs:26:pub struct CircuitBreakerConfig {
crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs:364:pub struct CircuitBreakerConfig {
crates/songbird-universal/src/types/config.rs:93:pub struct CircuitBreakerConfig {
crates/songbird-universal/src/circuit_breaker.rs:23:pub struct CircuitBreakerConfig {
crates/songbird-canonical/src/config/adapters.rs:173:pub struct CircuitBreakerConfig {
crates/songbird-config/src/canonical/resilience.rs:8:pub struct CircuitBreakerConfig {/// Number of consecutive failures before opening the circuit
crates/songbird-config/src/canonical/network.rs:788:pub struct CircuitBreakerConfig {
crates/songbird-config/src/unified/robustness.rs:38:pub struct CircuitBreakerConfig  {/// Enable circuit breaker
crates/songbird-config/src/unified/api.rs:255:pub struct CircuitBreakerConfig  {/// Failure threshold to open circuit
crates/songbird-primal-sdk/src/modern_api.rs:341:pub struct CircuitBreakerConfig  {/// Failure threshold to open circuit
crates/songbird-primal-sdk/src/config.rs:291:pub struct CircuitBreakerConfig  {/// Whether to enable circuit breaker
crates/songbird-primal-sdk/src/universal_registry/config.rs:64:pub struct CircuitBreakerConfig  {pub failure_threshold: u32,
crates/songbird-primal-sdk/src/modern_api/mod.rs:114:pub struct CircuitBreakerConfig  {/// Failure Threshold field
```

#### DiscoveryConfig (13 definitions)
```
crates/songbird-universal/src/infant_discovery.rs:133:pub struct DiscoveryConfig  {/// Network ranges to scan
crates/songbird-universal/src/agnostic_service_discovery.rs:124:pub struct DiscoveryConfig  {/// Maximum time to spend on discovery
crates/songbird-universal/src/capabilities/types.rs:56:pub struct DiscoveryConfig {
crates/songbird-universal/src/discovery.rs:46:pub struct DiscoveryConfig {
crates/songbird-discovery/src/abstraction/modernized_factory.rs:261:pub struct DiscoveryConfigBuilder  {configs: Vec<ProviderConfig>)
crates/songbird-discovery/src/traits/discovery.rs:252:pub struct DiscoveryConfig {
crates/songbird-config/src/zero_touch_config.rs:122:pub struct DiscoveryConfig {
crates/songbird-config/src/zero_touch/infant_config.rs:127:pub struct DiscoveryConfig {
crates/songbird-config/src/config/mod.rs:460:pub struct DiscoveryConfig {
crates/songbird-config/src/canonical/discovery.rs:29:pub struct DiscoveryConfig {
crates/songbird-primal-sdk/src/discovery/types.rs:88:pub struct DiscoveryConfig  {/// Enable network scanning
crates/songbird-primal-sdk/src/discovery/universal_discovery/types.rs:10:pub struct DiscoveryConfig  {/// Enable automatic discovery
crates/songbird-primal-sdk/src/adaptive_discovery.rs:766:pub struct DiscoveryConfig {
crates/songbird-network-federation/src/network/mod.rs:293:pub struct DiscoveryConfig {
```

#### RetryConfig (9 definitions)
```
crates/songbird-orchestrator/src/core/robustness/config.rs:41:pub struct RetryConfig {
crates/songbird-orchestrator/src/core/traits/hooks.rs:247:pub struct RetryConfig {
crates/songbird-universal/src/types/config.rs:69:pub struct RetryConfig {
crates/songbird-universal/src/network_effects_decoupling.rs:151:pub struct RetryConfig  {/// Maximum retry attempts
crates/songbird-discovery/src/traits/hooks.rs:227:pub struct RetryConfig  {/// Enable retries on failure
crates/songbird-config/src/canonical/resilience.rs:34:pub struct RetryConfig {/// Maximum number of retry attempts
crates/songbird-config/src/unified/robustness.rs:163:pub struct RetryConfig  {/// Enable retry mechanism
crates/songbird-primal-sdk/src/modern_api.rs:356:pub struct RetryConfig  {/// Maximum number of retries
crates/songbird-primal-sdk/src/modern_api/mod.rs:121:pub struct RetryConfig  {/// Maximum number of retry attempts
```

#### SecurityConfig (8 definitions)
```
crates/songbird-execution-agent/src/security_sovereign.rs:249:pub struct SecurityConfig {
crates/songbird-universal/src/types/config.rs:16:pub struct SecurityConfig {
crates/songbird-cli/src/cli/commands/firewall.rs:33:pub struct SecurityConfig  {pub security_level: String,
crates/songbird-cli/src/cli/commands/quick.rs:36:pub struct SecurityConfig {
crates/songbird-config/src/config/mod.rs:301:pub struct SecurityConfig {
crates/songbird-config/src/config/hardcoded_elimination.rs:42:pub struct SecurityConfig {
crates/songbird-primal-sdk/src/security_provider.rs:89:pub struct SecurityConfig  {/// Discovery refresh interval in seconds
crates/songbird-primal-sdk/src/config.rs:120:pub struct SecurityConfig  {/// Whether to enable TLS
```

#### PerformanceConfig (8 definitions)
```
crates/songbird-types/src/performance/mod.rs:217:pub struct PerformanceConfig<const FAST_MODE: bool, const DEBUG_MODE: bool>;
crates/songbird-types/src/config/communication.rs:351:pub struct PerformanceConfig {
crates/songbird-orchestrator/src/core/mod.rs:157:pub struct PerformanceConfig {
crates/songbird-canonical/src/config/performance.rs:10:pub struct PerformanceConfig {
crates/songbird-config/src/lib.rs:155:pub struct PerformanceConfig {
crates/songbird-config/src/config/hardcoded_elimination.rs:73:pub struct PerformanceConfig {
crates/songbird-config/src/performance.rs:15:pub struct PerformanceConfigCache  {/// Cached canonical endpoints (avoid repeated string allocations)
crates/songbird-config/src/canonical/performance.rs:30:pub struct PerformanceConfig {
crates/songbird-network-federation/src/network/mod.rs:329:pub struct PerformanceConfig {
```

#### NetworkConfig (8 definitions)
```
crates/songbird-discovery/src/discovery/config/mod.rs:26:pub struct NetworkConfig {
crates/songbird-canonical/src/config/environment.rs:107:pub struct NetworkConfig {
crates/songbird-config/src/zero_touch_config.rs:188:pub struct NetworkConfig {
crates/songbird-config/src/zero_touch/environment.rs:331:pub struct NetworkConfig  {pub interfaces: Vec<String>,
crates/songbird-config/src/zero_touch/infant_config.rs:193:pub struct NetworkConfig {
crates/songbird-config/src/config/mod.rs:237:pub struct NetworkConfig {
crates/songbird-config/src/config/hardcoded_elimination.rs:51:pub struct NetworkConfig {
crates/songbird-network-federation/src/network/mod.rs:142:pub struct NetworkConfig {
```

#### CanonicalDiscoveryConfig (7 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/discovery.rs:15:pub struct CanonicalDiscoveryConfig {
crates/songbird-types/src/config/discovery_corrupted.rs:11:pub struct CanonicalDiscoveryConfig {
crates/songbird-types/src/config/discovery.rs:10:pub struct CanonicalDiscoveryConfig {
crates/songbird-types/src/adapters/canonical.rs:135:pub struct CanonicalDiscoveryConfig {
crates/songbird-orchestrator/src/core/basic_iot/mod.rs:29:pub struct CanonicalDiscoveryConfig {
crates/songbird-orchestrator/src/core/traits/discovery.rs:212:pub struct CanonicalDiscoveryConfig {
crates/songbird-discovery/src/discovery/core.rs:12:pub struct CanonicalDiscoveryConfig {
```

#### CacheConfig (7 definitions)
```
crates/songbird-orchestrator/src/core/api/ai_optimized/cache.rs:68:pub struct CacheConfig {
crates/songbird-orchestrator/src/core/caching/advanced_cache.rs:50:pub struct CacheConfig {
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:418:pub struct CacheConfig {
crates/songbird-discovery/src/discovery/backends/service_discovery.rs:68:pub struct CacheConfig {
crates/songbird-discovery/src/traits/feature_flags.rs:396:pub struct CacheConfig {
crates/songbird-config/src/canonical/performance.rs:182:pub struct CacheConfig {
crates/songbird-primal-sdk/src/storage/cache.rs:55:pub struct CacheConfig  {/// Maximum number of entries in cache
```

#### ServiceDiscoveryConfig (6 definitions)
```
crates/songbird-types/src/config/discovery_corrupted.rs:209:pub struct ServiceDiscoveryConfig {
crates/songbird-types/src/config/environment.rs:87:pub struct ServiceDiscoveryConfig {
crates/songbird-types/src/config/discovery.rs:211:pub struct ServiceDiscoveryConfig {
crates/songbird-universal/src/service_discovery.rs:16:pub struct ServiceDiscoveryConfig  {/// Discovery interval in seconds
crates/songbird-canonical/src/config/orchestration.rs:25:pub struct ServiceDiscoveryConfig {
crates/songbird-config/src/canonical/discovery.rs:98:pub struct ServiceDiscoveryConfig {
```

#### ServiceConfig (5 definitions)
```
crates/songbird-config/src/zero_touch/deployment.rs:511:pub struct ServiceConfig  {pub name: String,
crates/songbird-config/src/config/hardcoded_elimination.rs:33:pub struct ServiceConfig {
crates/songbird-config/src/performance.rs:227:pub struct ServiceConfig  {/// The canonical name identifier for the service
crates/songbird-config/src/canonical/service.rs:11:pub struct ServiceConfig {
crates/songbird-config/src/unified/core.rs:26:pub struct ServiceConfig {
```

#### LoggingConfig (5 definitions)
```
crates/songbird-canonical/src/config/environment.rs:59:pub struct LoggingConfig {
crates/songbird-cli/src/cli/commands/firewall.rs:50:pub struct LoggingConfig  {pub enabled: bool,
crates/songbird-config/src/config/mod.rs:602:pub struct LoggingConfig {
crates/songbird-config/src/canonical/observability.rs:54:pub struct LoggingConfig {
crates/songbird-config/src/unified/observability.rs:76:pub struct LoggingConfig {
```

#### LoadBalancingConfig (5 definitions)
```
crates/songbird-types/src/config/performance.rs:536:pub struct LoadBalancingConfig {
crates/songbird-orchestrator/src/core/mod.rs:139:pub struct LoadBalancingConfig {
crates/songbird-universal/src/types/config.rs:45:pub struct LoadBalancingConfig {
crates/songbird-canonical/src/config/orchestration.rs:38:pub struct LoadBalancingConfig {
crates/songbird-config/src/canonical/network.rs:608:pub struct LoadBalancingConfig {
```

#### EncryptionConfig (5 definitions)
```
crates/songbird-types/src/config/gaming.rs:427:pub struct EncryptionConfig {
crates/songbird-types/src/config/security.rs:104:pub struct EncryptionConfig {
crates/songbird-config/src/config/mod.rs:386:pub struct EncryptionConfig {
crates/songbird-config/src/canonical/security.rs:303:pub struct EncryptionConfig {
crates/songbird-primal-sdk/src/config.rs:276:pub struct EncryptionConfig  {/// Whether to enable encryption
```

#### ZeroTouchConfig (4 definitions)
```
crates/songbird-orchestrator/src/core/mod.rs:219:pub struct ZeroTouchConfig {
crates/songbird-config/src/zero_touch_config.rs:27:pub struct ZeroTouchConfig {
crates/songbird-config/src/zero_touch/mod.rs:16:pub struct ZeroTouchConfig {
crates/songbird-config/src/zero_touch/infant_config.rs:27:pub struct ZeroTouchConfig {
```

#### TracingConfig (4 definitions)
```
crates/songbird-config/src/config/mod.rs:575:pub struct TracingConfig {
crates/songbird-config/src/canonical/observability.rs:104:pub struct TracingConfig {
crates/songbird-config/src/unified/observability.rs:113:pub struct TracingConfig {
crates/songbird-primal-sdk/src/config.rs:336:pub struct TracingConfig  {/// Whether to enable tracing
```

#### TimeoutConfig (4 definitions)
```
crates/songbird-orchestrator/src/core/robustness/config.rs:88:pub struct TimeoutConfig {
crates/songbird-config/src/config/hardcoded_elimination.rs:64:pub struct TimeoutConfig {
crates/songbird-config/src/canonical/network.rs:1151:pub struct TimeoutConfig {
crates/songbird-primal-sdk/src/config.rs:306:pub struct TimeoutConfig  {/// Default request timeout
```

#### RateLimitingConfig (4 definitions)
```
crates/songbird-orchestrator/src/core/robustness/config.rs:121:pub struct RateLimitingConfig {
crates/songbird-config/src/canonical/resilience.rs:226:pub struct RateLimitingConfig {
crates/songbird-config/src/canonical/network.rs:632:pub struct RateLimitingConfig {
crates/songbird-config/src/unified/robustness.rs:76:pub struct RateLimitingConfig  {/// Enable rate limiting
```

#### NetworkDiscoveryConfig (4 definitions)
```
crates/songbird-types/src/config/network.rs:366:pub struct NetworkDiscoveryConfig {
crates/songbird-types/src/config/discovery_corrupted.rs:155:pub struct NetworkDiscoveryConfig {
crates/songbird-types/src/config/discovery.rs:154:pub struct NetworkDiscoveryConfig {
crates/songbird-config/src/canonical/discovery.rs:216:pub struct NetworkDiscoveryConfig {
```

#### MonitoringConfig (4 definitions)
```
crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs:391:pub struct MonitoringConfiguration {
crates/songbird-orchestrator/src/core/traits/resource_management.rs:374:pub struct MonitoringConfig {
crates/songbird-discovery/src/discovery/config/mod.rs:40:pub struct MonitoringConfig {
crates/songbird-discovery/src/traits/resource_management.rs:346:pub struct MonitoringConfig  {pub monitoring_interval: Duration,
crates/songbird-config/src/unified/api.rs:281:pub struct MonitoringConfiguration  {/// Enable monitoring
crates/songbird-primal-sdk/src/config.rs:321:pub struct MonitoringConfig  {/// Whether to enable metrics collection
```

#### HealthMonitoringConfig (4 definitions)
```
crates/songbird-orchestrator/src/core/biome/modules/lifecycle.rs:86:pub struct HealthMonitoringConfig {
crates/songbird-orchestrator/src/core/api/ai_mesh/mesh.rs:120:pub struct HealthMonitoringConfig {
crates/songbird-config/src/unified/api.rs:131:pub struct HealthMonitoringConfig  {/// Health check interval
crates/songbird-primal-sdk/src/config.rs:258:pub struct HealthMonitoringConfig  {/// Whether to enable health monitoring
```

#### EnvironmentConfig (4 definitions)
```
crates/songbird-canonical/src/config/environment.rs:11:pub struct EnvironmentConfig {
crates/songbird-config/src/config/environment.rs:182:pub struct EnvironmentConfig {
crates/songbird-config/src/canonical/environment.rs:105:pub struct EnvironmentConfig {
crates/songbird-config/src/unified/core.rs:50:pub struct EnvironmentConfig {
```

#### ConnectionPoolConfig (4 definitions)
```
crates/songbird-types/src/config/communication.rs:465:pub struct ConnectionPoolConfig {
crates/songbird-config/src/config/universal_primals.rs:326:pub struct ConnectionPoolConfig {
crates/songbird-config/src/canonical/network.rs:653:pub struct ConnectionPoolConfig {
crates/songbird-primal-sdk/src/config.rs:147:pub struct ConnectionPoolConfig  {/// Maximum number of connections
```

#### CanonicalNetworkConfig (4 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/network.rs:18:pub struct CanonicalNetworkConfig {
crates/songbird-types/src/config/network.rs:9:pub struct CanonicalNetworkConfig {
crates/songbird-types/src/unified.rs:231:pub struct CanonicalNetworkConfig {
crates/songbird-config/src/canonical/network.rs:125:pub struct CanonicalNetworkConfig {
```

#### ZeroCostRouterConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/zero_cost_request_router.rs:60:pub struct ZeroCostRouterConfig {
crates/songbird-config/src/canonical/resilience.rs:444:pub struct ZeroCostRouterConfig {
crates/songbird-config/src/unified/robustness.rs:299:pub struct ZeroCostRouterConfig  {/// Enable zero-cost routing
```

#### SessionConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/api/real_time_ai_streaming/session.rs:399:pub struct SessionConfiguration {
crates/songbird-config/src/canonical/security.rs:280:pub struct SessionConfig {
crates/songbird-config/src/unified/api.rs:29:pub struct SessionConfig  {/// Maximum concurrent sessions
crates/songbird-network-federation/src/network/gaming.rs:223:pub struct SessionConfig {
```

#### ServiceMeshConfig (3 definitions)
```
crates/songbird-discovery/src/agnostic_service_mesh.rs:128:pub struct ServiceMeshConfig  {/// Detection patterns to use
crates/songbird-primal-sdk/src/capability_compute.rs:108:pub struct ServiceMeshConfig {
crates/songbird-primal-sdk/src/toadstool.rs:376:pub struct ServiceMeshConfig  {pub name: String,
```

#### ScalingConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/mod.rs:197:pub struct ScalingConfig {
crates/songbird-canonical/src/config/orchestration.rs:81:pub struct ScalingConfig {
crates/songbird-primal-sdk/src/config.rs:213:pub struct ScalingConfig  {/// Whether to enable auto-scaling
```

#### RobustnessConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/robustness/config.rs:8:pub struct RobustnessConfig {
crates/songbird-config/src/canonical/resilience.rs:191:pub struct RobustnessConfig {
crates/songbird-config/src/unified/robustness.rs:11:pub struct RobustnessConfig  {/// Circuit breaker configuration
```

#### ResourceManagementConfig (3 definitions)
```
crates/songbird-types/src/config/federation.rs:517:pub struct ResourceManagementConfig {
crates/songbird-orchestrator/src/core/traits/resource_management.rs:316:pub struct ResourceManagementConfig {
crates/songbird-discovery/src/traits/resource_management.rs:232:pub struct ResourceManagementConfig  {/// Resource tracking configuration
```

#### ResourceConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/scalability/types.rs:91:pub struct ResourceConfig {
crates/songbird-orchestrator/src/core/traits/resource_management.rs:90:pub struct ResourceConfig {
crates/songbird-discovery/src/traits/resource_management.rs:72:pub struct ResourceConfig  {/// Maximum memory usage in bytes
```

#### RateLimitConfig (3 definitions)
```
crates/songbird-types/src/config/network.rs:174:pub struct RateLimitConfig {
crates/songbird-config/src/config/mod.rs:412:pub struct RateLimitConfig {
crates/songbird-primal-sdk/src/universal_registry/config.rs:71:pub struct RateLimitConfig  {pub strategy: RateLimitStrategy,
```

#### ProxyConfig (3 definitions)
```
crates/songbird-config/src/config/mod.rs:293:pub struct ProxyConfig {
crates/songbird-config/src/canonical/network.rs:1241:pub struct ProxyConfig {
crates/songbird-network-federation/src/network/mod.rs:249:pub struct ProxyConfig {
```

#### ProviderConfig (3 definitions)
```
crates/songbird-types/src/traits/canonical.rs:377:pub struct ProviderConfig {
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:400:pub struct ProviderConfig {
crates/songbird-discovery/src/traits/feature_flags.rs:385:pub struct ProviderConfig {
```

#### PrimalConfiguration (3 definitions)
```
crates/songbird-config/src/config/universal_primals.rs:68:pub struct PrimalConfiguration {
crates/songbird-config/src/config/universal_primals.rs:230:pub struct PrimalConfigurationTemplate {
crates/songbird-config/src/config/universal_primals_clean.rs:107:pub struct PrimalConfiguration  {/// Primal identifier
crates/songbird-config/src/canonical/primals.rs:329:pub struct PrimalConfiguration {
```

#### OrchestratorConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/orchestrator.rs:9:pub struct OrchestratorConfig {
crates/songbird-orchestrator/src/core/biome/modules/types.rs:626:pub struct OrchestratorConfig {
crates/songbird-cli/src/cli/commands/quick.rs:19:pub struct OrchestratorConfig {
```

#### LoadBalancerConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/load_balancer/types.rs:52:pub struct LoadBalancerConfig {
crates/songbird-config/src/canonical/resilience.rs:349:pub struct LoadBalancerConfig {
crates/songbird-config/src/unified/robustness.rs:217:pub struct LoadBalancerConfig  {/// Load balancing algorithm
```

#### GamingSecurityConfig (3 definitions)
```
crates/songbird-types/src/config/network.rs:223:pub struct GamingSecurityConfig {
crates/songbird-types/src/config/gaming.rs:133:pub struct GamingSecurityConfig {
crates/songbird-config/src/gaming.rs:94:pub struct GamingSecurityConfig  {/// Enable packet filtering
```

#### GamingNetworkConfig (3 definitions)
```
crates/songbird-types/src/config/network.rs:26:pub struct GamingNetworkConfig {
crates/songbird-types/src/config/gaming.rs:78:pub struct GamingNetworkConfig {
crates/songbird-config/src/canonical/network.rs:217:pub struct GamingNetworkConfig {
```

#### FederationConfig (3 definitions)
```
crates/songbird-discovery/src/discovery/enhanced_discovery.rs:47:pub struct FederationConfig {
crates/songbird-config/src/config/hardcoded_elimination.rs:93:pub struct FederationConfig {
crates/songbird-network-federation/src/federation.rs:247:pub struct FederationConfig {
```

#### ErrorHandlingConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/structural_improvements/config.rs:62:pub struct ErrorHandlingConfig {
crates/songbird-orchestrator/src/core/traits/validation.rs:471:pub struct ErrorHandlingConfig {
crates/songbird-discovery/src/traits/validation.rs:412:pub struct ErrorHandlingConfig  {pub logging_options: LoggingOptions,
```

#### DashboardConfig (3 definitions)
```
crates/songbird-observability/src/observability/advanced_dashboard.rs:63:pub struct DashboardConfig  {/// Dashboard update interval
crates/songbird-config/src/canonical/observability.rs:23:pub struct DashboardConfig {
crates/songbird-config/src/unified/observability.rs:47:pub struct DashboardConfig {
```

#### ConnectionConfig (3 definitions)
```
crates/songbird-types/src/config/network.rs:343:pub struct ConnectionConfig {
crates/songbird-orchestrator/src/core/api/real_time_ai_streaming/connection.rs:122:pub struct ConnectionConfig {
crates/songbird-config/src/unified/api.rs:63:pub struct ConnectionConfig  {/// Maximum connections per client
```

#### ConfigProviderInfo (3 definitions)
```
crates/songbird-orchestrator/src/core/traits/config.rs:33:pub struct ConfigProviderInfo {
crates/songbird-discovery/src/traits/config.rs:45:pub struct ConfigProviderInfo {
crates/songbird-config/src/config/providers.rs:30:pub struct ConfigProviderInfo {
```

#### CliConfig (3 definitions)
```
crates/songbird-orchestrator/src/cli/config.rs:11:pub struct CliConfig {
crates/songbird-cli/src/cli/config.rs:10:pub struct CliConfig {
crates/songbird-cli/src/cli/core/types.rs:11:pub struct CliConfig {
```

#### CanonicalTimeoutConfig (3 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/network.rs:226:pub struct CanonicalTimeoutConfig {
crates/songbird-types/src/config/adapters.rs:427:pub struct CanonicalTimeoutConfig {
crates/songbird-types/src/adapters/canonical.rs:190:pub struct CanonicalTimeoutConfig {
```

#### CanonicalPerformanceConfig (3 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/performance.rs:15:pub struct CanonicalPerformanceConfig {
crates/songbird-types/src/config/performance.rs:29:pub struct CanonicalPerformanceConfig {
crates/songbird-orchestrator/src/core/scalability/types.rs:111:pub struct CanonicalPerformanceConfig {
```

#### CanonicalEnvironmentConfig (3 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/environment.rs:15:pub struct CanonicalEnvironmentConfig {
crates/songbird-types/src/config/environment_corrupted.rs:12:pub struct CanonicalEnvironmentConfig {
crates/songbird-types/src/config/environment.rs:13:pub struct CanonicalEnvironmentConfig {
```

#### CanonicalCircuitBreakerConfig (3 definitions)
```
crates/songbird-types/src/config/api.rs:230:pub struct CanonicalCircuitBreakerConfig {
crates/songbird-types/src/config/adapters.rs:371:pub struct CanonicalCircuitBreakerConfig {
crates/songbird-types/src/adapters/canonical.rs:161:pub struct CanonicalCircuitBreakerConfig {
```

#### BulkheadConfig (3 definitions)
```
crates/songbird-orchestrator/src/core/robustness/config.rs:163:pub struct BulkheadConfig {
crates/songbird-config/src/canonical/resilience.rs:283:pub struct BulkheadConfig {
crates/songbird-config/src/unified/robustness.rs:122:pub struct BulkheadConfig  {/// Enable bulkhead pattern
```

#### BenchmarkConfig (3 definitions)
```
crates/songbird-types/src/config/gaming.rs:503:pub struct BenchmarkConfig {
crates/songbird-orchestrator/src/core/production_benchmarks/types.rs:10:pub struct BenchmarkConfig {
crates/songbird-config/src/canonical/performance.rs:284:pub struct BenchmarkConfig {
```

#### AuthenticationConfig (3 definitions)
```
crates/songbird-types/src/config/gaming.rs:448:pub struct AuthenticationConfig {
crates/songbird-types/src/config/security.rs:41:pub struct AuthenticationConfig {
crates/songbird-config/src/canonical/security.rs:209:pub struct AuthenticationConfig {
```

#### WebSocketConfig (2 definitions)
```
crates/songbird-types/src/config/network.rs:389:pub struct WebSocketConfig {
crates/songbird-types/src/config/communication.rs:125:pub struct WebSocketConfig {
```

#### ValidationConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/validation.rs:430:pub struct ValidationConfig {
crates/songbird-discovery/src/traits/validation.rs:387:pub struct ValidationConfig  {/// Whether validation is enabled
```

#### ValidationCacheConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/validation.rs:456:pub struct ValidationCacheConfig {
crates/songbird-discovery/src/traits/validation.rs:405:pub struct ValidationCacheConfig  {pub enabled: bool,
```

#### UniversalAdapterConfig (2 definitions)
```
crates/songbird-canonical/src/config/adapters.rs:12:pub struct UniversalAdapterConfig {
crates/songbird-primal-sdk/src/universal_adapter/types.rs:182:pub struct UniversalAdapterConfig  {/// Discovery interval
```

#### UnifiedObservabilityConfig (2 definitions)
```
crates/songbird-config/src/canonical/observability.rs:10:pub struct UnifiedObservabilityConfig {
crates/songbird-config/src/unified/observability.rs:35:pub struct UnifiedObservabilityConfig  {/// Dashboard configuration
```

#### TrustConfig (2 definitions)
```
crates/songbird-universal/src/unified_agnostic_discovery.rs:211:pub struct TrustConfig  {pub initial_trust: f64,
crates/songbird-discovery/src/discovery/config/mod.rs:51:pub struct TrustConfig {
```

#### TrackingConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:336:pub struct TrackingConfig {
crates/songbird-discovery/src/traits/resource_management.rs:244:pub struct TrackingConfig  {pub enabled: bool,
```

#### TlsConfig (2 definitions)
```
crates/songbird-types/src/config/communication.rs:108:pub struct TlsConfig {
crates/songbird-config/src/config/mod.rs:284:pub struct TlsConfig {
```

#### StorageConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/biome/byob_coordinator/integration.rs:28:pub struct StorageConfig {
crates/songbird-primal-sdk/src/capability_storage.rs:142:pub struct StorageConfig  {/// Discovery timeout
```

#### ServiceFailureConfig (2 definitions)
```
```

#### SecurityProviderIntegrationConfig (2 definitions)
```
crates/songbird-types/src/config/gaming.rs:275:pub struct SecurityProviderIntegrationConfig {
crates/songbird-types/src/config/security.rs:125:pub struct SecurityProviderIntegrationConfig {
```

#### ScalabilityConfig (2 definitions)
```
crates/songbird-types/src/config/performance.rs:489:pub struct ScalabilityConfig {
crates/songbird-orchestrator/src/core/scalability/types.rs:131:pub struct ScalabilityConfig {
```

#### ResourceConstraintConfig (2 definitions)
```
```

#### RegistryConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/mod.rs:179:pub struct RegistryConfig {
crates/songbird-registry/src/persistence/production_registry.rs:60:pub struct RegistryConfig  {/// Service TTL before considered stale
```

#### ProtocolDetectionConfig (2 definitions)
```
crates/songbird-types/src/config/gaming.rs:549:pub struct ProtocolDetectionConfig {
crates/songbird-config/src/gaming.rs:46:pub struct ProtocolDetectionConfig  {/// Enable automatic protocol detection
```

#### PrimalHealthConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/biome/modules/types.rs:242:pub struct PrimalHealthConfig {
crates/songbird-config/src/unified/primals.rs:127:pub struct PrimalHealthConfig {
```

#### PrimalDiscoveryEngine (2 definitions)
```
crates/songbird-primal-sdk/src/discovery/engine.rs:13:pub struct PrimalDiscoveryEngine  {_config: PrimalConfig,
crates/songbird-primal-sdk/src/discovery/discovery_engine.rs:15:pub struct PrimalDiscoveryEngine  {/// Configuration for primal discovery
```

#### PrimalConfig (2 definitions)
```
crates/songbird-config/src/config/hardcoded_elimination.rs:82:pub struct PrimalConfig {
crates/songbird-config/src/config/universal_primals.rs:68:pub struct PrimalConfiguration {
crates/songbird-config/src/config/universal_primals.rs:230:pub struct PrimalConfigurationTemplate {
crates/songbird-config/src/config/universal_primals_clean.rs:107:pub struct PrimalConfiguration  {/// Primal identifier
crates/songbird-config/src/canonical/primals.rs:329:pub struct PrimalConfiguration {
crates/songbird-primal-sdk/src/traits/types.rs:254:pub struct PrimalConfig  {/// Instance-specific configuration
```

#### PortConfig (2 definitions)
```
crates/songbird-canonical/src/config/environment.rs:45:pub struct PortConfig {
crates/songbird-config/src/config/network_endpoints.rs:117:pub struct PortConfig {
```

#### PerformanceDegradationConfig (2 definitions)
```
```

#### PerformanceAnalysisConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/api/ai_mesh/mesh.rs:167:pub struct PerformanceAnalysisConfig {
crates/songbird-config/src/unified/api.rs:161:pub struct PerformanceAnalysisConfig  {/// Enable performance monitoring
```

#### ObservabilityConfig (2 definitions)
```
crates/songbird-canonical/src/config/environment.rs:82:pub struct ObservabilityConfig {
crates/songbird-config/src/config/mod.rs:536:pub struct ObservabilityConfig {
```

#### NetworkPerformanceConfig (2 definitions)
```
crates/songbird-types/src/config/network.rs:104:pub struct NetworkPerformanceConfig {
crates/songbird-types/src/config/performance.rs:231:pub struct NetworkPerformanceConfig {
```

#### NetworkFaultConfig (2 definitions)
```
```

#### MonitoringConfiguration (2 definitions)
```
crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs:391:pub struct MonitoringConfiguration {
crates/songbird-config/src/unified/api.rs:281:pub struct MonitoringConfiguration  {/// Enable monitoring
```

#### MetricsConfig (2 definitions)
```
crates/songbird-config/src/config/mod.rs:548:pub struct MetricsConfig {
crates/songbird-config/src/canonical/performance.rs:233:pub struct MetricsConfig {
```

#### LogRotationConfig (2 definitions)
```
crates/songbird-config/src/canonical/observability.rs:81:pub struct LogRotationConfig {
crates/songbird-config/src/unified/observability.rs:95:pub struct LogRotationConfig {
```

#### LogConfig (2 definitions)
```
crates/songbird-config/src/config/environment.rs:51:pub struct LogConfig {
crates/songbird-config/src/canonical/environment.rs:167:pub struct LogConfig {
```

#### LimitsConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:389:pub struct LimitsConfig {
crates/songbird-discovery/src/traits/resource_management.rs:354:pub struct LimitsConfig  {pub limit_enforcement: LimitEnforcement,
```

#### IntegrationTestConfig (2 definitions)
```
```

#### HookSystemConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:371:pub struct HookSystemConfig {
crates/songbird-discovery/src/traits/hooks.rs:329:pub struct HookSystemConfig  {/// Whether the hook system is enabled
```

#### HookConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:168:pub struct HookConfig {
crates/songbird-discovery/src/traits/hooks.rs:173:pub struct HookConfig  {/// Hook-specific settings
```

#### HealthCheckConfiguration (2 definitions)
```
crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs:213:pub struct HealthCheckConfiguration {
crates/songbird-config/src/unified/api.rs:225:pub struct HealthCheckConfiguration  {/// Health check endpoint path
```

#### GamingPerformanceConfig (2 definitions)
```
crates/songbird-types/src/config/gaming.rs:190:pub struct GamingPerformanceConfig {
crates/songbird-config/src/gaming.rs:78:pub struct GamingPerformanceConfig  {/// Enable zero-copy packet forwarding
```

#### GamingConfig (2 definitions)
```
crates/songbird-config/src/gaming.rs:14:pub struct GamingConfig  {/// Gaming bridge configuration
crates/songbird-network-federation/src/network/mod.rs:218:pub struct GamingConfig {
```

#### GamingBridgeConfig (2 definitions)
```
crates/songbird-types/src/config/network.rs:62:pub struct GamingBridgeConfig {
crates/songbird-config/src/gaming.rs:28:pub struct GamingBridgeConfig  {/// Enable the gaming bridge
```

#### FlagMonitoringConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:452:pub struct FlagMonitoringConfig {
crates/songbird-discovery/src/traits/feature_flags.rs:416:pub struct FlagMonitoringConfig {
```

#### FeatureFlagConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:380:pub struct FeatureFlagConfig {
crates/songbird-discovery/src/traits/feature_flags.rs:370:pub struct FeatureFlagConfig {
```

#### ExperimentConfig (2 definitions)
```
```

#### ExecutionConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:230:pub struct ExecutionConfig {
crates/songbird-discovery/src/traits/hooks.rs:215:pub struct ExecutionConfig  {/// Whether to execute asynchronously
```

#### EvaluationConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:435:pub struct EvaluationConfig {
crates/songbird-discovery/src/traits/feature_flags.rs:406:pub struct EvaluationConfig {
```

#### DiscoveryTimingConfig (2 definitions)
```
crates/songbird-types/src/config/discovery_corrupted.rs:52:pub struct DiscoveryTimingConfig {
crates/songbird-types/src/config/discovery.rs:51:pub struct DiscoveryTimingConfig {
```

#### DiscoveryPerformanceConfig (2 definitions)
```
crates/songbird-types/src/config/discovery_corrupted.rs:275:pub struct DiscoveryPerformanceConfig {
crates/songbird-types/src/config/discovery.rs:276:pub struct DiscoveryPerformanceConfig {
```

#### DiscoveryMechanismsConfig (2 definitions)
```
crates/songbird-types/src/config/discovery_corrupted.rs:84:pub struct DiscoveryMechanismsConfig {
crates/songbird-types/src/config/discovery.rs:83:pub struct DiscoveryMechanismsConfig {
```

#### ConsensusConfig (2 definitions)
```
crates/songbird-types/src/config/federation.rs:470:pub struct ConsensusConfig {
crates/songbird-config/src/unified/federation.rs:211:pub struct ConsensusConfig {
```

#### ConfigValidator; (2 definitions)
```
crates/songbird-config/src/zero_touch/config.rs:295:pub struct ConfigValidator;
crates/songbird-config/src/config/validation_clean.rs:61:pub struct ConfigValidator;
```

#### ConfigValidationResult (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/validation.rs:141:pub struct ConfigValidationResult {
crates/songbird-discovery/src/traits/validation.rs:117:pub struct ConfigValidationResult  {/// Overall validation status
```

#### ConfigMetadata (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/config.rs:53:pub struct ConfigMetadata {
crates/songbird-discovery/src/traits/config.rs:56:pub struct ConfigMetadata {
```

#### CleanupConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:357:pub struct CleanupConfig {
crates/songbird-discovery/src/traits/resource_management.rs:337:pub struct CleanupConfig  {pub strategy: String,
```

#### CapabilityDiscoveryConfig (2 definitions)
```
crates/songbird-config/src/self_discovery.rs:155:pub struct CapabilityDiscoveryConfig  {/// Discovery methods to use
crates/songbird-config/src/canonical/discovery.rs:157:pub struct CapabilityDiscoveryConfig {
```

#### CanonicalSystemConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/system.rs:17:pub struct CanonicalSystemConfig {
crates/songbird-types/src/config/system.rs:11:pub struct CanonicalSystemConfig {
```

#### CanonicalSecurityConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/security.rs:15:pub struct CanonicalSecurityConfig {
crates/songbird-types/src/config/security.rs:14:pub struct CanonicalSecurityConfig {
```

#### CanonicalRetryConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/network.rs:141:pub struct CanonicalRetryConfig {
crates/songbird-types/src/adapters/canonical.rs:174:pub struct CanonicalRetryConfig {
```

#### CanonicalPrimalConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/primals.rs:15:pub struct CanonicalPrimalConfig {
crates/songbird-types/src/primal.rs:141:pub struct CanonicalPrimalConfig {
```

#### CanonicalObservabilityConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/observability.rs:17:pub struct CanonicalObservabilityConfig {
crates/songbird-config/src/unified/core.rs:74:pub struct CanonicalObservabilityConfig {
```

#### CanonicalMonitoringConfig (2 definitions)
```
crates/songbird-types/src/config/api.rs:256:pub struct CanonicalMonitoringConfig {
crates/songbird-types/src/adapters/canonical.rs:216:pub struct CanonicalMonitoringConfig {
```

#### CanonicalHealthMonitoringConfig (2 definitions)
```
crates/songbird-types/src/config/api.rs:135:pub struct CanonicalHealthMonitoringConfig {
crates/songbird-types/src/config/adapters.rs:483:pub struct CanonicalHealthMonitoringConfig {
```

#### CanonicalHealthConfig (2 definitions)
```
crates/songbird-types/src/config/health.rs:8:pub struct CanonicalHealthConfig {
crates/songbird-types/src/health.rs:123:pub struct CanonicalHealthConfig {
```

#### CanonicalHealthCheckConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/observability.rs:32:pub struct CanonicalHealthCheckConfig {
crates/songbird-types/src/adapters/canonical.rs:203:pub struct CanonicalHealthCheckConfig {
```

#### CanonicalGamingConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/gaming.rs:15:pub struct CanonicalGamingConfig {
crates/songbird-types/src/config/gaming.rs:38:pub struct CanonicalGamingConfig {
```

#### CanonicalFederationConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/federation.rs:15:pub struct CanonicalFederationConfig {
crates/songbird-types/src/config/federation.rs:14:pub struct CanonicalFederationConfig {
```

#### CanonicalConnectionPoolConfig (2 definitions)
```
crates/songbird-types/src/config/consolidated_canonical/network.rs:204:pub struct CanonicalConnectionPoolConfig {
crates/songbird-types/src/config/adapters.rs:317:pub struct CanonicalConnectionPoolConfig {
```

#### CachingConfig (2 definitions)
```
crates/songbird-types/src/config/performance.rs:320:pub struct CachingConfig {
crates/songbird-types/src/config/communication.rs:426:pub struct CachingConfig {
```

#### ByzantineFailureConfig (2 definitions)
```
```

#### BootstrapConfig (2 definitions)
```
crates/songbird-config/src/zero_touch_config.rs:226:pub struct BootstrapConfig {
crates/songbird-config/src/zero_touch/infant_config.rs:231:pub struct BootstrapConfig {
```

#### ApiConfig (2 definitions)
```
crates/songbird-orchestrator/src/core/api.rs:12:pub struct ApiConfig {
crates/songbird-config/src/unified/api.rs:10:pub struct ApiConfig  {/// Session configuration for real-time AI streaming
```

---

## Trait Duplicates

### Summary

      4 HealthMonitor
      3 ConfigProvider
      2 ZeroCostSecurity
      2 ZeroCostLoadBalancer
      2 ZeroCostDiscovery
      2 ZeroCostCommunication
      2 ValidationManager
      2 UniversalService
      2 UniversalAdapterTrait
      2 ServiceDiscovery
      2 SecurityProvider
      2 ResourceMonitor
      2 ResourceManager
      2 PluginRegistry
      2 Observability
      2 LoadBalancer
      2 LifecycleHook
      2 HookManager
      2 FeatureFlagProvider
      2 FeatureFlagManager
      2 EventHook
      2 DiscoveryChannel
      2 ConfigValidator
      2 ComposablePlugin
      2 CommunicationLayer
      2 CleanupStrategy

### Detailed Locations


#### HealthMonitor (4 definitions)
```
crates/songbird-orchestrator/src/core/traits/mod.rs:57:pub trait HealthMonitor: Send + Sync { /// Add health check
crates/songbird-orchestrator/src/core/traits/health.rs:120:pub trait HealthMonitor: Send + Sync {
crates/songbird-observability/src/health/mod.rs:12:pub trait HealthMonitor: Send + Sync {
crates/songbird-discovery/src/traits/health.rs:44:pub trait HealthMonitor: Send + Sync {
```

#### ConfigProvider (3 definitions)
```
crates/songbird-orchestrator/src/core/traits/config.rs:15:pub trait ConfigProvider<T>: Send + /// Sync
crates/songbird-discovery/src/traits/config.rs:27:pub trait ConfigProvider<T>: Send + Sync
crates/songbird-config/src/config/providers.rs:22:pub trait ConfigProvider<T>: Send + Sync {
```

#### ZeroCostSecurity (2 definitions)
```
crates/songbird-orchestrator/src/core/zero_cost_unified_example.rs:187:pub trait ZeroCostSecurity { async fn placeholder_function() -> SongbirdResult<()>   {
crates/songbird-orchestrator/src/core/zero_cost_pilot.rs:116:pub trait ZeroCostSecurity { fn is_authorized() {
```

#### ZeroCostLoadBalancer (2 definitions)
```
crates/songbird-orchestrator/src/core/zero_cost_unified_example.rs:175:pub trait ZeroCostLoadBalancer { async fn placeholder_function() -> SongbirdResult<()>   {
crates/songbird-orchestrator/src/core/zero_cost_request_router.rs:198:pub trait ZeroCostLoadBalancer { /// Select instance with zero virtual dispatch
```

#### ZeroCostDiscovery (2 definitions)
```
crates/songbird-orchestrator/src/core/zero_cost_unified_example.rs:160:pub trait ZeroCostDiscovery { async fn health_check() -> SongbirdResult<()>   {
crates/songbird-orchestrator/src/core/zero_cost_discovery.rs:16:pub trait ZeroCostDiscovery<
```

#### ZeroCostCommunication (2 definitions)
```
crates/songbird-orchestrator/src/core/zero_cost_unified_example.rs:181:pub trait ZeroCostCommunication { async fn placeholder_function() -> SongbirdResult<()>   {
crates/songbird-orchestrator/src/core/zero_cost_request_router.rs:212:pub trait ZeroCostCommunication { /// Send request with zero virtual dispatch
```

#### ValidationManager (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/validation.rs:385:pub trait ValidationManager: Send + Sync { /// Register a validator
crates/songbird-discovery/src/traits/validation.rs:340:pub trait ValidationManager: Send + Sync {
```

#### UniversalService (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/service.rs:13:pub trait UniversalService: Send + Sync + 'static  {type Error: std::error::Error + Send + Sync + 'static;
crates/songbird-discovery/src/traits/service.rs:14:pub trait UniversalService: Send + Sync + 'static {
crates/songbird-primal-sdk/src/universal_registry/traits.rs:9:pub trait UniversalServiceRegistry: Send + Sync {
```

#### UniversalAdapterTrait (2 definitions)
```
crates/songbird-universal/src/self_discovery.rs:99:pub trait UniversalAdapterTrait: Send + Sync + std::fmt::Debug { /// Discover primals by capability (no hardcoded names,
crates/songbird-universal/src/network_effects_decoupling.rs:281:pub trait UniversalAdapterTrait: Send + Sync { /// Route request to capability provider
```

#### ServiceDiscovery (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/discovery.rs:17:pub trait ServiceDiscovery: Send + Sync { /// Register a service with the discovery system
crates/songbird-discovery/src/traits/discovery.rs:19:pub trait ServiceDiscovery: Send + Sync {
```

#### SecurityProvider (2 definitions)
```
crates/songbird-types/src/traits/canonical.rs:234:pub trait SecurityProvider: Provider {
crates/songbird-universal/src/adapters/security.rs:278:pub trait SecurityProvider: Send + Sync {
```

#### ResourceMonitor (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:275:pub trait ResourceMonitor: Send + Sync { /// Start monitoring resources
crates/songbird-discovery/src/traits/resource_management.rs:203:pub trait ResourceMonitor: Send + Sync {
```

#### ResourceManager (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:17:pub trait ResourceManager: Send + Sync { /// Initialize resource tracking
crates/songbird-discovery/src/traits/resource_management.rs:15:pub trait ResourceManager: Send + Sync {
```

#### PluginRegistry (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/mod.rs:210:pub trait PluginRegistry: Send + Sync { /// Register a plugin dynamically
crates/songbird-registry/src/registry/traits.rs:15:pub trait PluginRegistry: Send + Sync {
```

#### Observability (2 definitions)
```
crates/songbird-types/src/traits/canonical.rs:294:pub trait ObservabilityProvider: Provider {
crates/songbird-orchestrator/src/core/traits/observability.rs:154:pub trait Observability: Send + Sync  {/// Start a new span
crates/songbird-discovery/src/traits/observability.rs:126:pub trait Observability: Send + Sync {
```

#### LoadBalancer (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/load_balancer.rs:55:pub trait LoadBalancer: Send + Sync { /// Select a service instance for the given request
crates/songbird-discovery/src/traits/load_balancer.rs:44:pub trait LoadBalancer: Send + Sync {
```

#### LifecycleHook (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:334:pub trait LifecycleHook: Send + Sync  {/// Before service registration
crates/songbird-discovery/src/traits/hooks.rs:282:pub trait LifecycleHook: Send + Sync {
```

#### HookManager (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:264:pub trait HookManager: Send + Sync { /// Register a new hook
crates/songbird-discovery/src/traits/hooks.rs:238:pub trait HookManager: Send + Sync {
```

#### FeatureFlagProvider (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:13:pub trait FeatureFlagProvider: Send + Sync { /// Initialize the feature flag provider
crates/songbird-discovery/src/traits/feature_flags.rs:20:pub trait FeatureFlagProvider: Send + Sync {
```

#### FeatureFlagManager (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/feature_flags.rs:330:pub trait FeatureFlagManager: Send + Sync  {/// Initialize the manager
crates/songbird-discovery/src/traits/feature_flags.rs:313:pub trait FeatureFlagManager: Send + Sync {
```

#### EventHook (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/hooks.rs:14:pub trait EventHook: Send + Sync { /// Hook name for identification
crates/songbird-discovery/src/traits/hooks.rs:15:pub trait EventHook: Send + Sync {
```

#### DiscoveryChannel (2 definitions)
```
crates/songbird-primal-sdk/src/discovery/universal_discovery/channels.rs:9:pub trait DiscoveryChannel: Send + Sync  {/// Get channel name
crates/songbird-primal-sdk/src/adaptive_discovery.rs:482:pub trait DiscoveryChannel: Send + Sync {
```

#### ConfigValidator (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/validation.rs:13:pub trait ConfigValidator: Send + Sync { /// Validate a configuration value
crates/songbird-discovery/src/traits/validation.rs:14:pub trait ConfigValidator: Send + Sync {
```

#### ComposablePlugin (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/mod.rs:69:pub trait ComposablePlugin: Send + Sync   {/// Unique plugin identifier (e.g., "security_provider-encryption", "compute_provider-compute")"
crates/songbird-registry/src/plugin/mod.rs:22:pub trait ComposablePlugin: Send + Sync {
```

#### CommunicationLayer (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/communication.rs:12:pub trait CommunicationLayer: Send + Sync { /// Send a message to a specific service
crates/songbird-discovery/src/traits/communication.rs:14:pub trait CommunicationLayer: Send + Sync {
```

#### CleanupStrategy (2 definitions)
```
crates/songbird-orchestrator/src/core/traits/resource_management.rs:231:pub trait CleanupStrategy: Send + Sync { /// Determine if a resource should be cleaned up
crates/songbird-discovery/src/traits/resource_management.rs:174:pub trait CleanupStrategy: Send + Sync {
```

---

## Error Type Duplicates

### Summary

      4 ErrorSeverity
      3 HookErrorHandling
      2 ApiError

### Detailed Locations


#### ErrorSeverity (4 definitions)
```
crates/songbird-types/src/types/severity.rs:13:pub enum ErrorSeverity {
crates/songbird-orchestrator/src/core/api/ai_first_response.rs:198:pub enum ErrorSeverity {
crates/songbird-orchestrator/src/core/traits/validation.rs:122:pub enum ErrorSeverity {
crates/songbird-discovery/src/traits/validation.rs:100:pub enum ErrorSeverity  {Critical)
```

#### HookErrorHandling (3 definitions)
```
crates/songbird-types/src/types/hooks.rs:12:pub enum HookErrorHandling {
crates/songbird-orchestrator/src/core/traits/hooks.rs:416:pub enum HookErrorHandling {
crates/songbird-discovery/src/traits/hooks.rs:359:pub enum HookErrorHandling  {/// Continue on errors
```

#### ApiError (2 definitions)
```
crates/songbird-orchestrator/src/server/compute_api.rs:271:pub enum ApiError {
crates/songbird-orchestrator/src/server/execution_api.rs:98:pub enum ApiError {
```

---

## Recommendations

### Config Consolidation Process
1. For each duplicate config name above:
   - Identify the canonical version (usually in `canonical/` directory)
   - Compare definitions to ensure they're actually duplicates
   - Update all imports to use canonical version
   - Remove duplicate definitions

### Trait Consolidation Process
1. For each duplicate trait:
   - Determine if it should be in `songbird-types/src/traits/canonical.rs`
   - Or if it's domain-specific and belongs in the domain crate
   - Consolidate or clarify naming to indicate purpose

### Error Consolidation Process
1. Most errors should use `SongbirdError` from `songbird-types`
2. Domain-specific errors should be clearly named and documented
3. Consider if error variants can be added to canonical error instead

---

**Next Steps**: Review this report and create consolidation plan for each duplicate.
