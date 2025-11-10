# 🐿️ Squirrel AI/MCP Integration Plan

**Date:** November 9, 2025  
**Goal:** Distributed AI mesh across towers + cloud API integration  
**Status:** Planning phase

---

## 🎯 Objective

Integrate Squirrel (AI/MCP primal) into the 2-tower federation to create a **distributed AI mesh** that:
1. Runs local AI models across both towers
2. Pools compute resources (CPU + GPU)
3. Routes to cloud APIs (Anthropic, OpenAI) when needed
4. Minimizes latency, maximizes throughput

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   AI Request Router                         │
│                   (Songbird Orchestrator)                   │
│                                                             │
│  • Analyze prompt complexity                                │
│  • Estimate compute requirements                            │
│  • Route to: Local towers OR Cloud APIs                     │
│                                                             │
└────┬──────────────────────────┬───────────────────┬─────────┘
     │                          │                   │
     ▼                          ▼                   ▼
┌─────────────────┐   ┌─────────────────┐   ┌──────────────┐
│  Tower A        │   │  Tower B        │   │  Cloud APIs  │
│  192.168.1.144  │   │  192.168.1.134  │   │              │
│                 │   │                 │   │              │
│  Squirrel AI    │   │  Squirrel AI    │   │  Anthropic   │
│  Local Model:   │   │  Local Model:   │   │  (Claude)    │
│  • Llama 3 8B   │   │  • Mistral 7B   │   │              │
│  • CPU only     │   │  • GPU accel    │   │  OpenAI      │
│  • Fast, small  │   │  • 8x A6000     │   │  (GPT-4)     │
│                 │   │  • Large models │   │              │
│  Use: Simple    │   │  Use: Complex   │   │  Use: When   │
│  prompts        │   │  prompts        │   │  local       │
│                 │   │                 │   │  insufficient│
└─────────────────┘   └─────────────────┘   └──────────────┘
```

---

## 🎯 Use Cases

### Use Case 1: Simple Local Prompt
```
User: "Summarize this text"
↓
Router: Complexity LOW → Route to Tower A (Llama 3 CPU)
↓
Tower A: Process in 500ms
↓
Response: "Summary: ..."
```

### Use Case 2: Complex Local Prompt
```
User: "Generate detailed code with explanations"
↓
Router: Complexity MEDIUM → Route to Tower B (Mistral GPU)
↓
Tower B: Process in 2s on GPU
↓
Response: "Here's the code..."
```

### Use Case 3: Very Complex Prompt
```
User: "Analyze this 100-page document"
↓
Router: Complexity HIGH → Route to Anthropic (Claude)
↓
Cloud API: Process in 30s
↓
Response: "Analysis: ..."
```

### Use Case 4: Distributed Inference
```
User: "Process 100 prompts in parallel"
↓
Router: Split across both towers
↓
Tower A: 50 prompts (simpler ones)
Tower B: 50 prompts (complex ones)
↓
Results: Aggregated in 10s (vs 100s sequential)
```

---

## 📊 Performance Targets

### Local Models (1Gbps LAN, upgrading to 10Gbps)

| Model | Tower | Hardware | Tokens/sec | Latency | Use |
|-------|-------|----------|------------|---------|-----|
| **Llama 3 8B** | A | CPU | 10-20 | 500ms | Simple prompts |
| **Mistral 7B** | B | GPU | 50-100 | 200ms | Complex prompts |
| **Llama 3 70B** | B | 8x A6000 | 20-30 | 2s | Very complex |

### Cloud APIs

| API | Tokens/sec | Latency | Cost | Use |
|-----|------------|---------|------|-----|
| **Anthropic Claude** | Varies | 2-5s | $$$ | Max quality |
| **OpenAI GPT-4** | Varies | 2-5s | $$$ | General |
| **OpenAI GPT-3.5** | Varies | 1-2s | $$ | Fast, cheap |

### Network Considerations (Current: 1Gbps, Future: 10Gbps)

| Network | Token latency | Bandwidth | Status |
|---------|---------------|-----------|--------|
| **1Gbps LAN** | 10-20ms | 125 MB/s | ✅ Current |
| **10Gbps LAN** | 1-2ms | 1250 MB/s | 🚧 Planned |

**Impact of 10G upgrade:**
- 10x lower token latency (10-20ms → 1-2ms)
- 10x higher bandwidth (125 MB/s → 1250 MB/s)
- Enables larger model streaming
- Supports more concurrent requests

---

## 🚀 Implementation Phases

### Phase 1: Deploy Squirrel to Both Towers ✅
**Goal:** Get Squirrel running on both towers

**Steps:**
1. Build Squirrel binary
2. Deploy to Tower A via Songbird HTTP API
3. Deploy to Tower B via Songbird HTTP API
4. Verify health endpoints
5. Test basic AI requests

**Expected time:** 5-10 minutes

### Phase 2: Local Model Setup
**Goal:** Load AI models on both towers

**Tower A (CPU):**
- Model: Llama 3 8B (quantized)
- Memory: ~5GB
- Tokens/sec: 10-20

**Tower B (GPU):**
- Model: Mistral 7B
- Memory: ~15GB GPU
- Tokens/sec: 50-100

**Steps:**
1. Download models (if not cached)
2. Configure Squirrel with model paths
3. Test inference on each tower
4. Measure baseline performance

### Phase 3: Request Router
**Goal:** Intelligent routing of AI requests

**Logic:**
```python
def route_request(prompt):
    complexity = analyze_prompt(prompt)
    
    if complexity < 100:
        return "tower-a-cpu"  # Fast, simple
    elif complexity < 500:
        return "tower-b-gpu"  # Medium, GPU
    elif complexity < 1000:
        return "tower-b-gpu-large"  # Complex, large model
    else:
        return "anthropic-claude"  # Very complex, cloud
