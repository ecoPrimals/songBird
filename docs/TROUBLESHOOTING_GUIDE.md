# 🔧 **SONGBIRD TROUBLESHOOTING GUIDE**

**Version**: 0.1.0  
**Status**: Production Ready  
**Last Updated**: January 2025  

---

## 🚨 **EMERGENCY QUICK FIXES**

### **System Won't Start**
```bash
# 1. Check if already running
ps aux | grep songbird

# 2. Check port conflicts
netstat -tulpn | grep :8080

# 3. Restart with clean slate
pkill -f songbird-orchestrator
./target/release/songbird-orchestrator

# 4. Check logs for startup errors
tail -f /var/log/songbird/songbird.log
```

### **Health Check Failing**
```bash
# Quick health diagnostics
curl -v http://localhost:8080/api/health 2>&1 | head -20

# If connection refused:
systemctl status songbird          # For systemd deployments
./target/release/songbird-orchestrator --version

# Check configuration
./target/release/songbird-orchestrator --validate-config
```

### **Gaming Not Working**
```bash
# Reset gaming configuration
curl -X POST http://localhost:8080/api/gaming/reset

# Retry setup
curl -X POST http://localhost:8080/api/gaming/setup -d '{"setup_type":"one_touch"}'

# Check status
curl http://localhost:8080/api/gaming/status
```

---

## 🔍 **DIAGNOSTIC TOOLS**

### **System Status Check**
```bash
#!/bin/bash
# system-diagnostics.sh - Complete system diagnosis

echo "🔍 Songbird System Diagnostics"
echo "================================"

# Process Check
echo "1. Process Status:"
if pgrep -f songbird-orchestrator > /dev/null; then
    echo "✅ Songbird orchestrator is running"
else
    echo "❌ Songbird orchestrator not found"
fi

# Port Check
echo "2. Port Availability:"
for port in 8080 8443 9090; do
    if netstat -tulpn | grep ":$port " > /dev/null; then
        echo "✅ Port $port is in use"
    else
        echo "⚠️ Port $port is free"
    fi
done

# API Health
echo "3. API Health:"
if curl -f http://localhost:8080/api/health > /dev/null 2>&1; then
    echo "✅ API responding"
else
    echo "❌ API not responding"
fi

# System Resources
echo "4. System Resources:"
echo "   CPU: $(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1"%"}')"
echo "   Memory: $(free | grep Mem | awk '{printf "%.1f%%", $3/$2 * 100.0}')"
echo "   Disk: $(df / | grep / | awk '{print $5}')"

# Configuration
echo "5. Configuration:"
if [ -f "songbird.toml" ]; then
    echo "✅ Configuration file found"
else
    echo "⚠️ No configuration file in current directory"
fi

echo "================================"
```

### **Network Connectivity Test**
```bash
#!/bin/bash
# network-diagnostics.sh - Network troubleshooting

echo "🌐 Network Diagnostics"

# Basic connectivity
echo "1. Basic Connectivity:"
ping -c 3 127.0.0.1 > /dev/null && echo "✅ Localhost reachable" || echo "❌ Localhost unreachable"
ping -c 3 8.8.8.8 > /dev/null && echo "✅ Internet accessible" || echo "❌ Internet unreachable"

# Port accessibility
echo "2. Port Accessibility:"
for port in 8080 8443; do
    if nc -z localhost $port 2>/dev/null; then
        echo "✅ Port $port accessible"
    else
        echo "❌ Port $port not accessible"
    fi
done

# Firewall status
echo "3. Firewall Status:"
if command -v ufw > /dev/null; then
    ufw status | head -5
elif command -v iptables > /dev/null; then
    iptables -L INPUT | head -5
fi
```

---

## 🎮 **GAMING ISSUES**

### **Problem: Gaming Setup Fails**
**Symptoms**: Gaming setup returns `success: false` or errors

