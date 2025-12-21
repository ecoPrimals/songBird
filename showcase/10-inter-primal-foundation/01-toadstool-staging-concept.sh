#!/bin/bash
# Songbird + Toadstool - Staging Concept Demo
# Shows: Songbird routing to Toadstool's ML capabilities
# Status: STAGING (uses Toadstool's existing demos as backend)

set -e

SONGBIRD_URL="https://localhost:8080"
TOADSTOOL_DIR="../../../toadstool"
OUTPUT_DIR="./outputs/toadstool-staging-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  Songbird + Toadstool Integration (Staging Concept)              ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "🎯 Goal: Show Songbird managing Toadstool compute"
echo "📍 Status: STAGING (will be fully automated)"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! curl -sk "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo "❌ Songbird not running"
    echo "   Start: ./target/release/songbird-orchestrator"
    exit 1
fi
echo "✅ Songbird operational"

if [ ! -d "$TOADSTOOL_DIR" ]; then
    echo "❌ Toadstool not found at $TOADSTOOL_DIR"
    exit 1
fi
echo "✅ Toadstool directory found"
echo ""

# Demonstrate the pattern (conceptual)
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  Current Pattern (Manual)                                        ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "1. User → Songbird: \"I need GPU compute\""
echo "2. Songbird: \"Let me check available compute services...\""
echo "3. [MANUAL] Operator starts Toadstool"
echo "4. [MANUAL] Operator configures endpoint"
echo "5. Songbird → Toadstool: Route task"
echo "6. Toadstool: Execute"
echo "7. Toadstool → Songbird: Return result"
echo "8. Songbird → User: Here's your result"
echo ""

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  Target Pattern (Automated)                                      ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "1. User → Songbird: \"I need GPU compute\""
echo "2. Songbird: \"Discovering compute services...\""
echo "3. Toadstool (auto-starts): \"I'm here! I have GPU!\""
echo "4. Toadstool → Songbird: Register(capabilities: [gpu, python])"
echo "5. Songbird: \"Great! You're port 8091\""
echo "6. Songbird → Toadstool: Route task"
echo "7. Toadstool: Execute"
echo "8. Toadstool → Songbird: Return result"
echo "9. Songbird → User: Here's your result"
echo ""
echo "✨ Zero manual configuration!"
echo ""

# Show what Toadstool can do
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  Toadstool Capabilities (Available for Integration)              ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

if [ -d "$TOADSTOOL_DIR/showcase/gpu-universal" ]; then
    echo "✅ GPU Compute (CUDA, OpenCL, Metal)"
    echo "   - Matrix operations"
    echo "   - ML inference"
    echo "   - Training"
fi

if [ -d "$TOADSTOOL_DIR/showcase/python-ml" ]; then
    echo "✅ Python ML Runtime"
    echo "   - PyTorch"
    echo "   - TensorFlow"
    echo "   - Scikit-learn"
fi

if [ -d "$TOADSTOOL_DIR/showcase/neuromorphic" ]; then
    echo "✅ Neuromorphic Computing"
    echo "   - Akida acceleration"
    echo "   - Bioinformatics"
fi

echo ""

# Simulate the flow
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  Simulated Integration Flow                                      ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

echo "📝 Step 1: User submits ML task to Songbird"
cat > "${OUTPUT_DIR}/task-request.json" << 'EOF'
{
  "task_type": "ml_training",
  "code": "train_mnist.py",
  "runtime": "python",
  "requires": ["gpu", "pytorch"],
  "description": "Train MNIST classifier"
}
EOF

echo "   Request: $(cat ${OUTPUT_DIR}/task-request.json | jq -c '.')"
echo ""

echo "📡 Step 2: Songbird analyzes requirements"
echo "   Analysis:"
echo "     - Requires: GPU ✓"
echo "     - Requires: Python ✓"
echo "     - Requires: PyTorch ✓"
echo "   Decision: Route to Toadstool"
echo ""