```

### Phase 4: Distributed Inference
**Goal:** Split large workloads across towers

**Features:**
- Parallel prompt processing
- Load balancing
- Result aggregation
- Fault tolerance

### Phase 5: Cloud API Integration
**Goal:** Fallback to Anthropic/OpenAI

**Features:**
- API key management
- Cost tracking
- Rate limiting
- Automatic retry

---

## 🔬 Test Scenarios

### Scenario 1: Local Inference (Tower A)
```bash
# Submit simple prompt to Tower A Squirrel
curl -X POST http://192.168.1.144:9010/api/ai/complete \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "What is 2+2?",
    "model": "llama3-8b",
    "max_tokens": 50
  }'

Expected: Response in < 1 second
```

### Scenario 2: GPU Inference (Tower B)
```bash
# Submit complex prompt to Tower B Squirrel
curl -X POST http://192.168.1.134:9011/api/ai/complete \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Write a Python function to sort a list using quicksort",
    "model": "mistral-7b",
    "max_tokens": 500
  }'

Expected: Response in < 3 seconds
```

### Scenario 3: Distributed Batch
```bash
# Submit 100 prompts, distribute across towers
./squirrel_distributed_test.sh --prompts 100 --mode distributed

Expected:
- 50 prompts to Tower A (simpler)
- 50 prompts to Tower B (complex)
- Total time: < 15 seconds
- Speedup: ~6-7x vs sequential
```

### Scenario 4: Cloud Fallback
```bash
# Submit very complex prompt
curl -X POST http://192.168.1.144:8080/api/ai/complete \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Analyze this 50-page document and extract key insights...",
    "fallback": "anthropic",
    "max_tokens": 5000
  }'

Expected:
- Router detects complexity
- Automatically routes to Anthropic Claude
- Response in 30-60 seconds
- Cost tracked
```

---

## 📊 Expected Performance

### Throughput

| Workload | Sequential | Distributed (2 towers) | Speedup |
|----------|------------|------------------------|---------|
| **10 simple prompts** | 5s | 2.5s | 2x |
| **100 mixed prompts** | 120s | 18s | 6.7x |
| **1000 prompts** | 1200s | 180s | 6.7x |

### Latency (Token generation)

| Model | Tower | Current (1G) | Future (10G) | Improvement |
|-------|-------|--------------|--------------|-------------|
| **Llama 3 8B** | A | 500ms + 10-20ms | 500ms + 1-2ms | ~18ms faster |
| **Mistral 7B** | B | 200ms + 10-20ms | 200ms + 1-2ms | ~18ms faster |

**Note:** 10G NIC upgrade will reduce token-streaming latency significantly.

---

## 🛠️ Deployment Plan

### Build Squirrel
```bash
cd ../squirrel
cargo build --release

# Check binary size
ls -lh target/release/squirrel*
```

### Deploy to Tower A
```bash
cd ../songbird

./target/release/songbird-deploy deploy-http \
  --tower http://192.168.1.144:8080 \
  --binary ../squirrel/target/release/squirrel-server \
  --service squirrel-ai-tower-a \
  --env SQUIRREL_HOST=192.168.1.144 \
  --env SQUIRREL_PORT=9010 \
  --env SQUIRREL_MODEL=llama3-8b \
  --env SQUIRREL_DEVICE=cpu
```

### Deploy to Tower B
```bash
./target/release/songbird-deploy deploy-http \
  --tower http://192.168.1.134:8081 \
  --binary ../squirrel/target/release/squirrel-server \
  --service squirrel-ai-tower-b \
  --env SQUIRREL_HOST=192.168.1.134 \
  --env SQUIRREL_PORT=9011 \
  --env SQUIRREL_MODEL=mistral-7b \
  --env SQUIRREL_DEVICE=cuda \
  --env SQUIRREL_GPU_COUNT=8