**Solutions**:
```bash
# 1. Check gaming system status
curl http://localhost:8080/api/gaming/status

# 2. Verify no port conflicts
netstat -tulpn | grep -E ':2300|:2350|:6112'

# 3. Test with minimal configuration
curl -X POST http://localhost:8080/api/gaming/setup \
  -d '{"setup_type": "minimal", "debug": true}'

# 4. Check for missing dependencies
ldd target/release/songbird-orchestrator | grep "not found"

# 5. Reset and try again
curl -X POST http://localhost:8080/api/gaming/reset
curl -X POST http://localhost:8080/api/gaming/setup -d '{"setup_type":"one_touch"}'
```

### **Problem: Legacy Games Won't Connect**
**Symptoms**: Old games show "IPX not available" or networking errors

**Solutions**:
```bash
# 1. Enable legacy protocol support
curl -X POST http://localhost:8080/api/gaming/protocols/enable \
  -d '{"protocols": ["ipx", "directplay"]}'

# 2. Test protocol translation
curl -X POST http://localhost:8080/api/gaming/protocols/test \
  -d '{"protocol": "ipx", "test_target": "127.0.0.1"}'

# 3. Check protocol status
curl http://localhost:8080/api/gaming/protocols/status

# 4. Verify game-specific configuration
curl -X POST http://localhost:8080/api/gaming/configure \
  -d '{"game_name": "StarCraft", "force_ipx": true}'

# 5. Debug protocol translation
curl http://localhost:8080/api/gaming/debug/protocols
```

### **Problem: High Gaming Latency**
**Symptoms**: Games are laggy despite good internet

**Solutions**:
```bash
# 1. Enable latency optimization
curl -X PUT http://localhost:8080/api/gaming/optimization \
  -d '{"latency_priority": true, "buffer_optimization": "minimal"}'

# 2. Check current latency
curl http://localhost:8080/api/gaming/performance/metrics | jq '.latency'

# 3. Test direct connection mode
curl -X PUT http://localhost:8080/api/gaming/mode \
  -d '{"connection_mode": "direct", "bypass_proxy": true}'

# 4. Network analysis
curl http://localhost:8080/api/gaming/network/analysis

# 5. System resource check
curl http://localhost:8080/api/metrics | jq '.system_metrics'
```

### **Problem: Multiple Players Can't Join**
**Symptoms**: Only one player can connect to multiplayer games

**Solutions**:
```bash
# 1. Check NAT traversal
curl http://localhost:8080/api/gaming/network/nat-status

# 2. Enable UPnP if available
curl -X PUT http://localhost:8080/api/gaming/network \
  -d '{"upnp_enabled": true, "port_forwarding": "auto"}'

# 3. Check firewall rules
sudo iptables -L | grep -E '2300|2350|6112'

# 4. Use Songbird as game host
curl -X POST http://localhost:8080/api/gaming/server/start \
  -d '{"game": "StarCraft", "max_players": 8}'

# 5. Manual port forwarding (router configuration needed)
curl -X POST http://localhost:8080/api/gaming/network/manual-ports \
  -d '{"tcp_ports": [2300, 2350], "udp_ports": [2300, 2350]}'
```

---

## 🌐 **FEDERATION ISSUES**

### **Problem: Can't Join Federation**
**Symptoms**: Federation join fails with connection errors

**Solutions**:
```bash
# 1. Test network connectivity to federation nodes
curl -v http://remote-node:8080/api/health

# 2. Check local federation configuration
curl http://localhost:8080/api/federation/config

# 3. Verify certificates (if using HTTPS)
openssl s_client -connect remote-node:8443 -servername remote-node

# 4. Test with simplified configuration
curl -X POST http://localhost:8080/api/federation/join \
  -d '{
    "cluster_id": "test-cluster",
    "node_id": "debug-node", 
    "cluster_endpoints": ["http://localhost:8080"],
    "timeout": 30,
    "debug": true
  }'

# 5. Check DNS resolution
nslookup remote-node.example.com
```

### **Problem: Federation Nodes Keep Disconnecting**
**Symptoms**: Nodes show as offline intermittently

**Solutions**:
```bash
# 1. Check heartbeat configuration
curl http://localhost:8080/api/federation/heartbeat/config

# 2. Increase heartbeat interval
curl -X PUT http://localhost:8080/api/federation/config \
  -d '{"heartbeat_interval": 60, "timeout": 30}'

# 3. Monitor network stability
ping -i 10 -c 100 remote-node.example.com

# 4. Check system resources
curl http://localhost:8080/api/metrics | jq '.system_metrics'

# 5. Review federation logs
journalctl -u songbird | grep -i federation | tail -50
```

