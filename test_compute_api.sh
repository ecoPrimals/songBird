#!/bin/bash
# Test Songbird Compute API Integration

set -e

SONGBIRD_URL="${SONGBIRD_URL:-http://localhost:8080}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Testing Songbird Compute API Integration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Songbird URL: $SONGBIRD_URL"
echo ""

# Test 1: Health Check
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
curl -s "$SONGBIRD_URL/health" && echo "" || echo "❌ Health check failed"
echo ""

# Test 2: Submit Lightweight Task
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Submit Lightweight Task"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
RESPONSE=$(curl -s -X POST "$SONGBIRD_URL/api/compute/task" \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "health_check",
      "payload": {},
      "resource_requirements": null,
      "estimated_duration_secs": null,
      "metadata": {}
    },
    "priority": 5,
    "timeout_secs": 30
  }')

echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id' 2>/dev/null)
echo ""

if [ "$JOB_ID" != "null" ] && [ -n "$JOB_ID" ]; then
  echo "✅ Task submitted successfully: $JOB_ID"
  echo ""
  
  # Test 3: Query Task Status
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Test 3: Query Task Status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  curl -s "$SONGBIRD_URL/api/compute/task/$JOB_ID" | jq '.' 2>/dev/null || curl -s "$SONGBIRD_URL/api/compute/task/$JOB_ID"
  echo ""
else
  echo "❌ Task submission failed"
fi

# Test 4: Submit Heavy Task (GPU ML Training)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Submit Heavy Task (GPU ML Training)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Set capability endpoint for test
export CAPABILITY_COMPUTE_ENDPOINT="http://localhost:9000"

RESPONSE=$(curl -s -X POST "$SONGBIRD_URL/api/compute/task" \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "ml_training",
      "payload": {
        "model": "resnet50",
        "dataset": "imagenet"
      },
      "resource_requirements": {
        "cpu_cores": 8,
        "memory_mb": 16384,
        "gpu_required": true,
        "storage_mb": 10240,
        "network_mbps": 100
      },
      "estimated_duration_secs": 600,
      "metadata": {
        "experiment": "test_training"
      }
    },
    "priority": 10,
    "timeout_secs": 1800
  }')

echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
GPU_JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id' 2>/dev/null)
echo ""

if [ "$GPU_JOB_ID" != "null" ] && [ -n "$GPU_JOB_ID" ]; then
  echo "✅ GPU task submitted successfully: $GPU_JOB_ID"
  ROUTED_TO=$(echo "$RESPONSE" | jq -r '.routed_to' 2>/dev/null)
  echo "   Routed to: $ROUTED_TO"
  echo ""
  
  # Query GPU task status
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "GPU Task Status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  curl -s "$SONGBIRD_URL/api/compute/task/$GPU_JOB_ID" | jq '.' 2>/dev/null || curl -s "$SONGBIRD_URL/api/compute/task/$GPU_JOB_ID"
  echo ""
else
  echo "❌ GPU task submission failed"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Compute API Integration Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "API Endpoints:"
echo "  • POST   $SONGBIRD_URL/api/compute/task"
echo "  • GET    $SONGBIRD_URL/api/compute/task/:job_id"
echo "  • GET    $SONGBIRD_URL/health"
echo "  • GET    $SONGBIRD_URL/api/federation/status"
echo ""
echo "This is proper ecoPrimals sovereign orchestration! 🐦🍄🔐"
echo ""

