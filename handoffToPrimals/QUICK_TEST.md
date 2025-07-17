# 🧪 **QUICK TEST GUIDE**

## Test the Dynamic Plugin System

### **1. Run the Main Demo**
```bash
# From the main songbird-orchestrator directory:
cargo run --example dynamic_composition_demo
```

**Expected Output**:
```
🧩 Dynamic Plugin Composition Demo
===================================
🎯 Goal: Show how services work together like Lego blocks
📋 No static TOML files required!

1️⃣  Plugin Registration Demo
─────────────────────────────
✅ Registered BearDog: beardog-encryption
✅ Registered Songbird: songbird-orchestrator  
✅ Registered Toadstool-1: toadstool-compute-1
✅ Registered Toadstool-2: toadstool-compute-2
✅ Registered Toadstool-3: toadstool-compute-3
📊 Total registered plugins: 8

2️⃣  BearDog + Songbird Auto-Composition
────────────────────────────────────────
🔍 Found 1 possible compositions
✅ Optimal composition: beardog-encryption + songbird-orchestrator
📊 Estimated performance: 1250 RPS, 45ms latency

3️⃣  Toadstool Chaining Demo (Toadstool on Toadstool)
─────────────────────────────────────────────────────
🍄 Found 3 Toadstool instances
🔗 Toadstool chain composition:
   📥 Input → toadstool-compute-1
   toadstool-compute-2 → (processing)
   toadstool-compute-3 → 📤 Output

✅ Dynamic composition demo completed successfully!
```

### **2. Test CLI Commands**
```bash
# List available plugins
cargo run --bin songbird -- compose list --detailed

# Discover compositions
cargo run --bin songbird -- compose discover --capabilities "encryption,compute"

# Show examples
cargo run --bin songbird -- compose examples
```

### **3. Test Individual Integrations**

#### **BearDog Integration Test**
```bash
cargo run --example beardog_integration_demo
```

Expected: Complete security integration with encryption, key management, and audit logging.

#### **NestGate Integration Test**  
```bash
cargo run --example nestgate_integration
```

Expected: Network service adapter with health checks and request handling.

### **4. Verify Zero-Config Gaming**
```bash
# This would be the end-user experience:
cargo run --bin songbird -- gaming quick-setup --game "starcraft"
```

Expected: Automatic discovery of BearDog + NestGate + Songbird for secure gaming tunnel.

---

## ✅ **Success Criteria**

- [ ] Dynamic composition demo runs without errors
- [ ] All plugins register successfully  
- [ ] Auto-composition finds optimal combinations
- [ ] Toadstool chaining works (multiple instances)
- [ ] CLI commands respond correctly
- [ ] Individual integration examples work
- [ ] Zero-configuration gaming setup functions

---

## 🐛 **Troubleshooting**

**Missing Dependencies**: 
```bash
cargo build --release
```

**Permission Issues**: 
```bash
sudo chown -R $USER:$USER target/
```

**Port Conflicts**:
Edit port numbers in examples if needed (default: 8080-8083).

---

**If all tests pass**: ✅ The dynamic plugin system is working correctly and ready for team integration! 