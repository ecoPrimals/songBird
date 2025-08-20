#!/bin/bash
FILE="crates/songbird-registry/src/zero_cost_service_registry.rs"

# Fix QoSMetrics import issue
sed -i 's/QoSMetrics/QosMetrics/g' "$FILE"

# Add capability_type field before name field  
sed -i 's/name: "persistence"/capability_type: "storage".to_string(),\n                    name: "persistence"/g' "$FILE"
sed -i 's/name: "backup"/capability_type: "storage".to_string(),\n                    name: "backup"/g' "$FILE"
sed -i 's/name: "processing"/capability_type: "compute".to_string(),\n                    name: "processing"/g' "$FILE"
sed -i 's/name: "metrics"/capability_type: "compute".to_string(),\n                    name: "metrics"/g' "$FILE"
sed -i 's/name: "inference"/capability_type: "ai".to_string(),\n                    name: "inference"/g' "$FILE"
sed -i 's/name: "training"/capability_type: "ai".to_string(),\n                    name: "training"/g' "$FILE"

# Add parameters field after version field
sed -i 's/version: "1\.0\.0"/version: "1.0.0".to_string(),\n                    parameters: std::collections::HashMap::new()/g' "$FILE"

# Remove endpoint field lines
sed -i '/endpoint: /d' "$FILE"

# Fix provider field to provider_name with proper closing
sed -i 's/provider_name: Some("NestGate".to_string(),/provider_name: Some("NestGate".to_string()),/g' "$FILE"
sed -i 's/provider_name: Some("ToadStool".to_string(),/provider_name: Some("ToadStool".to_string()),/g' "$FILE"
sed -i 's/provider_name: Some("Squirrel".to_string(),/provider_name: Some("Squirrel".to_string()),/g' "$FILE"

# Remove health_status field and add available field
sed -i '/health_status: /d' "$FILE"
sed -i 's/},$/},\n                    available: true,/g' "$FILE"

echo "Fixed Capability struct definitions"
