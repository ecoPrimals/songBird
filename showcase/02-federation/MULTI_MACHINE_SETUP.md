# 🎵 Multi-Machine Federation Setup Guide

Complete guide for setting up Songbird federation across multiple physical machines (towers).

---

## 🎯 Quick Start

### On Tower 1 (Seed Node)

```bash
# Pull the latest code
cd ~/Development/ecoPrimals/songbird
git pull

# Build if needed
cargo build --release --bin songbird-orchestrator

# Start the seed tower
cd showcase/02-federation/scripts
./start-tower.sh
```

**Note the IP address shown!** You'll need this for Tower 2.

### On Tower 2 (Connecting Node)

```bash
# Pull the same code
cd ~/Development/ecoPrimals/songbird
git pull

# Build if needed
cargo build --release --bin songbird-orchestrator

# Connect to Tower 1 (replace IP with Tower 1's IP)
cd showcase/02-federation/scripts
SONGBIRD_PORT=8000 \
SONGBIRD_PEERS="192.168.1.144:8000" \
./start-tower.sh
```

### Verify the Mesh

From either machine:

```bash
# Check health
curl http://localhost:8000/health

# Check services (if endpoint available)
curl http://localhost:8000/api/v1/services
```

---

## 📋 Detailed Setup

### Prerequisites

1. **Network Connectivity**
   - Both machines on same network (or routable)
   - Ports accessible (default: 8000)
   - No restrictive firewalls blocking traffic

2. **Software Requirements**
   - Rust toolchain installed
   - Git for pulling code
   - `curl` for testing endpoints

3. **Repository Cloned**
   ```bash
   cd ~/Development/ecoPrimals
   git clone <your-repo-url> songbird
   # OR if already cloned:
   cd songbird && git pull
   ```

### Step-by-Step Setup

#### Tower 1: Seed Node

1. **Check Network Configuration**
   ```bash
   # Find your IP address
   ip addr show
   # OR
   ifconfig
   
   # Example output: 192.168.1.144
   ```

2. **Configure Firewall (if needed)**
   ```bash
   # Ubuntu/Debian with UFW
   sudo ufw allow 8000/tcp
   
   # CentOS/RHEL with firewalld
   sudo firewall-cmd --add-port=8000/tcp --permanent
   sudo firewall-cmd --reload
   ```

3. **Start Seed Tower**
   ```bash
   cd ~/Development/ecoPrimals/songbird/showcase/02-federation/scripts
   
   # Basic start (binds to 0.0.0.0:8000)
   ./start-tower.sh
   
   # OR with custom configuration
   SONGBIRD_PORT=8000 \
   SONGBIRD_NODE_ID="tower-main" \
   RUST_LOG=info \
   ./start-tower.sh
   ```

4. **Verify Running**
   ```bash
   # Check process
   ps aux | grep songbird-orchestrator
   
   # Check port
   lsof -i :8000
   
   # Test health endpoint
   curl http://localhost:8000/health
   ```

5. **Note Connection Details**
   ```
   Tower 1 IP: 192.168.1.144
   Tower 1 Port: 8000
   Connection String: 192.168.1.144:8000
   ```

#### Tower 2: Connecting Node

1. **Get Tower 1's IP**
   - From Tower 1 startup output
   - OR ping/ssh to Tower 1 and check

2. **Test Connectivity to Tower 1**
   ```bash
   # Ping test
   ping 192.168.1.144
   
   # Port test
   telnet 192.168.1.144 8000
   # OR
   nc -zv 192.168.1.144 8000
   
   # HTTP test
   curl http://192.168.1.144:8000/health
   ```

3. **Start Tower 2**
   ```bash
   cd ~/Development/ecoPrimals/songbird/showcase/02-federation/scripts
   
   SONGBIRD_PORT=8000 \
   SONGBIRD_NODE_ID="tower-secondary" \
   SONGBIRD_PEERS="192.168.1.144:8000" \
   ./start-tower.sh
   ```

4. **Verify Connection**
   ```bash
   # Check local health
   curl http://localhost:8000/health
   
   # Check logs for peer discovery
   tail -f ../logs/tower-secondary.log
   # Look for: "Connected to peer" or "Discovered peer"
   ```

#### Tower 3+: Additional Nodes

For additional nodes, connect to ANY existing tower:

```bash
# Can connect to Tower 1 OR Tower 2
SONGBIRD_PEERS="192.168.1.144:8000" ./start-tower.sh

# OR connect to Tower 2
SONGBIRD_PEERS="192.168.1.134:8000" ./start-tower.sh

# OR connect to multiple for redundancy
SONGBIRD_PEERS="192.168.1.144:8000,192.168.1.134:8000" ./start-tower.sh
```

---

## 🔧 Configuration Options

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_PORT` | `8000` | HTTP API port |
| `SONGBIRD_NODE_ID` | `songbird-$(hostname)` | Unique node identifier |
| `SONGBIRD_BIND` | `0.0.0.0` | Bind address (0.0.0.0 for all interfaces) |
| `SONGBIRD_FEDERATION` | `true` | Enable federation mode |
| `SONGBIRD_PEERS` | `` | Comma-separated seed peers (e.g., `host1:port1,host2:port2`) |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

### Example Configurations

**Development/Testing (Single Machine)**
```bash
# Tower A
SONGBIRD_PORT=8000 ./start-tower.sh

