# 🍄 ToadStool Quick Start - UNBLOCKED! ✅

## 🎉 TLS Issue: RESOLVED!

**Good News**: Songbird TLS is already configured with `rustls` + `ring` crypto provider!

**Your V2 distributed ML is ready to go!**

---

## 🚀 Quick Start (3 Steps)

### Step 1: Start Songbird on Each Tower

**Tower A (Eastgate)**:
```bash
cd ~/Development/ecoPrimals/songbird
export SONGBIRD_TLS_ENABLED=false  # Easy dev mode
cargo run --release --bin songbird-orchestrator
```

**Tower B (Strandgate)**:
```bash
cd ~/Development/ecoPrimals/songbird
export SONGBIRD_TLS_ENABLED=false  # Easy dev mode
cargo run --release --bin songbird-orchestrator
```

### Step 2: Test Connectivity

```bash
curl http://192.168.1.134:8081/health  # Tower A
curl http://192.168.1.135:8081/health  # Tower B
```

### Step 3: Submit Tasks from ToadStool

```bash
# Submit to Tower A
curl -X POST http://192.168.1.134:8081/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{"task": {"name": "ml_train_a", "gpu": true}, "priority": 8}'

# Submit to Tower B  
curl -X POST http://192.168.1.135:8081/api/compute/task \
  -H "Content-Type: application/json" \
  -d '{"task": {"name": "ml_train_b", "gpu": true}, "priority": 8}'
```

---

## 🎯 Why HTTP Mode?

**Fail-Safe Philosophy**:
- ✅ TLS **enabled by default** (production-safe)
- ✅ Easy opt-out for development (`SONGBIRD_TLS_ENABLED=false`)
- ✅ No certificate hassles during development
- ✅ Enable TLS later for production

**You're in control**: Secure by default, flexible for development!

---

## 📚 Full Documentation

- **[ToadStool Integration Guide](./docs/integrations/TOADSTOOL_INTEGRATION.md)** - Complete guide
- **[TLS Configuration](./docs/operations/TLS_CONFIGURATION.md)** - TLS details
- **[API Reference](./docs/api/REST_API.md)** - API docs

---

## 🔧 Helper Scripts

We created helper scripts for you:

```bash
# Quick start with automatic configuration
./scripts/toadstool-quick-start.sh eastgate    # Tower A
./scripts/toadstool-quick-start.sh strandgate  # Tower B

# Generic HTTP mode
./scripts/start-local-http.sh
```

---

## 📊 What You Can Do Now

✅ **V2-Lite**: Direct API calls to both towers  
✅ **Real GPUs**: RTX 2070 + RTX 3070  
✅ **Real Network**: Measure actual latency  
✅ **Prove Pattern**: Validate distributed ML works  

**Upgrade to full federation later** when you need auto-discovery!

---

## 🎓 Migration Path

1. **Now**: HTTP mode, direct API calls → **Prove it works!**
2. **Later**: Enable TLS with shared cert → **Add encryption**
3. **Production**: Full federation + real certs → **Scale it up**

---

## 🐛 Troubleshooting

### "Connection refused"
```bash
# Check Songbird is running
ps aux | grep songbird

# Check firewall
sudo ufw allow 8081
```

### "Want TLS but getting errors"
```bash
# Songbird auto-generates self-signed certs!
# Just run without SONGBIRD_TLS_ENABLED=false
cargo run --release --bin songbird-orchestrator
```

---

## 🎉 You're Unblocked!

**Status**: ✅ Ready to implement V2  
**TLS**: ✅ Already configured  
**Scripts**: ✅ Ready to use  
**Docs**: ✅ Complete  

**Go build your distributed ML system!** 🚀

---

**Questions?** See [docs/integrations/TOADSTOOL_INTEGRATION.md](./docs/integrations/TOADSTOOL_INTEGRATION.md)