echo "🔄 Step 3: Songbird → Toadstool (via REST API)"
echo "   POST https://localhost:8091/api/v1/compute/execute"
echo "   Authorization: Bearer <songbird-token>"
echo "   Body: {task details}"
echo ""

echo "⚙️  Step 4: Toadstool executes"
echo "   [In Toadstool] Loading model..."
echo "   [In Toadstool] Training epoch 1/10..."
echo "   [In Toadstool] Training epoch 2/10..."
echo "   [In Toadstool] ..."
echo "   [In Toadstool] Training complete! Accuracy: 98.7%"
echo ""

echo "✅ Step 5: Toadstool → Songbird (result)"
cat > "${OUTPUT_DIR}/task-result.json" << 'EOF'
{
  "status": "completed",
  "result": {
    "accuracy": 0.987,
    "loss": 0.042,
    "epochs": 10,
    "duration_sec": 127.5
  },
  "model_path": "toadstool://models/mnist-20251220-183045",
  "logs": "toadstool://logs/task-abc123"
}
EOF

echo "   Result: $(cat ${OUTPUT_DIR}/task-result.json | jq -c '.')"
echo ""

echo "🎉 Step 6: Songbird → User (final result)"
echo "   \"Your model is trained! Accuracy: 98.7%\""
echo "   \"Model available at: toadstool://models/mnist-20251220-183045\""
echo ""

# Show current vs future architecture
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  Architecture Evolution                                          ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

echo "CURRENT (Staging):"
echo "┌─────────┐"
echo "│ Songbird│"
echo "└────┬────┘"
echo "     │ (manual routing)"
echo "     ↓"
echo "┌──────────┐"
echo "│ Toadstool│ (port 8091, hardcoded)"
echo "└──────────┘"
echo ""

echo "TARGET (Automated):"
echo "┌─────────┐"
echo "│ Songbird│ ← Port Authority"
echo "└────┬────┘"
echo "     │ (automatic discovery)"
echo "     ↓"
echo "┌──────────┐"
echo "│ Toadstool│ (Songbird assigns port)"
echo "│  registers│"
echo "│  with SB │"
echo "└──────────┘"
echo ""

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  What We're Building                                             ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

cat > "${OUTPUT_DIR}/integration-plan.md" << 'EOF'
# Songbird + Toadstool Integration Plan

## Phase 1: Staging (Current)
- ✅ Toadstool has working ML demos
- ✅ Toadstool has REST API
- ⚠️  Manual port configuration
- ⚠️  Manual service discovery
- ⚠️  Songbird routes manually

## Phase 2: Registration Protocol
- [ ] Add Songbird client to Toadstool
- [ ] Toadstool registers on startup
- [ ] Songbird assigns port
- [ ] Automatic discovery

## Phase 3: Live Integration
- [ ] End-to-end demo working
- [ ] Zero manual configuration
- [ ] Toadstool never sets own port

## Phase 4: Federated Toadstool
- [ ] Toadstool on multiple towers
- [ ] Songbird routes across towers
- [ ] Network effects emerge

## Phase 5: Pattern Template
- [ ] Document registration protocol
- [ ] Other primals follow pattern:
  - Nestgate (data)
  - BearDog (security)
  - Squirrel (AI-MCP)
EOF

cat "${OUTPUT_DIR}/integration-plan.md"
echo ""

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Staging Demo Complete                                         ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "📁 Output saved to: ${OUTPUT_DIR}/"
echo ""
echo "🎯 Next Steps:"
echo "  1. Review Toadstool's existing demos:"
echo "     cd ../../../toadstool/showcase/"
echo ""
echo "  2. Design registration protocol:"
echo "     specs/PRIMAL_REGISTRATION_PROTOCOL.md"
echo ""
echo "  3. Implement Toadstool → Songbird registration"
echo "  4. Build live demo"
echo "  5. Extend to federated Toadstool (multi-tower)"
echo ""
echo "🎵 Songbird: Universal Orchestrator"
echo "🍄 Toadstool: Universal Compute"
echo "🤝 Together: Zero-Config Ecosystem"

