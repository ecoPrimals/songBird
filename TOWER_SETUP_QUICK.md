# 🏗️ Quick Tower Setup Guide
**For Multi-Tower LAN Testing**

---

## 🚀 TOWER A (This Machine)

Already setup! Just need to run:

```bash
cd /home/eastgate/Development/ecoPrimals/songbird

# Setup environment
export SERVICE_ID=tower-a-orchestrator
export SERVICE_PORT=8080
export SONGBIRD_HOST=$(hostname -I | awk '{print $1}')  # Auto-detect LAN IP

# Run orchestrator
./target/release/songbird-orchestrator

# Expected: Starts on your LAN IP:8080
```

---

## 🏗️ TOWER B (Other Machine)

On your other tower:

```bash
# 1. Clone from GitHub
cd ~
git clone https://github.com/ecoPrimals/songbird
cd songbird

# 2. Build
cargo build --workspace --release
# Expected: ~25s build time

# 3. Setup environment
export SERVICE_ID=tower-b-compute
export SERVICE_PORT=8081
export SONGBIRD_HOST=$(hostname -I | awk '{print $1}')  # Auto-detect LAN IP
export SONGBIRD_FEDERATION_PEERS="http://<TOWER_A_IP>:8080"

# 4. Run (if you have toadstool)
./target/release/toadstool-server

# OR run in worker mode
./target/release/songbird-orchestrator --mode=worker
```

---

## 📋 QUICK CHECKLIST

### Before Starting:
- [ ] Both towers on same network
- [ ] Can ping each other
- [ ] Firewall allows ports 8080-8090
- [ ] Git credentials setup on Tower B

### Tower A:
- [ ] Songbird built
- [ ] SERVICE_PORT=8080 set
- [ ] Running orchestrator

### Tower B:
- [ ] Git clone complete
- [ ] Cargo build complete
- [ ] SERVICE_PORT=8081 set
- [ ] FEDERATION_PEERS points to Tower A
- [ ] Running compute/worker

### Verification:
```bash
# From Tower A:
curl http://<TOWER_A_IP>:8080/health  # Should return healthy
curl http://<TOWER_B_IP>:8081/health  # Should return healthy

# Check discovery
curl http://<TOWER_A_IP>:8080/discovery/peers
# Should list Tower B
```

---

## 🎯 EXPECTED RESULTS

- ✅ Both towers discover each other (<5s)
- ✅ Health checks pass
- ✅ Task distribution works
- ✅ Sub-millisecond orchestration overhead

---

**Ready for real distributed orchestration testing!** 🚀