---

## 🔍 **PRIMAL INTEGRATION ISSUES**

### **Problem: No Primals Discovered**
**Symptoms**: Primal discovery returns empty results

**Solutions**:
```bash
# 1. Force discovery refresh
curl -X POST http://localhost:8080/api/primals/discover/refresh

# 2. Check network discovery methods
curl http://localhost:8080/api/primals/discovery/methods

# 3. Manually register a test primal
curl -X POST http://localhost:8080/api/primals/register \
  -d '{
    "primal_type": "test-primal",
    "display_name": "Test Primal",
    "endpoint": {"primary_url": "http://localhost:8443"}
  }'

# 4. Verify mDNS/discovery protocols
avahi-browse -at | grep songbird  # Linux
dns-sd -B _http._tcp              # macOS

# 5. Check development/mock mode
curl http://localhost:8080/api/primals/config | jq '.mock_mode'
```

### **Problem: Primal Health Check Failing**
**Symptoms**: Discovered primals show as unhealthy

**Solutions**:
```bash
# 1. Manual health check
curl http://localhost:8080/api/primals/health/force-check

# 2. Test primal endpoint directly
curl -v http://primal-endpoint:8080/health

# 3. Check primal configuration
curl http://localhost:8080/api/primals/list | jq '.[] | select(.primal_type == "failing-primal")'

# 4. Adjust health check parameters
curl -X PUT http://localhost:8080/api/primals/health/config \
  -d '{"timeout": 10, "retry_count": 3, "interval": 30}'

# 5. Debug primal communication
curl http://localhost:8080/api/primals/debug/communication
```

---

## 🤖 **AI & SERVICE MESH ISSUES**

### **Problem: Workload Classification Fails**
**Symptoms**: AI classification returns errors or low confidence

**Solutions**:
```bash
# 1. Test with simple workload
curl -X POST http://localhost:8080/api/ai/classify \
  -d '{
    "workload_id": "simple-test",
    "characteristics": ["web_service"],
    "resource_requirements": {"cpu_cores": 1, "memory_gb": 1}
  }'

# 2. Check AI system status
curl http://localhost:8080/api/ai/status

# 3. Verify input format
curl -X POST http://localhost:8080/api/ai/validate \
  -d '{"workload_data": "your_workload_request_here"}'

# 4. Check system resources for AI processing
curl http://localhost:8080/api/metrics | jq '.system_metrics'

# 5. Reset AI classification engine
curl -X POST http://localhost:8080/api/ai/reset
```

### **Problem: AI Provides Poor Recommendations**
**Symptoms**: AI confidence scores are consistently low

**Solutions**:
```bash
# 1. Provide more detailed workload characteristics
curl -X POST http://localhost:8080/api/ai/classify \
  -d '{
    "workload_id": "detailed-test",
    "characteristics": ["web_service", "high_throughput", "database_backend"],
    "resource_requirements": {"cpu_cores": 4, "memory_gb": 8},
    "historical_data": {"avg_requests_per_second": 1000}
  }'

# 2. Check AI training data status
curl http://localhost:8080/api/ai/training/status

# 3. Review AI decision rationale
curl -X POST http://localhost:8080/api/ai/explain \
  -d '{"workload_id": "test-workload"}'

# 4. Update AI model (if applicable)
curl -X POST http://localhost:8080/api/ai/model/update
```

---

## 🔒 **SECURITY ISSUES**

### **Problem: Authentication Failures**
**Symptoms**: Security authentication requests fail

**Solutions**:
```bash
# 1. Check security primal status
curl http://localhost:8080/api/primals/list | jq '.[] | select(.capabilities[] | contains("security"))'

# 2. Test security integration
curl -X POST http://localhost:8080/api/security/test \
  -d '{"test_type": "basic_auth"}'

# 3. Verify security configuration
curl http://localhost:8080/api/security/config

# 4. Check certificate validity
openssl x509 -in /path/to/cert.pem -text -noout

# 5. Test without authentication (development mode)
curl -X PUT http://localhost:8080/api/security/mode \
  -d '{"development_mode": true}'
```