```

---

## 🔌 API Integration

### Anthropic Claude
```rust
// In Squirrel router
async fn route_to_anthropic(prompt: &str) -> Result<String> {
    let client = anthropic::Client::new(api_key);
    let response = client.messages()
        .create()
        .model("claude-3-opus-20240229")
        .max_tokens(1000)
        .messages(vec![Message {
            role: Role::User,
            content: prompt.to_string(),
        }])
        .await?;
    
    Ok(response.content[0].text.clone())
}
```

### OpenAI GPT
```rust
async fn route_to_openai(prompt: &str) -> Result<String> {
    let client = openai::Client::new(api_key);
    let response = client.chat()
        .create()
        .model("gpt-4")
        .messages(vec![
            ChatMessage {
                role: "user",
                content: prompt.to_string(),
            }
        ])
        .await?;
    
    Ok(response.choices[0].message.content.clone())
}
```

---

## 🎯 Success Criteria

### Minimal Success ✅
- Squirrel deployed to both towers
- Basic inference working
- Request routing functional

### Full Success ✅
- All above
- Distributed inference (6-7x speedup)
- Cloud API fallback working
- < 1s latency for simple prompts
- < 3s latency for complex prompts

### Stretch Goals 🚀
- 10G NIC upgrade (1-2ms token latency)
- Larger models (Llama 3 70B)
- MCP protocol integration
- Distributed fine-tuning
- Cost optimization (prefer local over cloud)

---

## 💰 Cost Comparison

### Local vs Cloud (1000 prompts/day)

| Provider | Cost/1M tokens | 1000 prompts/day | Monthly cost |
|----------|----------------|------------------|--------------|
| **Local (Squirrel)** | $0 | $0 | **$0** ✅ |
| **Anthropic Claude** | $15 | ~$7.50 | **$225** |
| **OpenAI GPT-4** | $30 | ~$15 | **$450** |
| **OpenAI GPT-3.5** | $2 | ~$1 | **$30** |

**Savings with local-first:** $225-450/month!

**Smart routing (90% local, 10% cloud):**
- Local: 900 prompts = $0
- Cloud: 100 prompts = $22.50
- **Total:** $22.50/month (90% savings!)

---

## 📈 Roadmap

### Immediate (Today)
1. ✅ Check Squirrel availability
2. ✅ Create integration plan
3. 🚧 Build Squirrel binary
4. 🚧 Deploy to Tower A
5. 🚧 Deploy to Tower B
6. 🚧 Test basic inference

### Short-term (This Week)
- Request routing logic
- Distributed inference
- Performance benchmarking
- Cloud API integration

### Medium-term (This Month)
- 10G NIC upgrade
- Larger model support
- MCP protocol integration
- Production hardening

---

## 🎬 Demo Script

### Part 1: Local Inference (1 min)
```
"We have 2 towers running local AI models:
- Tower A: Llama 3 8B on CPU
- Tower B: Mistral 7B on GPU

Let's submit a simple prompt..."
[Submit] "What is the capital of France?"
[Result] Response in 500ms from Tower A ✅
```

### Part 2: GPU Acceleration (1 min)
```
"Now let's try something more complex..."
[Submit] "Write a quicksort implementation in Rust"
[Result] Response in 2s from Tower B GPU ✅
```

### Part 3: Distributed Batch (1 min)
```
"Now the magic: 100 prompts in parallel..."
[Submit] 100 mixed prompts
[Watch] Tasks split across both towers
[Result] All 100 complete in 18 seconds ✅
[Compare] Sequential would take 120 seconds
[Speedup] 6.7x faster! ✅
```

### Part 4: Cloud Fallback (30 sec)
```
"For very complex tasks, we route to cloud..."
[Submit] "Analyze this 50-page document"
[Watch] Router detects complexity
[Route] Automatically sends to Claude
[Result] Detailed analysis in 45 seconds ✅
```

---

## 🔮 Future Enhancements

1. **10G NIC Upgrade**
   - 10x lower latency (10-20ms → 1-2ms)
   - Enables real-time token streaming
   - Supports larger models

2. **Larger Models**
   - Llama 3 70B on Tower B (8x A6000)
   - GPT-J 6B as backup
   - Mixture-of-Experts models

3. **MCP Integration**
   - Model Context Protocol support
   - Tool calling
   - Multi-modal (text + images)

4. **Distributed Training**
   - Fine-tune models across towers
   - Federated learning
   - Privacy-preserving training

5. **Cost Optimization**
   - Track API costs
   - Optimize routing (prefer local)
   - Cache common responses

---

**Status:** Ready to implement distributed AI mesh!  
**Next:** Build and deploy Squirrel to both towers  
**Impact:** Local AI + Cloud APIs = Best of both worlds! 🐿️🚀

