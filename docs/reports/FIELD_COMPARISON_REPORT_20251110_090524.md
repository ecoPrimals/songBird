# Field-Level Struct Comparison Report

Generated: 2025-11-10 09:05:24

## Executive Summary

- **Total Analyzed**: 118 structs
- **✅ True Duplicates**: 13 (safe to consolidate)
- **⚠️  Domain Variants**: 105 (need review)
- **Consolidation Rate**: 11% can be safely consolidated

---

## ✅ True Duplicates (Safe to Consolidate)

### BootstrapConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-config/src/zero_touch/infant_config.rs`
- `crates/songbird-config/src/zero_touch_config.rs`

**Fields:**
```rust
discovery_phases: Vec<DiscoveryPhase>
enable_infant_discovery: bool
fail_on_missing_required: bool
max_bootstrap_time: Duration
```

**Action:** Replace all with re-exports to canonical location

---

### ByzantineFailureConfig

**1 identical definitions**

**Locations:**
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
corrupt_data_rate: f64
delayed_response_rate: f64
malicious_behavior_types: Vec<String>
```

**Action:** Replace all with re-exports to canonical location

---

### CleanupConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-discovery/src/traits/resource_management.rs`
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
cleanup_interval: Duration
cleanup_on_shutdown: bool
force_cleanup_timeout: Duration
max_resource_age: Duration
strategy: String
```

**Action:** Replace all with re-exports to canonical location

---

### ConfigMetadata

**2 identical definitions**

**Locations:**
- `crates/songbird-discovery/src/traits/config.rs`
- `crates/songbird-orchestrator/src/core/traits/config.rs`

**Fields:**
```rust
checksum: String
last_modified: chrono::DateTime<chrono::Utc>
source: String
version: u64
```

**Action:** Replace all with re-exports to canonical location

---

### ConfigValidator

**1 identical definitions**

**Locations:**
- `crates/songbird-config/src/zero_touch/config.rs`

**Fields:**
```rust
field: "unknown".to_string()
message: format!("Configuration error"
suggestion: None
```

**Action:** Replace all with re-exports to canonical location

---

### DiscoveryMechanismsConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-types/src/config/discovery.rs`
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
enable_broadcast_discovery: bool
enable_config_discovery: bool
enable_consul_discovery: bool
enable_dns_discovery: bool
enable_env_discovery: bool
enable_kubernetes_discovery: bool
enable_network_scanning: bool
fallback_mechanisms: Vec<DiscoveryMechanism>
primary_mechanism: DiscoveryMechanism
```

**Action:** Replace all with re-exports to canonical location

---

### DiscoveryTimingConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-types/src/config/discovery.rs`
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
discovery_interval_secs: u64
discovery_timeout_secs: u64
health_check_interval_secs: u64
request_timeout_ms: u64
service_timeout_secs: u64
```

**Action:** Replace all with re-exports to canonical location

---

### EvaluationConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-discovery/src/traits/feature_flags.rs`
- `crates/songbird-orchestrator/src/core/traits/feature_flags.rs`

**Fields:**
```rust
default_timeout_ms: u64
enable_analytics: bool
enable_context_enrichment: bool
enable_debugging: bool
max_rule_depth: u32
```

**Action:** Replace all with re-exports to canonical location

---

### ExperimentConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-config/src/unified/testing.rs`
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
byzantine_failure: Option<ByzantineFailureConfig>
network_fault: Option<NetworkFaultConfig>
performance_degradation: Option<PerformanceDegradationConfig>
resource_constraint: Option<ResourceConstraintConfig>
service_failure: Option<ServiceFailureConfig>
```

**Action:** Replace all with re-exports to canonical location

---

### LogConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-config/src/canonical/environment.rs`
- `crates/songbird-config/src/config/environment.rs`

**Fields:**
```rust
file_rotation: bool
format: String
level: String
max_file_size_mb: u32
output: String
```

**Action:** Replace all with re-exports to canonical location

---

### LogRotationConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-config/src/canonical/observability.rs`
- `crates/songbird-config/src/unified/observability.rs`

**Fields:**
```rust
enabled: bool
max_files: u32
max_size_mb: u64
```

**Action:** Replace all with re-exports to canonical location

---

### ValidationCacheConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-discovery/src/traits/validation.rs`
- `crates/songbird-orchestrator/src/core/traits/validation.rs`

**Fields:**
```rust
enabled: bool
max_entries: u32
ttl_seconds: u64
```

**Action:** Replace all with re-exports to canonical location

---

### ValidationConfig

**2 identical definitions**

**Locations:**
- `crates/songbird-discovery/src/traits/validation.rs`
- `crates/songbird-orchestrator/src/core/traits/validation.rs`

**Fields:**
```rust
cache: ValidationCacheConfig
collect_warnings: bool
enabled: bool
error_handling: ErrorHandlingConfig
fail_fast: bool
max_errors: u32
timeout_ms: u64
```

**Action:** Replace all with re-exports to canonical location

---

## ⚠️  Domain-Specific Variants (Review Needed)

### ApiConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
connection: ConnectionConfig
mesh: MeshConfig
service_registration: ServiceRegistrationConfig
session: SessionConfig
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api.rs`

**Fields:**
```rust
enable_cors: bool
host: String
port: u16
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### AuthenticationConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/security.rs`

**Fields:**
```rust
enabled: bool
preferred_methods: Vec<AuthenticationMethod>
session_config: SessionConfig
token_config: TokenConfig
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
enabled: bool
method: String
token_lifetime: Duration
```

**Variant 3:**
- `crates/songbird-types/src/config/security.rs`

**Fields:**
```rust
enabled: bool
method: AuthenticationMethod
session_timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### BenchmarkConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/performance.rs`

**Fields:**
```rust
batch_test_size: usize
concurrent_requests: usize
duration_secs: u64
enabled: bool
output_format: String
warmup_duration_secs: u64
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/production_benchmarks/types.rs`

**Fields:**
```rust
batch_test_size: usize
cache_test_data_size: usize
concurrent_workers: usize
object_pool_iterations: usize
requests_per_test: usize
service_instance_count: usize
test_duration: Duration
warmup_duration: Duration
```

**Variant 3:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
enabled: bool
interval: Duration
iterations: usize
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### BulkheadConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
enabled: bool
isolation_strategy: IsolationStrategy
max_concurrent_operations: usize
operation_timeout_ms: u64
queue_size: usize
thread_pool_size: usize
```

**Variant 2:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
enabled: bool
isolation_strategy: IsolationStrategy
max_concurrent_operations: usize
operation_timeout: Duration
queue_size: usize
thread_pool_size: usize
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
enable_priority_queue: bool
max_concurrent_requests: u32
max_queue_size: u32
queue_timeout: Duration
resource_pool: ResourcePoolConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CacheConfig

**6 different implementations** across 7 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/performance.rs`

**Fields:**
```rust
enabled: bool
max_size: usize
ttl_secs: u64
```

**Variant 2:**
- `crates/songbird-discovery/src/discovery/backends/service_discovery.rs`

**Fields:**
```rust
default_ttl: std::time::Duration
enabled: bool
max_cache_size: usize
```

**Variant 3:**
- `crates/songbird-discovery/src/traits/feature_flags.rs`
- `crates/songbird-orchestrator/src/core/traits/feature_flags.rs`

**Fields:**
```rust
cache_evaluations: bool
cache_flags: bool
enabled: bool
max_entries: u32
ttl_seconds: u64
```

**Variant 4:**
- `crates/songbird-orchestrator/src/core/api/ai_optimized/cache.rs`

**Fields:**
```rust
cleanup_interval: Duration
default_ttl: Duration
enable_predictive_caching: bool
max_items: usize
max_memory_bytes: usize
```

**Variant 5:**
- `crates/songbird-orchestrator/src/core/caching/advanced_cache.rs`

**Fields:**
```rust
cleanup_interval: Duration
default_ttl: Option<Duration>
enable_compression: bool
enable_persistence: bool
eviction_policy: EvictionPolicy
max_entries: usize
max_size_bytes: usize
```

**Variant 6:**
- `crates/songbird-primal-sdk/src/storage/cache.rs`

**Fields:**
```rust
default_ttl: Duration
enable_compression: bool
eviction_strategy: CacheEvictionStrategy
max_entries: usize
max_memory_bytes: usize
write_behind: bool
write_through: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CachingConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
enabled: bool
eviction_policy: EvictionPolicy
max_size: usize
ttl: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
cache_size_mb: usize
compression_enabled: bool
enabled: bool
eviction_policy: CacheEvictionPolicy
layers: Vec<CacheLayerConfig>
statistics_enabled: bool
ttl: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalCircuitBreakerConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
failure_threshold: u32
reset_timeout: Duration
success_threshold: u32
timeout: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/adapters.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
recovery_timeout_seconds: u64
```

**Variant 3:**
- `crates/songbird-types/src/config/api.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
recovery_timeout: Duration
timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalConnectionPoolConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/adapters.rs`

**Fields:**
```rust
connection_timeout_seconds: u64
idle_timeout_seconds: u64
max_connections: usize
min_connections: usize
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/network.rs`

**Fields:**
```rust
connect_timeout: Duration
health_check_query: Option<String>
idle_timeout: Duration
max_lifetime: Duration
max_size: u32
min_size: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalDiscoveryConfig

**6 different implementations** across 7 definitions

**Variant 1:**
- `crates/songbird-discovery/src/discovery/core.rs`

**Fields:**
```rust
backend: String
consul_url: Option<String>
enable_container_discovery: bool
enable_environment_discovery: bool
enable_network_scan: bool
health_check_interval: u64
kubernetes_namespace: Option<String>
timeout_seconds: u64
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/basic_iot/mod.rs`

**Fields:**
```rust
common_ports: Vec<u16>
enable_mdns: bool
enable_upnp: bool
scan_timeout: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/traits/discovery.rs`

**Fields:**
```rust
backend: DiscoveryBackend
connection_timeout: std::time::Duration
health_check_interval: std::time::Duration
retry_attempts: u32
retry_delay: std::time::Duration
```

**Variant 4:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
interval: Duration
max_services_per_capability: usize
service_ttl: Duration
timeout: Duration
```

**Variant 5:**
- `crates/songbird-types/src/config/consolidated_canonical/discovery.rs`

**Fields:**
```rust
backend: String
enabled: bool
```

**Variant 6:**
- `crates/songbird-types/src/config/discovery.rs`
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
enabled: bool
health: HealthCheckConfig
mechanisms: DiscoveryMechanismsConfig
network: NetworkDiscoveryConfig
performance: DiscoveryPerformanceConfig
service: ServiceDiscoveryConfig
timing: DiscoveryTimingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalEnvironmentConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/environment.rs`

**Fields:**
```rust
deployment_mode: String
name: String
```

**Variant 2:**
- `crates/songbird-types/src/config/environment.rs`

**Fields:**
```rust
capability_endpoints: CapabilityEndpoints
deployment_mode: DeploymentMode
environment_overrides: HashMap<String, String>
legacy_compatibility: LegacyCompatibilityConfig
network_binding: NetworkBindingConfig
resource_limits: ResourceLimits
service_discovery: ServiceDiscoveryConfig
```

**Variant 3:**
- `crates/songbird-types/src/config/environment_corrupted.rs`

**Fields:**
```rust
capability_endpoints: CapabilityEndpoints
deployment_mode: DeploymentMode
environment_overrides: HashMap<String, String>)
legacy_compatibility: LegacyCompatibilityConfig
network_binding: NetworkBindingConfig
resource_limits: ResourceLimits
service_discovery: ServiceDiscoveryConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalFederationConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/federation.rs`

**Fields:**
```rust
cluster_name: String
enabled: bool
```

**Variant 2:**
- `crates/songbird-types/src/config/federation.rs`

**Fields:**
```rust
consensus: ConsensusConfig
local_node: CanonicalLocalNodeConfig
peers: PeerManagementConfig
resources: ResourceManagementConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalGamingConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/gaming.rs`

**Fields:**
```rust
enabled: bool
protocol_version: String
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
auto: GamingAutoConfig
core: GamingCoreConfig
network: GamingNetworkConfig
one_touch: OneTouchConfig
performance: GamingPerformanceConfig
security: GamingSecurityConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalHealthCheckConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
healthy_threshold: u32
interval: Duration
timeout: Duration
unhealthy_threshold: u32
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/observability.rs`

**Fields:**
```rust
detailed_reporting: bool
enabled: bool
endpoint: String
expected_status_codes: Vec<u16>
failure_threshold: u32
headers: HashMap<String, String>
interval: Duration
success_threshold: u32
timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalHealthConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/health.rs`

**Fields:**
```rust
check_interval: Duration
check_timeout: Duration
enabled: bool
```

**Variant 2:**
- `crates/songbird-types/src/health.rs`

**Fields:**
```rust
check_interval_seconds: u64
enabled: bool
endpoint: String
timeout_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalHealthMonitoringConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/adapters.rs`

**Fields:**
```rust
check_interval: Duration
check_timeout: Duration
failure_threshold: u32
success_threshold: u32
```

**Variant 2:**
- `crates/songbird-types/src/config/api.rs`

**Fields:**
```rust
check_interval: Duration
check_timeout: Duration
enable_detailed_metrics: bool
failure_threshold: u32
recovery_threshold: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalMonitoringConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
collection_interval: Duration
enabled: bool
history_size: usize
retention_period: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/api.rs`

**Fields:**
```rust
enable_tracing: bool
enabled: bool
log_level: String
metrics_interval: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalNetworkConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
allowed_networks: Vec<String>
bind_address: IpAddr
connection_limits: ConnectionLimits
connection_timeout: Duration
cors: CorsConfig
dashboard_port: u16
discovery_port: u16
discovery_ports: Vec<u16>
federation_bind_address: IpAddr
federation_endpoints: Vec<String>
federation_port: u16
gaming: GamingNetworkConfig
gaming_port_range: PortRange
health_port: u16
max_bandwidth_mbps: u64
max_connections: usize
metrics_bind_address: IpAddr
metrics_port: u16
network_metrics: Option<NetworkMeasurement>
orchestrator_port: u16
production_bind_address: IpAddr
request_timeout: Duration
require_tls: bool
reverse_proxy: Option<ReverseProxyConfig>
self_config: Option<SelfAwareConfig>
ssl_config: Option<SslConfig>
stun_servers: Vec<String>
timeouts: NetworkTimeouts
topology_discovery: Option<DiscoveryNetworkTopology>
turn_relay: Option<TURNRelay>
universal_discovery: Option<UniversalDiscoveryConfig>
upnp_device: Option<UPnPDevice>
websocket_port: u16
worker_threads: usize
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/network.rs`

**Fields:**
```rust
base_port: u16
bind: CanonicalBindConfig
bind_host: String
client: CanonicalClientConfig
connection_pool: CanonicalConnectionPoolConfig
proxy: Option<CanonicalProxyConfig>
rate_limiting: CanonicalRateLimitConfig
timeouts: CanonicalTimeoutConfig
tls: Option<CanonicalTlsConfig>
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
core: NetworkCoreConfig
gaming: GamingNetworkConfig
performance: NetworkPerformanceConfig
ports: NetworkPortConfig
security: NetworkSecurityConfig
```

**Variant 4:**
- `crates/songbird-types/src/unified.rs`

**Fields:**
```rust
bind_address: String
connection_timeout: Duration
discovery_port: u16
health_port: u16
max_connections: usize
orchestrator_port: u16
request_timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalObservabilityConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/core.rs`

**Fields:**
```rust
health_check_enabled: bool
metrics_enabled: bool
metrics_port: u16
tracing_enabled: bool
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/observability.rs`

**Fields:**
```rust
enabled: bool
health_checks: CanonicalHealthCheckConfig
metrics: CanonicalMetricsConfig
metrics_interval: u64
tracing: CanonicalTracingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalPerformanceConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-orchestrator/src/core/scalability/types.rs`

**Fields:**
```rust
cache_size_mb: u32
connection_pool_size: u32
max_concurrent_requests: u32
request_timeout_ms: u64
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/performance.rs`

**Fields:**
```rust
enabled: bool
thread_pool_size: usize
```

**Variant 3:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
caching: CachingConfig
cpu: CpuOptimizationConfig
enabled: bool
io: IoPerformanceConfig
memory: CanonicalMemoryConfig
monitoring: PerformanceMonitoringConfig
network: NetworkPerformanceConfig
rust_optimizations: RustOptimizationFeatures
scalability: ScalabilityConfig
threading: CanonicalThreadingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalPrimalConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/primals.rs`