# Tower B
SONGBIRD_PORT=8001 SONGBIRD_PEERS="localhost:8000" ./start-tower.sh

# Tower C
SONGBIRD_PORT=8002 SONGBIRD_PEERS="localhost:8000" ./start-tower.sh
```

**Production (Multi-Machine)**
```bash
# Tower 1 (Data Center)
SONGBIRD_NODE_ID="dc-primary" \
SONGBIRD_PORT=8000 \
RUST_LOG=info \
./start-tower.sh

# Tower 2 (Edge Location)
SONGBIRD_NODE_ID="edge-us-west" \
SONGBIRD_PORT=8000 \
SONGBIRD_PEERS="dc-primary.internal:8000" \
RUST_LOG=info \
./start-tower.sh
```

---

## 🧪 Testing the Federation

### 1. Health Checks

```bash
# From Tower 1
curl http://localhost:8000/health

# From Tower 2
curl http://localhost:8000/health

# Cross-tower (from Tower 2 to Tower 1)
curl http://192.168.1.144:8000/health
```

### 2. Process Verification

```bash
# Check running processes
ps aux | grep songbird-orchestrator

# Check port listening
lsof -i :8000

# Check network connections
netstat -an | grep 8000
```

### 3. Log Inspection

```bash
# View logs
cd showcase/02-federation/logs
ls -la

# Tail specific tower
tail -f tower-main.log

# Search for federation events
grep -i "peer\|federation\|discover" tower-main.log
```

---

## 🔍 Troubleshooting

### Nodes Can't Discover Each Other

**Symptoms:**
- Tower 2 starts but doesn't connect to Tower 1
- No peer discovery messages in logs

**Solutions:**

1. **Check Network Connectivity**
   ```bash
   ping <tower1-ip>
   telnet <tower1-ip> 8000
   ```

2. **Check Firewall**
   ```bash
   # Check if port is accessible
   nmap -p 8000 <tower1-ip>
   
   # Temporarily disable firewall for testing
   sudo ufw disable  # Ubuntu
   sudo systemctl stop firewalld  # CentOS
   ```

3. **Verify SONGBIRD_PEERS**
   ```bash
   # Check environment variable is set
   echo $SONGBIRD_PEERS
   
   # Should be: <ip>:<port>
   # e.g., 192.168.1.144:8000
   ```

4. **Check Logs**
   ```bash
   tail -100 logs/tower-*.log | grep -i error
   ```

### Port Already in Use

**Symptoms:**
- "Address already in use" error
- Tower won't start

**Solutions:**

```bash
# Find process using the port
lsof -i :8000

# Kill the process
kill <PID>

# OR kill all songbird instances
killall songbird-orchestrator
```

### High Latency Between Towers

**Symptoms:**
- Slow responses
- Timeouts

**Solutions:**

```bash
# Check network latency
ping -c 10 <tower-ip>

# Check bandwidth
iperf3 -c <tower-ip>  # Server on Tower 1: iperf3 -s

# Adjust timeouts (if configuration supports it)
# Check logs for timeout messages
```

### Split Brain Scenario

**Symptoms:**
- Network partition causes towers to operate independently
- Inconsistent state after partition heals

**Solutions:**

1. **Prevention: Use Odd Number of Nodes**
   - 3 towers better than 2
   - 5 towers better than 4

2. **Detection: Monitor Connectivity**
   ```bash
   # Regularly check peer connections
   curl http://localhost:8000/api/v1/federation/peers
   ```

3. **Recovery: Restart Affected Nodes**
   ```bash
   # Restart one side of the partition
   killall songbird-orchestrator
   ./start-tower.sh
   ```

---

## 📊 Monitoring

### Health Monitoring Script

```bash
#!/bin/bash
# monitor-federation.sh

TOWERS=(
    "192.168.1.144:8000"
    "192.168.1.134:8000"
    "192.168.1.207:8000"
)

while true; do
    clear
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎵 Songbird Federation Monitor"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    date
    echo
    
    for tower in "${TOWERS[@]}"; do
        echo -n "Tower $tower: "
        if curl -s --max-time 2 "http://$tower/health" > /dev/null 2>&1; then
            health=$(curl -s "http://$tower/health")
            echo "✅ $health"
        else
            echo "❌ Offline"
        fi
    done
    
    echo
    echo "Refreshing in 5 seconds... (Ctrl+C to exit)"
    sleep 5
done
```

---

## 🚀 Next Steps

1. **Run Demo 2**: `cd demos && ./02-connect-to-remote.sh`
2. **Add More Towers**: Repeat Tower 2 setup for additional nodes
3. **Test Failover**: Kill one tower and observe redistribution
4. **Monitor Mesh**: Use monitoring scripts to track health

---

## 💡 Tips

- **Use Static IPs or DNS**: Avoid DHCP for production towers
- **Monitor Logs**: Set up log aggregation (e.g., ELK stack)
- **Health Checks**: Automate health monitoring
- **Backups**: Regular configuration backups
- **Security**: Use TLS for inter-tower communication (future enhancement)

---

**Ready to scale?** Start with 2-3 towers and expand from there! 🎵

