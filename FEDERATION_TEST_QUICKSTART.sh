
╔════════════════════════════════════════════════════════════════╗
║                                                                ║
║     🚀 READY FOR FEDERATION TESTING! 🚀                        ║
║                                                                ║
╚════════════════════════════════════════════════════════════════╝

✅ All changes are committed and pushed to GitHub!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 STEP-BY-STEP TESTING INSTRUCTIONS:

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  STEP 1: Start Eastgate (Bootstrap Node)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

On Eastgate (this machine):

# Terminal 1 - Keep this running
cd /home/eastgate/Development/ecoPrimals/songbird

export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_NODE_NAME=Eastgate
export SONGBIRD_PORT=8080

cargo run --release --bin songbird-orchestrator

Expected output:
  🚀 Starting Songbird Orchestrator
  🌐 Federation mode enabled
  🌐 Starting HTTP server on 0.0.0.0:8080
  ✅ HTTP server listening on 0.0.0.0:8080
  ✅ Songbird Orchestrator started successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  STEP 2: Pull Code on Strandgate
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

SSH to Strandgate, then:

cd ~/Development/ecoPrimals/songbird
git pull origin type-unification-capability

# Build (may take a few minutes)
cargo build --release --bin songbird-orchestrator

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  STEP 3: Start Strandgate (Joining Node)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Still on Strandgate:

# Terminal - Keep this running
export SONGBIRD_FEDERATION_ENABLED=true
export SONGBIRD_NODE_NAME=Strandgate
export SONGBIRD_PORT=8080
export SONGBIRD_BOOTSTRAP_ADDRESS=192.168.1.144:8080

cargo run --release --bin songbird-orchestrator

Expected output:
  🚀 Starting Songbird Orchestrator
  🌐 Federation mode enabled
  🔗 Will join federation via bootstrap: 192.168.1.144:8080
  🌐 Starting federation coordinator...
  🤝 Joining federation via bootstrap: 192.168.1.144:8080
  ✅ Joined federation successfully
  ✅ Federation coordinator started successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  STEP 4: Verify Federation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

From any machine on your network:

# Check federation status
curl http://192.168.1.144:8080/api/federation/status | jq

Expected: You should see BOTH Eastgate and Strandgate!

# Pretty format
curl -s http://192.168.1.144:8080/api/federation/status | jq '.nodes[] | {name: .node_name, status: .status, cores: .cpu_cores, memory: .memory_gb}'

# List just the nodes
curl -s http://192.168.1.144:8080/api/federation/nodes | jq '.[].node_name'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  STEP 5: Watch Heartbeats
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

In the terminal logs on both Eastgate and Strandgate, you should see:

Every 30 seconds:
  📡 Sending heartbeat to node: <other-node-name>

Every 60 seconds:
  🔍 Health monitoring check completed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  BONUS: Test Failure Detection
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. With both nodes running, check status (both should be 'active')
2. Press Ctrl+C on Strandgate to stop it
3. Wait 60 seconds
4. Check status again from Eastgate
5. Strandgate should now show status: 'inactive'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🐛 TROUBLESHOOTING:

If Strandgate can't connect:
  1. Check firewall: sudo ufw status
  2. Verify ping: ping 192.168.1.144
  3. Test port: nc -zv 192.168.1.144 8080
  4. Check Eastgate logs for connection attempts

If build fails on Strandgate:
  cargo clean
  cargo build --release --bin songbird-orchestrator

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📚 Full testing guide: PHASE_1A_TEST_GUIDE.md

Let me know what you see! 🚀