**Fields:**
```rust
discovery_method: String
enabled: bool
```

**Variant 2:**
- `crates/songbird-types/src/primal.rs`

**Fields:**
```rust
config: HashMap<String, String>
enabled: bool
id: CanonicalPrimalId
security_level: Option<String>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalRetryConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
backoff_multiplier: f64
base_delay: Duration
jitter_factor: f64
max_attempts: u32
max_delay: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/consolidated_canonical/network.rs`

**Fields:**
```rust
backoff_multiplier: f64
base_delay: Duration
jitter_factor: f64
max_attempts: u32
max_delay: Duration
retryable_status_codes: Vec<u16>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalSecurityConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/security.rs`

**Fields:**
```rust
auth_method: String
enabled: bool
```

**Variant 2:**
- `crates/songbird-types/src/config/security.rs`

**Fields:**
```rust
authentication: AuthenticationConfig
authorization: AuthorizationConfig
enabled: bool
encryption: EncryptionConfig
security_provider_integration: SecurityProviderIntegrationConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalSystemConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/consolidated_canonical/system.rs`

**Fields:**
```rust
app_name: String
cache_dir: String
config_dir: String
data_dir: String
environment: String
instance_id: String
log_dir: String
logging: CanonicalLoggingConfig
resources: CanonicalResourceConfig
shutdown: CanonicalShutdownConfig
system_id: String
temp_dir: String
version: String
```

**Variant 2:**
- `crates/songbird-types/src/config/system.rs`

**Fields:**
```rust
environment: String
instance_id: String
system_id: String
version: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CanonicalTimeoutConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-types/src/adapters/canonical.rs`

**Fields:**
```rust
connection_timeout: Duration
discovery_timeout: Duration
health_check_timeout: Duration
request_timeout: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/adapters.rs`

**Fields:**
```rust
connection_timeout: Duration
default_request_timeout: Duration
discovery_timeout: Duration
health_check_timeout: Duration
```

**Variant 3:**
- `crates/songbird-types/src/config/consolidated_canonical/network.rs`

**Fields:**
```rust
connect: Duration
health_check: Duration
keep_alive: Duration
request: Duration
shutdown: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CapabilityDiscoveryConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/discovery.rs`

**Fields:**
```rust
cache_ttl_secs: u64
discovery_batch_size: usize
enabled: bool
max_retry_attempts: usize
```

**Variant 2:**
- `crates/songbird-config/src/self_discovery.rs`

**Fields:**
```rust
capability_cache_ttl_secs: u64
discovery_methods: Vec<String>
discovery_timeout_secs: u64
manual_services: Vec<ManualServiceRegistration>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CircuitBreakerConfig

**13 different implementations** across 14 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/adapters.rs`
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
success_threshold: u32
timeout_seconds: u64
```

**Variant 2:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
success_threshold: u32
timeout_secs: u64
```