### **Problem: Family Safety Features Not Working**
**Symptoms**: Content filtering or safety checks fail

**Solutions**:
```bash
# 1. Check family safety configuration
curl http://localhost:8080/api/gaming/safety/config

# 2. Test family safety validation
curl -X POST http://localhost:8080/api/gaming/safety/test \
  -d '{"content": "test content", "family_mode": true}'

# 3. Verify safety primal connection
curl http://localhost:8080/api/primals/health | jq '.[] | select(.capabilities[] | contains("safety"))'

# 4. Reset family safety settings
curl -X POST http://localhost:8080/api/gaming/safety/reset

# 5. Enable strict safety mode
curl -X PUT http://localhost:8080/api/gaming/safety \
  -d '{"strict_mode": true, "content_filtering": "maximum"}'
```

---

## 📊 **PERFORMANCE ISSUES**

### **Problem: High CPU/Memory Usage**
**Symptoms**: System resources constantly high

**Solutions**:
```bash
# 1. Check current resource usage
curl http://localhost:8080/api/metrics | jq '.system_metrics'

# 2. Identify resource-intensive components
curl http://localhost:8080/api/debug/profiling

# 3. Reduce active services
curl -X PUT http://localhost:8080/api/system/optimization \
  -d '{"profile": "resource_conservative"}'

# 4. Check for memory leaks
curl http://localhost:8080/api/debug/memory

# 5. Restart with clean state
systemctl restart songbird
```

### **Problem: Slow API Response Times**
**Symptoms**: API endpoints taking >500ms to respond

**Solutions**:
```bash
# 1. Performance diagnostics
time curl http://localhost:8080/api/health

# 2. Check system load
curl http://localhost:8080/api/metrics | jq '.system_metrics.cpu_usage'

# 3. Enable performance optimization
curl -X PUT http://localhost:8080/api/system/performance \
  -d '{"optimization_level": "maximum"}'

# 4. Check for blocking operations
curl http://localhost:8080/api/debug/blocking-operations

# 5. Reduce concurrent operations
curl -X PUT http://localhost:8080/api/system/limits \
  -d '{"max_concurrent_requests": 50}'
```

---

## 🗂️ **CONFIGURATION ISSUES**

### **Problem: Configuration Not Loading**
**Symptoms**: Settings not applied or default values used

**Solutions**:
```bash
# 1. Validate configuration file
./target/release/songbird-orchestrator --validate-config --config=songbird.toml

# 2. Check configuration file permissions
ls -la songbird.toml

# 3. Test with minimal configuration
cat > minimal-songbird.toml << EOF
[primal_registry]
auto_discovery = true

[gaming]
family_safe_mode = false
EOF

# 4. Verify environment variables
env | grep SONGBIRD

# 5. Check configuration precedence
./target/release/songbird-orchestrator --show-config
```

### **Problem: Environment Variables Not Working**
**Symptoms**: Environment variable overrides ignored

**Solutions**:
```bash
# 1. Check environment variable format
echo $SONGBIRD_CONFIG
echo $NODE_ID

# 2. Test variable substitution
SONGBIRD_CONFIG=./test.toml ./target/release/songbird-orchestrator --version

# 3. Verify variable names
grep -r "NODE_ID" crates/songbird-*/src/ | head -5

# 4. Use configuration file instead
cat > config-with-overrides.toml << EOF
node_id = "${NODE_ID:-default-node}"
EOF

# 5. Debug configuration loading
RUST_LOG=debug ./target/release/songbird-orchestrator 2>&1 | grep -i config
```

---

## 🚨 **EMERGENCY RECOVERY**

