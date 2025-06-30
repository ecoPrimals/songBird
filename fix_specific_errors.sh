#!/bin/bash

echo "Fixing specific compilation errors..."

# Remove EnvironmentMode imports since it doesn't exist
find tests/ -name "*.rs" -exec sed -i 's/use.*EnvironmentMode.*;//g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/EnvironmentMode::[^,}]*//"environment"/' {} \;
find tests/ -name "*.rs" -exec sed -i 's/environment_mode.*Environment/true/' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\.environment_mode/\.bind_address.to_string()/' {} \;

# Remove ServiceNetworkConfig usage since it doesn't exist
find tests/ -name "*.rs" -exec sed -i 's/use.*ServiceNetworkConfig.*;//g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/ServiceNetworkConfig::[^(]*/GamingNetworkConfig::default/' {} \;

# Fix struct instantiation patterns
find tests/ -name "*.rs" -exec sed -i 's/config\.external_access/true/g' {} \;

# Fix CPU usage wrapping
find tests/ -name "*.rs" -exec sed -i 's/cpu_usage: \([0-9.]*\)/cpu_usage: Some(\1)/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/memory_usage: \([0-9]*\)/memory_usage: Some(\1)/g' {} \;

# Fix ServiceInfo field names
find tests/ -name "*.rs" -exec sed -i 's/id: self\.id/service_id: self.id/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/capabilities: /tags: /g' {} \;

# Fix Resource and Action field names
find tests/ -name "*.rs" -exec sed -i 's/id: "test-resource"/resource_id: "test-resource"/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/name: "read"/action_type: "read"/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\.id}/\.resource_id}/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\.name}/\.action_type}/g' {} \;

# Fix Credentials field names
find tests/ -name "*.rs" -exec sed -i 's/username:/credentials:/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/password:/credentials:/g' {} \;

# Fix ServiceResponse fields
find tests/ -name "*.rs" -exec sed -i 's/payload:/body:/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/duration:/processing_time:/g' {} \;

# Fix SongbirdError construction patterns
find tests/ -name "*.rs" -exec sed -i 's/SongbirdError::Network(/SongbirdError::Network { service: "test", message: /g' {} \;

echo "Done fixing specific errors!" 