**Variant 3:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
half_open_max_requests: u32
success_threshold: u32
timeout: Duration
```

**Variant 4:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
recovery_timeout: Duration
timeout: Duration
```

**Variant 5:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
half_open_max_calls: u32
min_throughput_threshold: u32
recovery_timeout: Duration
success_threshold: u32
timeout: Duration
```

**Variant 6:**
- `crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs`

**Fields:**
```rust
failure_threshold_percentage: f64
minimum_request_threshold: u32
request_volume_threshold: u32
sleep_window_seconds: u64
```

**Variant 7:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
failure_threshold: 5
failure_threshold: u32
service_name: String
success_threshold: 3;
timeout: Duration
timeout: Duration::from_secs(30)
```

**Variant 8:**
- `crates/songbird-primal-sdk/src/modern_api.rs`

**Fields:**
```rust
failure_threshold: u32
retry_delay_secs: u64
success_threshold: u32
timeout_duration_secs: u64
```

**Variant 9:**
- `crates/songbird-primal-sdk/src/modern_api/mod.rs`

**Fields:**
```rust
failure_threshold: u32
```

**Variant 10:**
- `crates/songbird-primal-sdk/src/universal_registry/config.rs`

**Fields:**
```rust
failure_threshold: u32
half_open_max_calls: u32
timeout_duration: Duration
```

**Variant 11:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: usize
half_open_timeout: Duration
reset_timeout_multiplier: f64
success_threshold: usize
timeout: Duration
```

**Variant 12:**
- `crates/songbird-universal/src/circuit_breaker.rs`

**Fields:**
```rust
failure_threshold: u32
success_threshold: u32
timeout: Duration
```

**Variant 13:**
- `crates/songbird-universal/src/types/config.rs`

**Fields:**
```rust
failure_threshold: u32
failure_window: Duration
recovery_timeout: Duration
success_threshold: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### CliConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-cli/src/cli/config.rs`

**Fields:**
```rust
color: bool
config_dir: PathBuf
data_dir: PathBuf
default_deployment_type: String
editor: Option<String>
log_dir: PathBuf
```

**Variant 2:**
- `crates/songbird-cli/src/cli/core/types.rs`

**Fields:**
```rust
config_path: Option<String>
output_format: OutputFormat
quiet: bool
verbose: bool
```

**Variant 3:**
- `crates/songbird-orchestrator/src/cli/config.rs`

**Fields:**
```rust
colored_output: bool
config_path: Option<String>
verbose: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ConfigProviderInfo

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/config/providers.rs`

**Fields:**
```rust
description: String
format: ConfigFormat
name: String
```

**Variant 2:**
- `crates/songbird-discovery/src/traits/config.rs`

**Fields:**
```rust
description: String
name: String
provider_type: String
supports_reload: bool
supports_watch: bool
version: String
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/traits/config.rs`

**Fields:**
```rust
description: String
name: String
provider_type: String
supports_reload: bool
supports_watch: bool ;
version: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ConfigValidationResult

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/validation.rs`

**Fields:**
```rust
cross_field_errors: Vec<ValidationError>
error_count: u32
field_results: HashMap<String, ValidationResult>
schema_errors: Vec<ValidationError>
summary: ValidationSummary
total_duration_ms: u64
valid: bool
warning_count: u32
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/validation.rs`

**Fields:**
```rust
cross_field_errors: Vec<ValidationError>
error_count: u32
field_results: HashMap<String, ValidationResult>)
schema_errors: Vec<ValidationError>
summary: ValidationSummary
total_duration_ms: u64
valid: bool
warning_count: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ConnectionConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
connection_timeout: Duration
enable_pooling: bool
max_connections_per_client: usize
pool_size: usize
pool_timeout: Duration
read_timeout: Duration
write_timeout: Duration
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api/real_time_ai_streaming/connection.rs`

**Fields:**
```rust
compression_enabled: bool
connection_timeout_seconds: u32
heartbeat_interval_seconds: u32
max_message_size_bytes: u32
quality_monitoring_enabled: bool
reconnection_policy: ReconnectionPolicy
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
connection_timeout: Duration
max_connections: usize
request_timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ConnectionPoolConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
connection_timeout_secs: u64
idle_timeout_secs: u64
max_lifetime_secs: u64
max_size: usize
min_idle: usize
```

**Variant 2:**
- `crates/songbird-config/src/config/universal_primals.rs`

**Fields:**
```rust
idle_timeout: Duration
max_connections: u32
max_lifetime: Duration
min_idle: u32
```

**Variant 3:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
connection_timeout_seconds: u64
idle_timeout_seconds: u64
max_connections: usize
min_connections: usize
```

**Variant 4:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
connection_timeout: Duration
health_check_interval: Duration
idle_timeout: Duration
max_connections: usize
max_lifetime: Duration
min_connections: usize
pool_timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ConsensusConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/federation.rs`

**Fields:**
```rust
algorithm: String
election_timeout_ms: u64
heartbeat_interval_ms: u64
```

**Variant 2:**
- `crates/songbird-types/src/config/federation.rs`

**Fields:**
```rust
algorithm: ConsensusAlgorithm
election_timeout: u64
heartbeat_interval: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### DashboardConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/observability.rs`

**Fields:**
```rust
enabled: bool
host: String
port: u16
realtime_updates: bool
update_interval_secs: u64
```

**Variant 2:**
- `crates/songbird-config/src/unified/observability.rs`

**Fields:**
```rust
enable_real_time: bool
enabled: bool
max_alerts: usize
port: u16
refresh_interval_ms: u64
```

**Variant 3:**
- `crates/songbird-observability/src/observability/advanced_dashboard.rs`

**Fields:**
```rust
alert_thresholds: UnifiedAlertThresholds
enable_analytics: bool
enable_predictions: bool
max_history_points: usize
update_interval: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### DiscoveryConfig

**12 different implementations** across 13 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/discovery.rs`

**Fields:**
```rust
auto_discovery: bool
capability_discovery: CapabilityDiscoveryConfig
common_ports: Vec<u16>
network_discovery: NetworkDiscoveryConfig
scan_timeout_secs: u64
service_discovery: ServiceDiscoveryConfig
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
health_check: HealthCheckConfig
interval_seconds: u64
mechanism: DiscoveryMechanism
registration: RegistrationConfig
```

**Variant 3:**
- `crates/songbird-config/src/zero_touch/infant_config.rs`
- `crates/songbird-config/src/zero_touch_config.rs`

**Fields:**
```rust
cache_ttl: Duration
enable_cache: bool
methods: Vec<DiscoveryMethod>
refresh_interval: Duration
timeout: Duration
```

**Variant 4:**
- `crates/songbird-discovery/src/traits/discovery.rs`

**Fields:**
```rust
backend: DiscoveryBackend
connection_timeout: std::time::Duration
health_check_interval: std::time::Duration
retry_attempts: u32
retry_delay: std::time::Duration
```

**Variant 5:**
- `crates/songbird-network-federation/src/network/mod.rs`

**Fields:**
```rust
enabled: bool
interval: Duration
methods: Vec<DiscoveryMethod>
timeout: Duration
```

**Variant 6:**
- `crates/songbird-primal-sdk/src/adaptive_discovery.rs`

**Fields:**
```rust
discovery_interval_secs: u64
enable_community_discovery: bool
enable_environment_discovery: bool
enable_filesystem_discovery: bool
enable_network_discovery: bool
enable_registry_discovery: bool
health_check_interval_secs: u64
max_discovery_timeout_secs: u64
```

**Variant 7:**
- `crates/songbird-primal-sdk/src/discovery/types.rs`

**Fields:**
```rust
discovery_timeout_secs: u64
enable_broadcast: bool
enable_ecosystem_discovery: bool
enable_federation: bool
enable_network_scan: bool
enable_service_registry: bool
max_concurrent_operations: usize
network_scan_port_ranges: Vec<(u16, u16)>)
```

**Variant 8:**
- `crates/songbird-primal-sdk/src/discovery/universal_discovery/types.rs`

**Fields:**
```rust
consul_endpoints: Vec<String>
discovery_interval: Duration
discovery_ports: Vec<u16>
discovery_retry_attempts: u32
discovery_retry_delay: Duration
dns_discovery_domains: Vec<String>
enable_auto_discovery: bool
enable_consul_discovery: bool
enable_dns_discovery: bool
enable_kubernetes_discovery: bool
enable_multicast_discovery: bool
enable_network_scanning: bool
health_check_interval: Duration
kubernetes_namespace: Option<String>
max_concurrent_discoveries: usize
multicast_addresses: Vec<String>
network_scan_ranges: Vec<String>
service_timeout: Duration
```

**Variant 9:**
- `crates/songbird-universal/src/agnostic_service_discovery.rs`

**Fields:**
```rust
cache_expiry_seconds: u64;
discovery_timeout_ms: u64
enable_caching: bool
enable_network_scanning: bool
probe_ports: Vec<u16>
scan_ranges: Vec<String>
```

**Variant 10:**
- `crates/songbird-universal/src/capabilities/types.rs`

**Fields:**
```rust
auto_discovery: bool
discovery_timeout: std::time::Duration
enable_network_discovery: bool
max_concurrent_discoveries: usize
refresh_interval: std::time::Duration
```

**Variant 11:**
- `crates/songbird-universal/src/discovery.rs`

**Fields:**
```rust
mechanisms: DiscoveryMechanisms
timeout: Duration
```

**Variant 12:**
- `crates/songbird-universal/src/infant_discovery.rs`