### **Complete System Reset**
```bash
#!/bin/bash
# emergency-reset.sh - Nuclear option for system recovery

echo "🚨 Emergency Songbird Reset"
echo "This will reset ALL configuration and restart services"
read -p "Are you sure? (yes/no): " confirm

if [ "$confirm" = "yes" ]; then
    echo "1. Stopping all Songbird processes..."
    pkill -f songbird
    
    echo "2. Clearing temporary data..."
    rm -rf /tmp/songbird-*
    
    echo "3. Resetting configuration..."
    cp songbird.toml songbird.toml.backup.$(date +%s)
    
    echo "4. Clearing logs..."
    > /var/log/songbird/songbird.log 2>/dev/null || true
    
    echo "5. Restarting Songbird..."
    ./target/release/songbird-orchestrator &
    
    echo "6. Waiting for startup..."
    sleep 10
    
    echo "7. Testing system health..."
    curl -f http://localhost:8080/api/health && echo "✅ System recovered" || echo "❌ Recovery failed"
fi
```

### **Data Backup Before Recovery**
```bash
#!/bin/bash
# backup-before-recovery.sh

BACKUP_DIR="./songbird-backup-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "📦 Creating backup in $BACKUP_DIR"

# Configuration files
cp -r *.toml "$BACKUP_DIR/" 2>/dev/null || true

# Logs
cp -r /var/log/songbird "$BACKUP_DIR/" 2>/dev/null || true

# Runtime data
curl -s http://localhost:8080/api/metrics > "$BACKUP_DIR/last-metrics.json"
curl -s http://localhost:8080/api/federation/status > "$BACKUP_DIR/federation-status.json"
curl -s http://localhost:8080/api/gaming/status > "$BACKUP_DIR/gaming-status.json"

echo "✅ Backup complete in $BACKUP_DIR"
```

---

## 📋 **TROUBLESHOOTING CHECKLIST**

### **Before Seeking Help** ✅
- [ ] Checked system logs: `journalctl -u songbird -n 50`
- [ ] Ran health check: `curl http://localhost:8080/api/health`
- [ ] Verified system resources: CPU, memory, disk space
- [ ] Tested with minimal configuration
- [ ] Checked for port conflicts
- [ ] Reviewed recent changes to configuration

### **Information to Collect** 📝
- [ ] Songbird version: `./target/release/songbird-orchestrator --version`
- [ ] Operating system and version
- [ ] Configuration file contents (redacted sensitive info)
- [ ] Error messages and logs
- [ ] Steps to reproduce the issue
- [ ] Network configuration details

### **Common Resolution Steps** 🔧
1. **Restart Service**: Often resolves transient issues
2. **Check Configuration**: Validate syntax and values
3. **Network Testing**: Verify connectivity and ports
4. **Resource Check**: Ensure adequate CPU/memory
5. **Log Analysis**: Review recent log entries
6. **Clean Restart**: Stop, clean, restart
7. **Minimal Config**: Test with basic configuration

---

## 🆘 **WHEN TO SEEK HELP**

### **Community Support**
- GitHub Issues: For bugs and feature requests
- Community Forums: For usage questions
- Discord/Chat: For real-time help

### **Escalation Criteria**
- Security vulnerabilities
- Data loss or corruption
- System crashes or hangs
- Performance degradation
- Configuration corruption

---

## 🎯 **TROUBLESHOOTING QUICK REFERENCE**

### **Common Commands**
```bash
# Health check
curl http://localhost:8080/api/health

# System metrics
curl http://localhost:8080/api/metrics | jq '.'

# Gaming status
curl http://localhost:8080/api/gaming/status

# Federation status
curl http://localhost:8080/api/federation/status

# Restart service
systemctl restart songbird

# View logs
journalctl -u songbird -f

# Configuration validation
./target/release/songbird-orchestrator --validate-config

# Emergency reset
pkill -f songbird && ./target/release/songbird-orchestrator
```

### **Log Locations**
- **Systemd**: `journalctl -u songbird`
- **File logs**: `/var/log/songbird/`
- **Application logs**: Current directory (development)
- **Debug logs**: Enable with `RUST_LOG=debug`

---

**🔧 Most issues can be resolved with the solutions above. If problems persist, your Songbird system may need more specialized attention.**

**Next Steps**:
- [API Reference](API_REFERENCE.md) - Detailed API documentation
- [Deployment Guide](DEPLOYMENT_GUIDE.md) - Deployment troubleshooting
- [Gaming Setup Guide](GAMING_SETUP_GUIDE.md) - Gaming-specific issues 