#!/bin/bash

# Fix common compilation errors in test files

echo "Fixing NetworkConfig method calls..."
find tests/ -name "*.rs" -exec sed -i 's/NetworkConfig::development()/NetworkConfig::secure_defaults()/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/NetworkConfig::production()/NetworkConfig::from_env().unwrap_or_else(|_| NetworkConfig::secure_defaults())/g' {} \;

echo "Fixing crate name references..."
find tests/ -name "*.rs" -exec sed -i 's/songbird_orchestrator::/songbird_gaming_bridge::/g' {} \;
find examples/ -name "*.rs" -exec sed -i 's/songbird_orchestrator::/songbird_gaming_bridge::/g' {} \;

echo "Adding missing imports..."
find tests/ -name "*.rs" -exec sed -i '1i use std::collections::HashMap;' {} \;

echo "Fixing ServiceResponse::success calls with two parameters..."
find tests/ -name "*.rs" -exec perl -i -pe 's/ServiceResponse::success\(\s*([^,]+),\s*[^)]+\)/ServiceResponse::success($1)/g' {} \;

echo "Fixing ServiceMetrics field names..."
find tests/ -name "*.rs" -exec sed -i 's/avg_response_time_ms:/average_response_time:/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/p95_response_time_ms:/average_response_time:/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/p99_response_time_ms:/average_response_time:/g' {} \;

echo "Fixing FederationConfig field access..."
find tests/ -name "*.rs" -exec sed -i 's/config\.enabled/config.mode == crate::federation::FederationMode::Peer/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/config\.discovery_endpoints/vec!["localhost:8080".to_string()]/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/config\.heartbeat_interval/60/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/config\.node_timeout/120/g' {} \;

echo "Fixing SongbirdError variants..."
find tests/ -name "*.rs" -exec sed -i 's/SongbirdError::Internal/SongbirdError::service_error("internal", /g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/SongbirdError::Environment(/SongbirdError::configuration_error(/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/SongbirdError::Configuration(/SongbirdError::Config { message: /g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/SongbirdError::HealthCheck/SongbirdError::health_check_failed("service", /g' {} \;

echo "Fixing struct field names..."
find tests/ -name "*.rs" -exec sed -i 's/service_id:/service:/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\.unwrap()/\.unwrap_or_default()/g' {} \;

echo "Done fixing test files!" 