**Fields:**
```rust
aggressive_discovery: bool ;
discovery_timeout: Duration
max_concurrent_discoveries: usize
network_ranges: Vec<String>
probe_ports: Vec<u16>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### DiscoveryPerformanceConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/discovery.rs`

**Fields:**
```rust
background_discovery: bool
batch_size: usize
cache_ttl_secs: u64
enable_caching: bool
max_cache_size: usize
max_concurrent_discoveries: usize
```

**Variant 2:**
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
max_concurrent_discoveries: usize
max_peers: usize
max_primals_per_capability: usize
peer_timeout: Duration
scan_timeout: Duration
topology_update_interval: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### EncryptionConfig

**4 different implementations** across 5 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/security.rs`

**Fields:**
```rust
enabled: bool
key_management: KeyManagementConfig
preferred_algorithms: Vec<EncryptionAlgorithm>
transport: TransportEncryptionConfig
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
algorithm: EncryptionAlgorithm
at_rest: bool
in_transit: bool
key_rotation_days: u32
```

**Variant 3:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
algorithm: String
enabled: bool
key_derivation: String
key_size: u32
```

**Variant 4:**
- `crates/songbird-types/src/config/gaming.rs`
- `crates/songbird-types/src/config/security.rs`

**Fields:**
```rust
algorithm: String
enabled: bool
key_size: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### EnvironmentConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/environment.rs`

**Fields:**
```rust
environment: Environment
logging: LoggingConfig
network: NetworkConfig
observability: ObservabilityConfig
ports: PortConfig
security: EnvironmentSecurityConfig
```

**Variant 2:**
- `crates/songbird-config/src/canonical/environment.rs`

**Fields:**
```rust
environment: Environment
log_config: LogConfig
performance_config: PerformanceParameters
resource_limits: ResourceLimits
service_endpoints: ServiceEndpoints
```

**Variant 3:**
- `crates/songbird-config/src/config/environment.rs`

**Fields:**
```rust
bind_address: String
bind_port: u16
connection_timeout_secs: u64
dashboard_port: u16
discovery_ports: Vec<u16>
discovery_timeout_secs: u64
enable_encryption: bool
environment: String
gaming_port_range: (u16, u16)
health_check_interval_secs: u64
log_config: LogConfig
log_level: String
max_connections: usize
metrics_interval_secs: u64
performance_config: PerformanceParameters
require_tls: bool
resource_limits: ResourceLimits
service_endpoints: ServiceEndpoints
session_timeout_secs: u64
```

**Variant 4:**
- `crates/songbird-config/src/unified/core.rs`

**Fields:**
```rust
config_path: Option<String>
debug: bool
environment: String
log_level: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ErrorHandlingConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/validation.rs`

**Fields:**
```rust
aggregate_errors: bool
logging_options: LoggingOptions
throw_on_critical: bool
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/structural_improvements/config.rs`

**Fields:**
```rust
base_retry_delay: Duration
circuit_breaker_threshold: u32
enable_auto_recovery: bool
escalation_timeout: Duration
recovery_timeout: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/traits/validation.rs`

**Fields:**
```rust
aggregate_errors: bool;
log_errors: bool
log_warnings: bool
throw_on_critical: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ExecutionConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/hooks.rs`

**Fields:**
```rust
async_execution: bool
log_execution: bool
measure_performance: bool
timeout_ms: u64
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/hooks.rs`

**Fields:**
```rust
async_execution: bool
log_execution: bool
measure_performance: bool,;
timeout_ms: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### FeatureFlagConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/feature_flags.rs`

**Fields:**
```rust
cache: CacheConfig
default_user: ProviderConfig
evaluation: EvaluationConfig
monitoring: FlagMonitoringConfig
providers: HashMap<String, ProviderConfig>
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/feature_flags.rs`

**Fields:**
```rust
cache: CacheConfig
default_user: ProviderConfig
evaluation: EvaluationConfig
monitoring: FlagMonitoringConfig
providers: HashMap<String, ProviderConfig>)
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### FederationConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
auto_discovery_enabled: bool
broadcast_ports: Vec<u16>
cluster_endpoints: Vec<String>
default_cluster_id: String
discovery_ports: Vec<u16>
heartbeat_endpoint: String
```

**Variant 2:**
- `crates/songbird-discovery/src/discovery/enhanced_discovery.rs`

**Fields:**
```rust
coordination: NetworkCoordinationConfig
discovery_methods: Vec<FederationDiscoveryMethod>
enabled: bool
node_id: String
sovereignty: SovereigntyConfig
```

**Variant 3:**
- `crates/songbird-network-federation/src/federation.rs`

**Fields:**
```rust
bootstrap_address: Option<String>
enabled: bool
heartbeat_interval_secs: u64
node_timeout_secs: i64
self_registration: Option<NodeRegistration>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### FlagMonitoringConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/feature_flags.rs`

**Fields:**
```rust
enabled: bool
metrics_interval: u64
monitoring_options: MonitoringOptions
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/feature_flags.rs`

**Fields:**
```rust
alert_on_errors: bool
enabled: bool
export_evaluations: bool;
metrics_interval: u64
track_performance: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### GamingBridgeConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/gaming.rs`

**Fields:**
```rust
bind_address: String
buffer_size: usize
enable_packet_logging: bool
enabled: bool
max_sessions: usize
port_range: (u16, u16)
session_timeout_seconds: u64
```

**Variant 2:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
enabled: bool
interface: String
ip_address: Option<String>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### GamingConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/gaming.rs`

**Fields:**
```rust
bridge: GamingBridgeConfig
performance: GamingPerformanceConfig
protocol_detection: ProtocolDetectionConfig
security: GamingSecurityConfig
session_management: SessionManagementConfig
```

**Variant 2:**
- `crates/songbird-network-federation/src/network/mod.rs`

**Fields:**
```rust
enabled: bool
max_sessions: u32
port_range: (u16, u16)
protocols: Vec<GameProtocolType>
session_timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### GamingNetworkConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
aoe2_port: u16
enable_lan_discovery: bool
ipx_port: u16
max_players_per_game: usize
starcraft_port: u16
udp_port: u16
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
optimization: NetworkOptimizationConfig
ports: GamingPortConfig
protocols: ProtocolConfig
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
bridge: GamingBridgeConfig
protocols: GamingProtocolConfig
virtual_network: VirtualNetworkConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### GamingPerformanceConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/gaming.rs`

**Fields:**
```rust
batch_size: usize
coalescing_timeout_us: u64
hardware_acceleration: bool
packet_coalescing: bool
worker_threads: usize
zero_copy_forwarding: bool
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
optimization: GamingOptimizationConfig
settings: GamingPerformanceSettings
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### GamingSecurityConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/gaming.rs`

**Fields:**
```rust
banned_ips: Vec<String>
ddos_protection: bool
enable_encryption: bool
max_packet_rate: u64
packet_filtering: bool
rate_limit_window_seconds: u64
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
auth: GamingAuthConfig
settings: GamingSecuritySettings
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
allowed_interfaces: Vec<String>
enable_encryption: bool
max_discovery_requests_per_minute: u32
max_players_per_session: u8
session_timeout_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### HealthCheckConfig

**15 different implementations** across 18 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/primals.rs`
- `crates/songbird-config/src/config/universal_primals_clean.rs`

**Fields:**
```rust
enabled: bool
endpoint_path: String
expected_status_codes: Vec<u16>
failure_threshold: u32
interval: Duration
timeout: Duration
```

**Variant 2:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
interval_secs: u64
path: String
recovery_threshold: u32
timeout_secs: u64
```

**Variant 3:**
- `crates/songbird-config/src/canonical/service.rs`

**Fields:**
```rust
endpoint: String
interval: u64
timeout: u64
```

**Variant 4:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
enabled: bool
endpoint: String
interval_seconds: u64
retries: u32
timeout_seconds: u64
```

**Variant 5:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
enabled: bool
failure_threshold: u32
interval: Duration
path: String
recovery_threshold: u32
timeout: Duration
```

**Variant 6:**
- `crates/songbird-discovery/src/traits/health.rs`
- `crates/songbird-orchestrator/src/core/traits/health.rs`

**Fields:**
```rust
enabled: bool
endpoint: Option<String>
interval: Duration
retries: u32
timeout: Duration
```

**Variant 7:**
- `crates/songbird-observability/src/health/production_health.rs`

**Fields:**
```rust
check_interval: Duration
degraded_threshold: u32
health_endpoints: Vec<String>
max_concurrent_checks: usize
request_timeout: Duration
unhealthy_threshold: u32
```

**Variant 8:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
check_interval: Duration
check_timeout: Duration
deep_check_config: DeepHealthCheckConfig
enable_deep_checks: bool
failure_threshold: u32
success_threshold: u32
```

**Variant 9:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
enabled: bool
endpoint: String
interval_seconds: u64
retry_count: u32
timeout_seconds: u64
```

**Variant 10:**
- `crates/songbird-primal-sdk/src/modern_api.rs`

**Fields:**
```rust
endpoint: String
expected_codes: Vec<u16>
healthy_threshold: u32
interval_secs: u64
timeout_secs: u64
unhealthy_threshold: u32
```

**Variant 11:**
- `crates/songbird-primal-sdk/src/universal_registry/config.rs`

**Fields:**
```rust
failure_threshold: u32
interval_seconds: u64
path: String
success_threshold: u32
timeout_seconds: u64
```

**Variant 12:**
- `crates/songbird-registry/src/types/health.rs`

**Fields:**
```rust
check_type: HealthCheckType
failure_threshold: u32
interval: Duration
success_threshold: u32
timeout: Duration
```

**Variant 13:**
- `crates/songbird-types/src/config/discovery.rs`
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
enabled: bool
endpoint: String
retries: u32
timeout_seconds: u64
```

