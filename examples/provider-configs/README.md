# Provider Configuration Examples

This directory contains example provider configurations for Songbird's universal HTTP gateway.

## Philosophy

**Zero Hardcoding**: These configurations define provider behavior without any code changes!

**Universal Design**: ONE proxy implementation works with ALL providers via configuration.

**Capability-Based**: Providers are discovered and routed by capability, not by name.

## Configuration Format

Each provider configuration defines:

1. **Identity**: `id` and `name` of the provider
2. **Capabilities**: What capabilities this provider offers
3. **Connection**: Socket path (for local) or backend config (for external)
4. **Transforms**: How to map generic requests/responses to provider-specific format
5. **Metadata**: Rate limits, caching, retries, etc.

## Examples

### External API Providers

- `openai.json` - OpenAI GPT-4 API
- `huggingface.json` - HuggingFace Inference API
- `anthropic.json` - Anthropic Claude API (example)

### Local Primal Providers

- `toadstool-local.json` - Local GPU compute provider

## Usage

### Option 1: Environment Variable

```bash
export SONGBIRD_PROVIDER_REGISTRY=/path/to/provider-registry.json
```

Where `provider-registry.json` contains:
```json
{
  "version": "1.0",
  "providers": [
    { ... provider config ... },
    { ... provider config ... }
  ]
}
```

### Option 2: Config Directory

```bash
export SONGBIRD_PROVIDER_CONFIG_DIR=/path/to/provider-configs/
```

Songbird will load all `*.json` files from this directory.

### Option 3: Runtime Registration

Providers can register themselves at runtime via IPC:

```rust
let provider_config = ProviderConfig { ... };
router.register_provider(provider_config).await?;
```

## Transform Mappings

Transform mappings allow you to map generic field names to provider-specific formats:

```json
{
  "request_transform": {
    "field_mappings": {
      "prompt": "inputs",              // Map "prompt" → "inputs"
      "max_tokens": "parameters.max_new_tokens"  // Nested fields supported
    }
  },
  "response_transform": {
    "field_mappings": {
      "[0].generated_text": "response"  // Array access supported
    }
  }
}
```

## Adding New Providers

To add a new provider:

1. **Create a JSON file** with provider configuration
2. **Define capabilities** the provider offers
3. **Configure transforms** for request/response mapping
4. **Set metadata** for rate limits, caching, etc.
5. **Place in config directory** or registry

**NO CODE CHANGES REQUIRED!** The universal proxy handles everything.

## Capability IDs

Capability IDs follow the format: `category:type` or `category:type:subtype`

Examples:
- `ai:text-generation` - Generic text generation
- `ai:text-generation:openai` - OpenAI-specific
- `ai:text-generation:local` - Local GPU
- `ai:image-generation` - Image generation
- `ai:embedding` - Embedding generation
- `storage:object-storage` - Object storage
- `compute:function-execution` - Function execution

## Metadata Fields

Common metadata fields:

- `rate_limit_requests_per_minute` - Rate limit quota
- `rate_limit_requests_per_day` - Daily quota
- `cache_enabled` - Enable caching
- `cache_ttl_seconds` - Cache TTL
- `retry_max_attempts` - Max retry attempts
- `retry_backoff` - Backoff strategy (exponential, linear)
- `timeout_seconds` - Request timeout
- `cost_per_1k_tokens` - Cost per 1K tokens
- `avg_latency_ms` - Average latency
- `quality_tier` - Quality tier (premium, standard, basic)

## Security

API keys are NEVER in configuration files!

Use environment variables:
```bash
export OPENAI_API_KEY=sk-...
export HUGGINGFACE_API_KEY=hf_...
```

The `api_key_env` field specifies which environment variable to read.

## Zero Hardcoding Principle

Notice what's NOT in the code:

❌ No "if provider == openai"  
❌ No hardcoded API endpoints  
❌ No vendor-specific handlers  
❌ No custom transformation logic  

Everything is configuration-driven! 🎉

## Example: Adding Anthropic

Create `anthropic.json`:

```json
{
  "id": "anthropic",
  "name": "Anthropic Claude",
  "capabilities": [
    {
      "id": "ai:text-generation:anthropic",
      "description": "Anthropic Claude AI",
      "category": "ai",
      "capability_type": "text-generation",
      "sub_type": "anthropic",
      "metadata": {}
    }
  ],
  "backend": {
    "base_url": "https://api.anthropic.com/v1/messages",
    "api_key_env": "ANTHROPIC_API_KEY",
    "request_transform": {
      "field_mappings": {
        "prompt": "messages[0].content",
        "max_tokens": "max_tokens"
      }
    },
    "response_transform": {
      "field_mappings": {
        "content[0].text": "response"
      }
    },
    "headers": {
      "anthropic-version": "2023-06-01",
      "Content-Type": "application/json"
    }
  },
  "metadata": {
    "rate_limit_requests_per_minute": 50
  }
}
```

That's it! No code changes needed.

## Testing

To test a provider configuration:

```bash
# Load config
export SONGBIRD_PROVIDER_CONFIG_DIR=./examples/provider-configs/

# Set API key
export OPENAI_API_KEY=sk-...

# Run Songbird
cargo run --bin songbird-orchestrator

# Test via Unix socket (from Squirrel or other primal)
echo '{"jsonrpc":"2.0","method":"proxy","params":{"capability":"ai:text-generation:openai","payload":{"prompt":"Hello"}},"id":1}' | nc -U /run/user/1000/songbird-ai.sock
```

## Philosophy Vindication

This is the **DEEP DEBT SOLUTION** approach:

Instead of writing separate handlers for each vendor:
- OpenAI handler (100 lines)
- HuggingFace handler (100 lines)  
- Anthropic handler (100 lines)
- = 300 lines of vendor-specific code

We have:
- ONE universal proxy (350 lines)
- Configuration files (provider-specific)
- = Works with INFINITE providers!

**Zero vendor hardcoding achieved!** 🎉

---

**See also**:
- `../http_gateway/capability_router.rs` - Capability routing implementation
- `../http_gateway/universal_proxy.rs` - Universal proxy implementation
- `../http_gateway/unix_listener.rs` - Unix socket listener implementation

