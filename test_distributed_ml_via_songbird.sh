#!/bin/bash
# Test Distributed ML via Songbird Compute API

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Testing Distributed ML via Songbird"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 1: Lightweight task (should execute locally or on peer)
echo "Test 1: Lightweight Task"
curl -s -X POST http://192.168.1.144:8080/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "health_check",
      "resource_requirements": {
        "cpu_cores": 1.0,
        "memory_mb": 512,
        "gpu_required": false
      }
    }
  }' | python3 -m json.tool 2>/dev/null || echo "API response received"
echo ""

# Test 2: Heavy GPU task (should route to Toadstool/Compute capability)
echo "Test 2: Heavy GPU Task"
curl -s -X POST http://192.168.1.144:8080/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{
    "task": {
      "task_type": "ml_training",
      "payload": {
        "model": "resnet50",
        "dataset": "imagenet100",
        "epochs": 5
      },
      "resource_requirements": {
        "cpu_cores": 4.0,
        "memory_mb": 8192,
        "gpu_required": true
      }
    }
  }' | python3 -m json.tool 2>/dev/null || echo "API response received"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Check logs: tail -f /tmp/songbird.log"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