**Variant 14:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
enabled: bool
healthy_threshold: u32
interval: Duration
timeout: Duration
unhealthy_threshold: u32
```

**Variant 15:**
- `crates/songbird-universal/src/types/config.rs`

**Fields:**
```rust
healthy_threshold: u32
interval: Duration
timeout: Duration
unhealthy_threshold: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### HealthCheckConfiguration

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
enabled: bool
endpoint: String
failure_threshold: u32
interval: Duration
timeout: Duration
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs`

**Fields:**
```rust
custom_parameters: HashMap<String, serde_json::Value> );
failure_threshold: u32
interval_seconds: u64
timeout_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### HealthMonitoringConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
check_interval: Duration
check_timeout: Duration
enable_detailed_metrics: bool
failure_threshold: u32
recovery_threshold: u32
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api/ai_mesh/mesh.rs`

**Fields:**
```rust
anomaly_threshold: f64
check_interval: std::time::Duration
prediction_window: std::time::Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/biome/modules/lifecycle.rs`

**Fields:**
```rust
default_interval: Duration
default_timeout: Duration
max_failures: u32
restart_on_failure: bool
```

**Variant 4:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
check_interval_seconds: u64
check_timeout_seconds: u64
enabled: bool
failure_threshold: u32
recovery_threshold: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### HookConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/hooks.rs`

**Fields:**
```rust
event_filter: EventFilter
execution: ExecutionConfig
retry: RetryConfig
settings: HashMap<String, serde_json::Value>
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/hooks.rs`

**Fields:**
```rust
event_filter: EventFilter
execution: ExecutionConfig
retry: RetryConfig
settings: HashMap<String, serde_json: :Value>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### HookSystemConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/hooks.rs`

**Fields:**
```rust
default_timeout_ms: u64
enabled: bool
error_handling: HookErrorHandling
execution_strategy: HookExecutionStrategy
log_executions: bool
max_hooks: u32
measure_performance: bool
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/hooks.rs`

**Fields:**
```rust
default_timeout_ms: u64
enabled: bool
error_handling: HookErrorHandling ;
execution_strategy: HookExecutionStrategy
log_executions: bool
max_hooks: u32
measure_performance: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### IntegrationTestConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-test-utils/src/config/mod.rs`

**Fields:**
```rust
network: TestNetworkConfig
security: TestSecurityConfig
services: Vec<String>
test_data: TestDataConfig ;
```

**Variant 2:**
- `crates/songbird-test-utils/src/integration.rs`

**Fields:**
```rust
max_services: usize
startup_delay: Duration
timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### LimitsConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/resource_management.rs`

**Fields:**
```rust
action_on_violation: ViolationAction
limit_enforcement: LimitEnforcement
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
action_on_violation: ViolationAction
enforce_connection_limits: bool
enforce_cpu_limits: bool
enforce_file_handle_limits: bool
enforce_memory_limits: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### LoadBalancerConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
algorithm: LoadBalancingAlgorithm
connection_timeout_ms: u64
fail_fast: bool
max_connections_per_backend: usize
session_timeout_secs: u64
sticky_sessions: bool
```

**Variant 2:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
algorithm: LoadBalancingAlgorithm
connection_timeout: Duration
fail_fast: bool
health_check: HealthCheckConfig
max_connections_per_backend: usize
session_timeout: Duration
sticky_sessions: bool
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/load_balancer/types.rs`

**Fields:**
```rust
health_check_interval_secs: u64
max_retries: u32
strategy: LoadBalancerStrategy
timeout_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### LoadBalancingConfig

**5 different implementations** across 5 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/orchestration.rs`

**Fields:**
```rust
health_check_interval_seconds: u64
max_retries: u32
request_timeout_ms: u64
strategy: LoadBalancingStrategy
```

**Variant 2:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
backends: Vec<String>
enabled: bool
health_check_interval_secs: u64
strategy: String
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/mod.rs`

**Fields:**
```rust
health_check_interval: u64
max_retries: u32
strategy: LoadBalancingStrategy
```

**Variant 4:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
algorithm: LoadBalancingAlgorithm
enabled: bool
health_check: HealthCheckConfig
session_affinity: bool
```

**Variant 5:**
- `crates/songbird-universal/src/types/config.rs`

**Fields:**
```rust
connection_timeout_ms: u64
health_check_enabled: bool
max_retries: u32
strategy: LoadBalancingStrategy
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### LoggingConfig

**5 different implementations** across 5 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/environment.rs`

**Fields:**
```rust
file_rotation: bool
format: String
level: String
max_file_size_mb: u32
max_files: u32
output: String
structured: bool
```

**Variant 2:**
- `crates/songbird-cli/src/cli/commands/firewall.rs`

**Fields:**
```rust
enabled: bool
```

**Variant 3:**
- `crates/songbird-config/src/canonical/observability.rs`

**Fields:**
```rust
enabled: bool
format: String
level: String
rotation: LogRotationConfig
```

**Variant 4:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
format: LogFormat
level: LogLevel
output: LogOutput
rotation: LogRotation
```

**Variant 5:**
- `crates/songbird-config/src/unified/observability.rs`

**Fields:**
```rust
format: String
level: String
output: String
rotation: LogRotationConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### MetricsConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/performance.rs`

**Fields:**
```rust
collection_interval_secs: u64
enabled: bool
export_prometheus: bool
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
enabled: bool
endpoint: String
exporters: Vec<MetricsExporter>
interval_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### MonitoringConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-discovery/src/discovery/config/mod.rs`

**Fields:**
```rust
detailed_cpu_monitoring: bool
gpu_monitoring_enabled: bool
network_stats_window_secs: u64
process_scan_enabled: bool
resource_update_interval_secs: u64
storage_stats_window_secs: u64
```

**Variant 2:**
- `crates/songbird-discovery/src/traits/resource_management.rs`

**Fields:**
```rust
alert_thresholds: HashMap<String, f64>
enable_leak_detection: bool
leak_detection_interval: Duration
monitoring_interval: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
alert_thresholds: HashMap<String, f64>)
enable_leak_detection: bool
leak_detection_interval: Duration
monitoring_interval: Duration
```

**Variant 4:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
metrics_enabled: bool
metrics_endpoint: String
metrics_port: u16
tracing: TracingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### MonitoringConfiguration

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
enable_tracing: bool
enabled: bool
log_level: String
metrics_interval: Duration
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api/universal_service_registration/types.rs`

**Fields:**
```rust
alert_thresholds: AlertThresholds
health_check_frequency_seconds: u64
metrics_interval_seconds: u64
performance_baseline: PerformanceBaseline
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### NetworkConfig

**6 different implementations** across 7 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/environment.rs`

**Fields:**
```rust
bind_address: String
connection_timeout: u64
enable_tls: bool
max_connections: usize
read_timeout: u64
tls_cert_path: Option<String>
tls_key_path: Option<String>
write_timeout: u64
```

**Variant 2:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
bind_address: IpAddr
dashboard_endpoint: Arc<str>
federation_endpoint: Arc<str>
gaming_endpoint: Arc<str>
gaming_port_range: PortRange
orchestrator_endpoint: Arc<str>
port_ranges: HashMap<String, (u16, u16)>
production_bind_address: IpAddr
stun_servers: Vec<String>
```

**Variant 3:**
- `crates/songbird-config/src/zero_touch/environment.rs`

**Fields:**
```rust
default_gateway: String
dns_servers: Vec<String>
has_internet: bool
interfaces: Vec<String>
public_ip: Option<String>
```

**Variant 4:**
- `crates/songbird-config/src/zero_touch/infant_config.rs`
- `crates/songbird-config/src/zero_touch_config.rs`

**Fields:**
```rust
bind_address: IpAddr
connection_limits: ConnectionLimits
health_port: u16
metrics_port: u16
service_port: u16
timeouts: NetworkTimeouts
```

**Variant 5:**
- `crates/songbird-discovery/src/discovery/config/mod.rs`

**Fields:**
```rust
announcement_interval_secs: u64
bind_address: String
default_bandwidth_mbps: f64
federation_port: u16
max_packet_size: usize
multicast_address: String
ping_timeout_secs: u64
response_timeout_secs: u64
service_port: u16
```

**Variant 6:**
- `crates/songbird-network-federation/src/network/mod.rs`

**Fields:**
```rust
discovery: DiscoveryConfig
gaming: GamingConfig
interface: InterfaceConfig
performance: PerformanceConfig
proxy: ProxyConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### NetworkDiscoveryConfig

**3 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/discovery.rs`

**Fields:**
```rust
discovery_protocols: Vec<String>
enabled: bool
scan_local_network: bool
scan_ports: Vec<u16>
```

**Variant 2:**
- `crates/songbird-types/src/config/discovery.rs`
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
bind_address: String
discovery_ports: Vec<u16>
dns_search_domains: Vec<String>
enable_stun: bool
enable_turn: bool
enable_upnp: bool
federation_port: u16
gaming_optimized: bool
multicast_address: String
network_scan_ranges: Vec<String>
service_port: u16
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
enabled: bool
interval: Duration
timeout: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### NetworkFaultConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/testing.rs`

**Fields:**
```rust
latency_ms: Option<u64>
packet_loss_percent: Option<f64>
```

**Variant 2:**
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
bandwidth_limit_bps: Option<u64>
latency_ms: Option<u64>
packet_loss_percent: Option<f64>
partition_enabled: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### NetworkPerformanceConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
buffer_sizes: BufferSizeConfig
connection_timeout: u32
keep_alive: bool
read_timeout: u32
write_timeout: u32
```

**Variant 2:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
buffer_config: BufferConfig
connection_timeout_ms: u64
keepalive_enabled: bool
keepalive_interval_secs: u64
max_connections: u32
optimization_level: NetworkOptimizationLevel
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ObservabilityConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/environment.rs`

**Fields:**
```rust
custom_tags: HashMap<String, String>
enable_health_checks: bool
enable_metrics: bool
enable_tracing: bool
health_check_interval: u64
metrics_interval: u64
trace_sampling_rate: f64
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
logging: LoggingConfig
metrics: MetricsConfig
tracing: TracingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### OrchestratorConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-cli/src/cli/commands/quick.rs`

**Fields:**
```rust
capabilities: Vec<String>
discovery_endpoints: Vec<String>
node_name: String
ports: HashMap<String, u16>
resource_limits: ResourceLimits
security: SecurityConfig
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/biome/modules/types.rs`

**Fields:**
```rust
default_port: Option<u16>
endpoints: HashMap<String, String>)
id: String
name: String
timeout: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/orchestrator.rs`

**Fields:**
```rust
max_services: u32
name: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PerformanceAnalysisConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
analysis_window: Duration
cpu_threshold: f64
enabled: bool
latency_threshold: Duration
memory_threshold: f64
metrics_interval: Duration
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/api/ai_mesh/mesh.rs`

**Fields:**
```rust
analysis_window: std::time::Duration
outlier_threshold: f64
trend_sensitivity: f64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PerformanceConfig

**7 different implementations** across 7 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/performance.rs`

**Fields:**
```rust
enable_zero_cost: bool
latency: LatencyConfig
memory: MemoryConfig
throughput: ThroughputConfig
```

**Variant 2:**
- `crates/songbird-config/src/canonical/performance.rs`

**Fields:**
```rust
auto_tuning_sensitivity: f64
cache: CacheConfig
cache_size_mb: usize
connection_pool_size: usize
enable_adaptive_caching: bool
enable_async_batching: bool
enable_fast_load_balancing: bool
enable_memory_optimization: bool
enable_zero_copy: bool
max_concurrent_requests: usize
metrics: MetricsConfig
monitoring_interval_secs: u64
object_pool_sizes: ObjectPoolSizes
request_buffer_size: usize
thread_pool_size: usize
```

**Variant 3:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
cache_ttl: Duration
connection_pool_size: usize
large_buffer_size: usize
max_packet_size: usize
small_buffer_size: usize
```

**Variant 4:**
- `crates/songbird-config/src/lib.rs`

**Fields:**
```rust
batch_size: Option<usize>
buffer_pool_size: Option<usize>
connection_pool_size: Option<usize>
custom_params: Option<HashMap<String, serde_json::Value>>
enable_zero_copy: Option<bool>
max_memory_mb: Option<u64>
request_timeout_ms: Option<u64>
worker_threads: Option<usize>
```

**Variant 5:**
- `crates/songbird-network-federation/src/network/mod.rs`

**Fields:**
```rust
buffer_size: usize
keepalive: Option<Duration>
tcp_nodelay: bool
worker_threads: Option<usize>
```

**Variant 6:**
- `crates/songbird-orchestrator/src/core/mod.rs`

**Fields:**
```rust
alert_thresholds: HashMap<String, f64>
enable_benchmarking: bool
metrics_interval: u64
```

**Variant 7:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
caching: CachingConfig
connection_pooling: ConnectionPoolingConfig
enabled: bool
request_batching: RequestBatchingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PerformanceDegradationConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/testing.rs`

**Fields:**
```rust
response_time_multiplier: f64
throughput_reduction: f64
```

**Variant 2:**
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
resource_constraint: Option<ResourceConstraintConfig>
slowdown_factor: f64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PortConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/environment.rs`

**Fields:**
```rust
discovery_port: u16
dynamic_port_range: (u16, u16)
federation_port: u16
health_port: u16
```

**Variant 2:**
- `crates/songbird-config/src/config/network_endpoints.rs`

**Fields:**
```rust
dashboard_port: u16
discovery_port: u16
federation_port: u16
metrics_port: u16
orchestrator_port: u16
websocket_port: u16
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PrimalConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
base_port: u16
beardog_endpoint: Arc<str>
discovery_endpoints: Vec<String>
nestgate_endpoint: Arc<str>
port_range: (u16, u16)
squirrel_endpoint: Arc<str>
toadstool_endpoint: Arc<str>
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/traits/types.rs`

**Fields:**
```rust
custom: HashMap<String, serde_json::Value>)
instance: HashMap<String, serde_json::Value>)
performance: HashMap<String, serde_json::Value>)
security: HashMap<String, serde_json::Value>)
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PrimalConfiguration

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/primals.rs`

**Fields:**
```rust
capabilities: Vec<PrimalCapability>
connection_settings: ConnectionSettings
display_name: String
enabled: bool
endpoint: PrimalEndpoint
health_check: HealthCheckConfig
primal_type: String
```

**Variant 2:**
- `crates/songbird-config/src/config/universal_primals.rs`

**Fields:**
```rust
authentication: PrimalAuthentication
capabilities: Vec<PrimalCapability>
connection_settings: ConnectionSettings
discovery_metadata: DiscoveryMetadata
display_name: String
enabled: bool
endpoint: PrimalEndpoint
health_check: HealthCheckConfig
last_seen: Option<chrono::DateTime<chrono::Utc>>
primal_type: String
specific_config: HashMap<String, serde_json::Value>
```

**Variant 3:**
- `crates/songbird-config/src/config/universal_primals_clean.rs`

**Fields:**
```rust
auth_method: AuthenticationMethod
capabilities: Vec<PrimalCapability>
display_name: String
health_check: HealthCheckConfig
load_balancing: LoadBalancingStrategy
metadata: HashMap<String, String>)
primal_id: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PrimalDiscoveryEngine

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-primal-sdk/src/discovery/discovery_engine.rs`

**Fields:**
```rust
config: CanonicalSongbirdConfig
discovered_primals: HashMap<String, DiscoveredPrimal>)
discovery_cache: HashMap<String, std::time::Instant>)
discovery_stats: DiscoveryStats
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/discovery/engine.rs`

**Fields:**
```rust
_config: PrimalConfig
_discovery_cache: HashMap<String, std::time::Instant>)
discovered_primals: HashMap<String, DiscoveredPrimal>)
discovery_config: DiscoveryConfig
discovery_stats: DiscoveryStats
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### PrimalHealthConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/primals.rs`

**Fields:**
```rust
health_check_interval_secs: u64
health_check_timeout_secs: u64
recovery_threshold: u32
unhealthy_threshold: u32
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/biome/modules/types.rs`

**Fields:**
```rust
capabilities: vec!["generic".to_string()],"
check_interval_secs: u64
check_timeout_secs: u64
endpoint: PrimalEndpoint { primary_url: "http://songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:config.network.http_port".to_string()
fallback_urls: vec![]
health_endpoint: String
use_tls: false
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ProtocolDetectionConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/gaming.rs`

**Fields:**
```rust
confidence_threshold: f32
deep_inspection: bool
enabled: bool
max_packet_size: usize
signature_database_path: Option<String>
timeout_ms: u64
```

**Variant 2:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
detection_rules: Vec<DetectionRule>
detection_timeout: Duration
enabled: bool
supported_protocols: Vec<GameProtocolClass>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ProviderConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/feature_flags.rs`

**Fields:**
```rust
api_key: Option<String>
endpoint: Option<String>
provider_type: String
refresh_interval: Option<u64>
settings: HashMap<String, serde_json::Value>
timeout_ms: u64
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/feature_flags.rs`

**Fields:**
```rust
api_key: Option<String>
endpoint: Option<String>
provider_type: String
refresh_interval: Option<u64>
settings: HashMap<String, serde_json::Value> );
timeout_ms: u64
```

**Variant 3:**
- `crates/songbird-types/src/traits/canonical.rs`

**Fields:**
```rust
enabled_features: Vec<String>
environment: String
settings: HashMap<String, serde_json::Value>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ProxyConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
bind_address: String
bind_port: u16
connection_timeout_ms: u64
enabled: bool
target_address: String
target_port: u16
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
bypass_list: Vec<String>
enabled: bool
proxy_url: String
```

**Variant 3:**
- `crates/songbird-network-federation/src/network/mod.rs`

**Fields:**
```rust
enabled: bool
load_balancing: LoadBalancingStrategy
proxy_type: ProxyType
upstream_servers: Vec<SocketAddr>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### RateLimitConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
burst_size: u32
enabled: bool
requests_per_minute: u32
window_seconds: u32
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/universal_registry/config.rs`

**Fields:**
```rust
burst_size: Option<u32>
max_requests: u32
strategy: RateLimitStrategy
window_duration: Duration
```

**Variant 3:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
burst_size: u32
enabled: bool
requests_per_second: u32
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### RateLimitingConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
burst_size: u32
enabled: bool
requests_per_second: u32
```

**Variant 2:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
algorithm: RateLimitAlgorithm
burst_capacity: u32
enabled: bool
max_requests_per_second: u32
per_client_enabled: bool
per_client_max_requests: u32
window_size_secs: u64
```

**Variant 3:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
algorithm: RateLimitAlgorithm
burst_capacity: u32
enabled: bool
max_requests_per_second: u32
per_client_enabled: bool
per_client_max_requests: u32
window_size: Duration
```

**Variant 4:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
burst_size: u32
refill_rate: f64
requests_per_window: u32
sliding_window: bool
sliding_window_config: SlidingWindowConfig
strategy: RateLimitStrategy
window_duration: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### RegistryConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-orchestrator/src/core/mod.rs`

**Fields:**
```rust
discovery_interval: u64
max_services: u32
service_timeout: u64
```

**Variant 2:**
- `crates/songbird-registry/src/persistence/production_registry.rs`

**Fields:**
```rust
enable_events: bool
health_check_interval: Duration
max_services: usize
persistence_type: PersistenceType
service_ttl: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ResourceConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/resource_management.rs`

**Fields:**
```rust
auto_cleanup: bool
custom_limits: HashMap<String, serde_json::Value>
max_connections: Option<u32>
max_cpu_usage: Option<f64>
max_file_handles: Option<u32>
max_memory_bytes: Option<u64>
timeout: Option<Duration>
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/scalability/types.rs`

**Fields:**
```rust
cpu_limit: f64
cpu_request: f64
memory_limit_mb: u32
memory_request_mb: u32
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
auto_cleanup: bool
custom_limits: HashMap<String, serde_json::Value> );
max_connections: Option<u32>
max_cpu_usage: Option<f64>
max_file_handles: Option<u32>
max_memory_bytes: Option<u64>
timeout: Option<Duration>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ResourceConstraintConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/testing.rs`

**Fields:**
```rust
cpu_limit: Option<f64>
memory_limit_bytes: Option<u64>
```

**Variant 2:**
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
cpu_limit_percent: Option<f64>
disk_io_limit_mbps: Option<u64>
memory_limit_mb: Option<u64>
network_bandwidth_limit_mbps: Option<u64>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ResourceManagementConfig

**2 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/resource_management.rs`
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
cleanup: CleanupConfig
limits: LimitsConfig
monitoring: MonitoringConfig
tracking: TrackingConfig
```

**Variant 2:**
- `crates/songbird-types/src/config/federation.rs`

**Fields:**
```rust
allocation: ResourceAllocationConfig
limits: CanonicalResourceLimits
monitoring: ResourceMonitoringConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### RetryConfig

**8 different implementations** across 9 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
backoff_multiplier: f64
enabled: bool
initial_delay: Duration
jitter: bool
max_attempts: u32
max_delay: Duration
```

**Variant 2:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
backoff_multiplier: f64
backoff_strategy: BackoffStrategy
enabled: bool
initial_delay: Duration
jitter_enabled: bool
max_attempts: u32
max_delay: Duration
retryable_errors: Vec<String>
```

**Variant 3:**
- `crates/songbird-discovery/src/traits/hooks.rs`
- `crates/songbird-orchestrator/src/core/traits/hooks.rs`

**Fields:**
```rust
backoff_multiplier: f64
enabled: bool
max_attempts: u32
retry_delay_ms: u64
```

**Variant 4:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
backoff_multiplier: f64
base_delay_ms: u64
enable_jitter: bool
jitter_percentage: f64
max_delay_ms: u64
max_retries: u32
retry_on_errors: Vec<super::error_types::RetryableError>
```

**Variant 5:**
- `crates/songbird-primal-sdk/src/modern_api.rs`

**Fields:**
```rust
base_delay_ms: u64
max_delay_ms: u64
max_retries: u32
retryable_errors: Vec<String>
strategy: RetryStrategy
```

**Variant 6:**
- `crates/songbird-primal-sdk/src/modern_api/mod.rs`

**Fields:**
```rust
base_delay: Duration
max_attempts: u32
max_delay: Duration
strategy: RetryStrategy
```

**Variant 7:**
- `crates/songbird-universal/src/network_effects_decoupling.rs`

**Fields:**
```rust
backoff_strategy: BackoffStrategy
retry_conditions: Vec<RetryCondition> ;
```

**Variant 8:**
- `crates/songbird-universal/src/types/config.rs`

**Fields:**
```rust
backoff_multiplier: f64
base_delay: Duration
max_attempts: u32
max_delay: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### RobustnessConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
bulkhead: BulkheadConfig
circuit_breaker: CircuitBreakerConfig
health_check: HealthCheckConfig
load_balancer: LoadBalancerConfig
rate_limiting: RateLimitingConfig
retry: RetryConfig
zero_cost_router: ZeroCostRouterConfig
```

**Variant 2:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
bulkhead: BulkheadConfig
circuit_breaker: CircuitBreakerConfig
load_balancer: LoadBalancerConfig
rate_limiting: RateLimitingConfig
retry: RetryConfig
zero_cost_router: ZeroCostRouterConfig
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
bulkhead: BulkheadConfig
circuit_breaker: CircuitBreakerConfig
health_check: HealthCheckConfig
rate_limiting: RateLimitingConfig
retry: RetryConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ScalabilityConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-orchestrator/src/core/scalability/types.rs`

**Fields:**
```rust
enable_predictive_scaling: bool
max_instances: u32
metrics_interval: Duration
min_instances: u32
scale_down_cooldown: Duration
scale_up_cooldown: Duration
target_cpu_threshold: f64
target_memory_threshold: f64
```

**Variant 2:**
- `crates/songbird-types/src/config/performance.rs`

**Fields:**
```rust
auto_scaling_enabled: bool
load_balancing: LoadBalancingConfig
max_instances: u32
min_instances: u32
scale_down_cpu_threshold: f64
scale_down_memory_threshold: f64
scale_up_cpu_threshold: f64
scale_up_memory_threshold: f64
scaling_cooldown: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ScalingConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/orchestration.rs`

**Fields:**
```rust
check_interval_seconds: u64
enabled: bool
max_instances: u32
min_instances: u32
target_cpu_percent: f64
target_memory_percent: f64
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/mod.rs`

**Fields:**
```rust
enable_auto_scaling: bool
max_instances: u32
min_instances: u32
scale_down_threshold: f64
scale_up_threshold: f64
```

**Variant 3:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
auto_scaling_enabled: bool
max_instances: usize
min_instances: usize
scale_down_cpu_threshold: f64
scale_down_memory_threshold: f64
scale_up_cpu_threshold: f64
scale_up_memory_threshold: f64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### SecurityConfig

**8 different implementations** across 8 definitions

**Variant 1:**
- `crates/songbird-cli/src/cli/commands/firewall.rs`

**Fields:**
```rust
security_level: String
```

**Variant 2:**
- `crates/songbird-cli/src/cli/commands/quick.rs`

**Fields:**
```rust
allow_insecure_networks: bool
enable_audit_logging: bool
require_tls: bool
```

**Variant 3:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
beardog_endpoint: String
encryption_key_size: usize
oauth_redirect_uri: String
session_timeout: Duration
tls_cert_path: String
```

**Variant 4:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
audit_logging: AuditConfig
authentication: AuthConfig
authorization: AuthzConfig
enabled: bool
encryption: EncryptionConfig
rate_limiting: RateLimitConfig
```

**Variant 5:**
- `crates/songbird-execution-agent/src/security_sovereign.rs`

**Fields:**
```rust
auth_tokens: Vec<String>
enable_auth: bool
enable_beardog_discovery: bool
max_timeout_seconds: u64
```

**Variant 6:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
api_key_auth: bool
ca_cert_path: Option<String>
client_cert_path: Option<String>
client_key_path: Option<String>
encryption: EncryptionConfig
jwt_auth: bool
tls_enabled: bool
verify_certificates: bool
```

**Variant 7:**
- `crates/songbird-primal-sdk/src/security_provider.rs`

**Fields:**
```rust
discovery_refresh_secs: u64
enable_fallback: bool
operation_timeout_secs: u64
require_redundancy: bool
```

**Variant 8:**
- `crates/songbird-universal/src/types/config.rs`

**Fields:**
```rust
authentication_required: bool
certificate_path: Option<String>
enabled: bool
level: SecurityLevel
tls_enabled: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### SecurityProviderIntegrationConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/gaming.rs`

**Fields:**
```rust
auth: GamingAuthConfig
enabled: bool
endpoint: Option<String>
monitoring: SecurityProviderMonitoringConfig
performance: GamingPerformanceSettings
security: GamingSecuritySettings
```

**Variant 2:**
- `crates/songbird-types/src/config/security.rs`

**Fields:**
```rust
enabled: bool
providers: HashMap<String, SecurityProviderConfig>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ServiceConfig

**5 different implementations** across 5 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/service.rs`

**Fields:**
```rust
address: String
health_check: Option<HealthCheckConfig>
metadata: HashMap<String, String>
name: String
port: u16
```

**Variant 2:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
base_url: String
health_endpoint: String
metrics_endpoint: String
service_name: String
version: String
```

**Variant 3:**
- `crates/songbird-config/src/performance.rs`

**Fields:**
```rust
endpoint: String
name: String
port: u16
timeout: Duration ;
```

**Variant 4:**
- `crates/songbird-config/src/unified/core.rs`

**Fields:**
```rust
instance_id: String
name: String
tags: Vec<String>
version: String
```

**Variant 5:**
- `crates/songbird-config/src/zero_touch/deployment.rs`

**Fields:**
```rust
deployment_method: Option<String>
environment_variables: HashMap<String, String>)
health_check_path: Option<String>
image: String
name: String
ports: Vec<u16>
resource_limits: Option<ResourceLimits>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ServiceDiscoveryConfig

**6 different implementations** across 6 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/orchestration.rs`

**Fields:**
```rust
enabled: bool
interval_seconds: u64
max_services: usize
timeout_ms: u64
```

**Variant 2:**
- `crates/songbird-config/src/canonical/discovery.rs`

**Fields:**
```rust
discovery_interval_secs: u64
discovery_timeout_secs: u64
enabled: bool
max_concurrent_discoveries: usize
```

**Variant 3:**
- `crates/songbird-types/src/config/discovery.rs`

**Fields:**
```rust
auto_register: bool
max_federation_nodes: u32
metadata: HashMap<String, String>)
service_name: String
tags: Vec<String>
trust_verification_enabled: bool
```

**Variant 4:**
- `crates/songbird-types/src/config/discovery_corrupted.rs`

**Fields:**
```rust
auto_register: bool
max_federation_nodes: usize
metadata: HashMap<String, String>)
service_name: String
tags: Vec<String>
trust_verification_enabled: bool
```

**Variant 5:**
- `crates/songbird-types/src/config/environment.rs`

**Fields:**
```rust
auto_discovery: bool
discovery_timeout: Duration
fallback_endpoints: HashMap<String, String>
health_checks: EnvironmentHealthCheckConfig
refresh_interval: Duration
```

**Variant 6:**
- `crates/songbird-universal/src/service_discovery.rs`

**Fields:**
```rust
discovery_interval_secs: u64
enable_env_discovery: bool
enable_network_discovery: bool
health_check_timeout_secs: u64
max_services: usize
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ServiceFailureConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/unified/testing.rs`

**Fields:**
```rust
failure_rate: f64
mean_time_to_failure: Duration
```

**Variant 2:**
- `crates/songbird-test-utils/src/chaos_engineering/config.rs`

**Fields:**
```rust
failure_rate: f64
mean_time_to_failure: Duration
mean_time_to_recovery: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ServiceMeshConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-discovery/src/agnostic_service_mesh.rs`

**Fields:**
```rust
cache_expiry_ms: u64
detection_patterns: Vec<DetectionPattern>
min_confidence: f32
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/capability_compute.rs`

**Fields:**
```rust
name: String
routing_rules: HashMap<String, String>
security_policies: Vec<String>
services: Vec<String>
```

**Variant 3:**
- `crates/songbird-primal-sdk/src/toadstool.rs`

**Fields:**
```rust
name: String
routing_rules: HashMap<String, String>)
security_policies: Vec<String>
services: Vec<String>
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### SessionConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/security.rs`

**Fields:**
```rust
max_concurrent_sessions: Option<u32>
persistent: bool
timeout_secs: u64
```

**Variant 2:**
- `crates/songbird-config/src/unified/api.rs`

**Fields:**
```rust
buffer_size: usize
cleanup_interval: Duration
enable_persistence: bool
keep_alive_interval: Duration
max_concurrent_sessions: usize
session_timeout: Duration
```

**Variant 3:**
- `crates/songbird-network-federation/src/network/gaming.rs`

**Fields:**
```rust
max_players: u32
name: String
password: Option<String>
properties: HashMap<String, String>
public: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### StorageConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-orchestrator/src/core/biome/byob_coordinator/integration.rs`

**Fields:**
```rust
capabilities: Vec<String>
config_data: serde_json::Value,;
endpoint: String
is_active: bool;
primal_name: String
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/capability_storage.rs`

**Fields:**
```rust
fallback_strategies: Vec<StorageFallbackStrategy>
quality_requirements: StorageQualityRequirements
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### TimeoutConfig

**4 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/network.rs`

**Fields:**
```rust
connection_timeout_secs: u64
default_timeout_secs: u64
discovery_timeout_secs: u64
health_check_timeout_secs: u64
registration_timeout_secs: u64
```

**Variant 2:**
- `crates/songbird-config/src/config/hardcoded_elimination.rs`

**Fields:**
```rust
connection_timeout: Duration
health_check_timeout: Duration
heartbeat_interval: Duration
request_timeout: Duration
scaling_check_interval: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/robustness/config.rs`

**Fields:**
```rust
adaptive: bool
adaptive: false
decrease_factor: 0.95
decrease_factor: f64
default_timeout: Duration
increase_factor: 1.5
increase_factor: f64
max_timeout: Duration
max_timeout: Duration::from_secs(300
min_timeout: Duration
min_timeout: Duration::from_secs(1
p95_threshold: Duration
p95_threshold: Duration::from_secs(5)
sample_size: 100;
```

**Variant 4:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
connection_timeout_seconds: u64
default_request_timeout_seconds: u64
read_timeout_seconds: u64
write_timeout_seconds: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### TlsConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
ca_path: Option<String>
cert_path: String
enabled: bool
key_path: String
verify_client: bool
```

**Variant 2:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
ca_file: Option<String>
cert_file: Option<String>
key_file: Option<String>
server_name: Option<String>
verify_peer: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### TracingConfig

**3 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/observability.rs`
- `crates/songbird-config/src/unified/observability.rs`

**Fields:**
```rust
enabled: bool
endpoint: Option<String>
sample_rate: f64
```

**Variant 2:**
- `crates/songbird-config/src/config/mod.rs`

**Fields:**
```rust
enabled: bool
exporters: Vec<TracingExporter>
max_span_attributes: u32
sample_rate: f64
```

**Variant 3:**
- `crates/songbird-primal-sdk/src/config.rs`

**Fields:**
```rust
enabled: bool
format: String
include_location: bool
level: String
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### TrackingConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/traits/resource_management.rs`

**Fields:**
```rust
enabled: bool
max_tracked_resources: Option<u32>
reporting_interval: u64
tracking_options: TrackingOptions
```

**Variant 2:**
- `crates/songbird-orchestrator/src/core/traits/resource_management.rs`

**Fields:**
```rust
enabled: bool
max_tracked_resources: Option<u32>
track_connections: bool
track_cpu: bool
track_file_handles: bool
track_memory: bool
tracking_interval: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### TrustConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-discovery/src/discovery/config/mod.rs`

**Fields:**
```rust
edu_domain_bonus: u32
gov_domain_bonus: u32
institutional_base_score: u32
interaction_penalties: InteractionPenalties
reputation_weight: f64
service_diversity_weight: u32
trust_thresholds: TrustThresholds
uptime_weight: u32
```

**Variant 2:**
- `crates/songbird-universal/src/unified_agnostic_discovery.rs`

**Fields:**
```rust
initial_trust: f64
interaction_weight: f64
minimum_trust_threshold: f64
trust_boost_factor: f64
trust_decay_rate: f64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### UnifiedObservabilityConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/observability.rs`

**Fields:**
```rust
dashboard: DashboardConfig
logging: LoggingConfig
tracing: TracingConfig
```

**Variant 2:**
- `crates/songbird-config/src/unified/observability.rs`

**Fields:**
```rust
dashboard: DashboardConfig
health_checks: HealthCheckConfig
logging: LoggingConfig
tracing: TracingConfig
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### UniversalAdapterConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-canonical/src/config/adapters.rs`

**Fields:**
```rust
compute_adapters: ComputeAdapterConfig
security_adapters: SecurityAdapterConfig
settings: AdapterSettings
storage_adapters: StorageAdapterConfig
```

**Variant 2:**
- `crates/songbird-primal-sdk/src/universal_adapter/types.rs`

**Fields:**
```rust
discovery_interval_secs: u64
enable_detailed_logging: bool
enable_performance_monitoring: bool
health_check_interval_secs: u64
max_concurrent_operations: usize
request_timeout_secs: u64
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### WebSocketConfig

**2 different implementations** across 2 definitions

**Variant 1:**
- `crates/songbird-types/src/config/communication.rs`

**Fields:**
```rust
compression_enabled: bool
headers: HashMap<String, String>
max_frame_size: usize
max_message_size: usize
ping_interval: Duration
pong_timeout: Duration
subprotocols: Vec<String>
timeout: Duration
```

**Variant 2:**
- `crates/songbird-types/src/config/network.rs`

**Fields:**
```rust
connection_timeout: Duration
heartbeat_interval: Duration
max_connections: u32
message_buffer_size: usize
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ZeroCostRouterConfig

**3 different implementations** across 3 definitions

**Variant 1:**
- `crates/songbird-config/src/canonical/resilience.rs`

**Fields:**
```rust
discovery_timeout_ms: u64
enabled: bool
max_route_depth: usize
optimize_routes: bool
route_cache_size: usize
route_cache_ttl_secs: u64
```

**Variant 2:**
- `crates/songbird-config/src/unified/robustness.rs`

**Fields:**
```rust
discovery_timeout: Duration
enabled: bool
max_route_depth: usize
optimize_routes: bool
route_cache_size: usize
route_cache_ttl: Duration
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/zero_cost_request_router.rs`

**Fields:**
```rust
default_timeout: Duration
enable_request_tracing: bool
max_retries: u32
retry_delay: Duration
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

### ZeroTouchConfig

**3 different implementations** across 4 definitions

**Variant 1:**
- `crates/songbird-config/src/zero_touch/infant_config.rs`
- `crates/songbird-config/src/zero_touch_config.rs`

**Fields:**
```rust
bootstrap: BootstrapConfig
discovery: DiscoveryConfig
network: NetworkConfig
optional_capabilities: Vec<CapabilityRequirement>
required_capabilities: Vec<CapabilityRequirement>
self_identity: ServiceIdentity
```

**Variant 2:**
- `crates/songbird-config/src/zero_touch/mod.rs`

**Fields:**
```rust
auto_deploy: bool
environment_detection: bool
```

**Variant 3:**
- `crates/songbird-orchestrator/src/core/mod.rs`

**Fields:**
```rust
deployment_strategy: DeploymentStrategy
enable_auto_deployment: bool
rollback_on_failure: bool
```

**Action:** Review to determine if:
- Variants should be unified (accidental divergence)
- Variants should be renamed for clarity (legitimate differences)

---

## Recommendations

### Immediate Actions (True Duplicates)
1. Consolidate the 13 TRUE duplicates marked with ✅
2. Each consolidation: ~30 minutes (proven process)
3. Replace all occurrences with re-exports to canonical

### Review Actions (Domain Variants)
1. Review each of the 105 domain-specific variants
2. Determine if differences are legitimate or accidental
3. Either:
   - Unify if differences are accidental
   - Rename for clarity if legitimate (e.g., NetworkConfig → EdgeNetworkConfig)

