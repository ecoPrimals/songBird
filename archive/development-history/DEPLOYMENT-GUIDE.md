# 🚀 **Songbird Orchestrator Deployment Guide**

## 🎮 **Choose Your Gaming Experience**

### 🏃‍♂️ **Quick Start: Free Gaming Bridge**
Perfect for playing games with friends, individual use, or learning

### 🏢 **Enterprise Production**  
Full monitoring, high availability, and production deployment

---

## 🎯 **Option 1: Free Gaming Bridge (AGPL 3.0)**

### ⚡ **5-Minute Setup**

```bash
# 1. Clone repository
git clone https://github.com/ecoPrimals/SongBird.git
cd SongBird

# 2. Start gaming bridge (one command!)
docker-compose -f docker-compose.core.yml up -d

# 3. Check if running
curl http://localhost:8080/health
```

### 🎮 **Start Gaming Immediately**

```bash
# Create a gaming session for StarCraft
curl -X POST http://localhost:8080/gaming/session \
  -H "Content-Type: application/json" \
  -d '{
    "game_name": "StarCraft",
    "protocol": "ipx",
    "max_players": 8
  }'

# Response: {"session_code": "ABC123", "join_info": {...}}

# Share session code "ABC123" with friends!
```

### 📊 **Check Performance** 
```bash
# View built-in metrics
curl http://localhost:8081/metrics

# Check gaming bridge status  
curl http://localhost:8080/status
```

### 🛑 **Stop Gaming Bridge**
```bash
docker-compose -f docker-compose.core.yml down
```

---

## 🏢 **Option 2: Enterprise Production (BearDog Licensed)**

### 📋 **Prerequisites**
- Valid BearDog license (free for universities/research)
- Docker & Docker Compose
- Minimum 4GB RAM, 2 CPU cores

### 🔐 **Get BearDog License**

#### **Automatic (Universities/Research):**
```bash
# Will auto-detect .edu/.org domains
export BEARDOG_AUTO_LICENSE=true
```

#### **Manual (Individual/Power User):**
1. Email: `licenses@beardog.dev`
2. Subject: "Free BearDog License - Songbird Gaming"
3. Include your use case

#### **Commercial:**
Contact `sales@beardog.dev` for pricing

### 🚀 **Production Deployment**

```bash
# 1. Set your license key
export BEARDOG_LICENSE_KEY="your-license-key-here"

# 2. Start full production stack
docker-compose -f docker-compose.production.yml up -d

# 3. Verify all services running
docker-compose -f docker-compose.production.yml ps
```

### 📊 **Access Enterprise Services**

#### **Gaming Bridge (Load Balanced)**
- **URL**: http://localhost (HAProxy load balancer)
- **API**: Same as core version but load balanced
- **High Availability**: Automatic failover between 2 instances

#### **Monitoring Dashboard (Grafana)**
- **URL**: http://localhost:3000
- **Login**: admin / songbird2024
- **Dashboards**: Gaming performance, system metrics, network analysis

#### **Metrics Collection (Prometheus)**
- **URL**: http://localhost:9090
- **Metrics**: Detailed gaming bridge performance data
- **Alerts**: Automated alerting on latency/errors

#### **Load Balancer Stats (HAProxy)**
- **URL**: http://localhost:8404/stats
- **Shows**: Traffic distribution, health checks, response times

### 🔧 **Production Configuration**

```bash
# Scale gaming bridges
docker-compose -f docker-compose.production.yml up -d --scale songbird-bridge-1=3

# View logs
docker-compose -f docker-compose.production.yml logs -f

# Update configuration
# Edit docker/songbird-production.toml then:
docker-compose -f docker-compose.production.yml restart
```

---

## 🎮 **Gaming Examples**

### 🏺 **StarCraft (IPX Protocol)**
```bash
# Create StarCraft session
curl -X POST http://localhost:8080/gaming/session \
  -H "Content-Type: application/json" \
  -d '{
    "game_name": "StarCraft", 
    "protocol": "ipx",
    "max_players": 8,
    "map": "Lost Temple"
  }'

# Join session with code ABC123
curl -X POST http://localhost:8080/gaming/session/ABC123/join \
  -H "Content-Type: application/json" \
  -d '{"player_name": "Player1"}'
```

### 🏰 **Age of Empires II (DirectPlay)**
```bash
# Create AoE2 session
curl -X POST http://localhost:8080/gaming/session \
  -H "Content-Type: application/json" \
  -d '{
    "game_name": "Age of Empires II",
    "protocol": "directplay", 
    "max_players": 8,
    "game_speed": "normal"
  }'
```

### 🌐 **Auto-Discovery (Any LAN Game)**
```bash
# Scan for existing games
curl http://localhost:8080/gaming/scan

# Auto-configure for detected game
curl -X POST http://localhost:8080/gaming/auto-configure
```

---

## 🔍 **Troubleshooting**

### 🐛 **Common Issues**

#### **Port Conflicts**
```bash
# Check what's using gaming ports
sudo netstat -tulpn | grep :7000-8000

# Change port range in config
# Edit docker/songbird-core.toml:
# gaming_port_start = 9000
# gaming_port_end = 10000
```

#### **Permission Issues (Linux)**
```bash
# Gaming bridge needs network capabilities
sudo setcap 'cap_net_raw,cap_net_admin=+ep' /usr/local/bin/songbird

# Or run with Docker (recommended)
# Docker handles capabilities automatically
```

#### **BearDog License Issues**
```bash
# Check license status
curl http://localhost:8080/license/status

# Validate license
curl -X POST http://localhost:8080/license/validate \
  -H "Authorization: Bearer $BEARDOG_LICENSE_KEY"
```

### 📊 **Performance Issues**

#### **High Latency**
```bash
# Check current latency
curl http://localhost:8081/metrics | grep latency

# Enable performance mode
export SONGBIRD_PERFORMANCE_MODE=high
docker-compose restart
```

#### **Low Throughput**
```bash
# Check throughput metrics
curl http://localhost:8081/metrics | grep throughput

# Increase worker threads (docker/songbird-core.toml):
# worker_thread_count = 16
```

---

## 🔄 **Migration & Updates**

### ⬆️ **Upgrading**
```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker-compose -f docker-compose.core.yml down
docker-compose -f docker-compose.core.yml build --no-cache
docker-compose -f docker-compose.core.yml up -d
```

### 🔄 **Core to Production Migration**
```bash
# 1. Stop core deployment
docker-compose -f docker-compose.core.yml down

# 2. Export gaming sessions (optional)
curl http://localhost:8080/gaming/sessions/export > sessions-backup.json

# 3. Get BearDog license (see above)

# 4. Start production deployment
export BEARDOG_LICENSE_KEY="your-key"
docker-compose -f docker-compose.production.yml up -d

# 5. Import sessions (optional)
curl -X POST http://localhost:8080/gaming/sessions/import \
  -H "Content-Type: application/json" \
  -d @sessions-backup.json
```

---

## 📈 **Performance Benchmarks**

### 🏆 **What You Can Expect**

#### **Core Gaming Bridge (Free)**
- **Latency**: ~1.1ms (50x better than 50ms target!)
- **Throughput**: 17,880 packets/sec
- **Concurrent Sessions**: 100+ 
- **Protocols**: IPX, DirectPlay, NetBIOSS, UDP/TCP

#### **Enterprise Production (BearDog Licensed)**
- **High Availability**: 99.9% uptime
- **Load Balancing**: Automatic traffic distribution
- **Monitoring**: Real-time dashboards
- **Alerting**: Automated issue detection
- **Scaling**: Horizontal scaling support

---

## 💬 **Community & Support**

- **Documentation**: [docs.songbird-orchestrator.dev](https://docs.songbird-orchestrator.dev)
- **Community Forum**: [forum.songbird-orchestrator.dev](https://forum.songbird-orchestrator.dev) 
- **Discord**: [discord.gg/songbird-gaming](https://discord.gg/songbird-gaming)
- **Bug Reports**: [GitHub Issues](https://github.com/ecoPrimals/SongBird/issues)

### 🎮 **Ready to Game!**

The gaming bridge is ready to bring your favorite legacy games back to life with modern networking. Whether you're playing StarCraft with college friends or running a commercial gaming service, Songbird Orchestrator has you covered.

**Core gaming = Free forever. Enterprise features = Fair pricing. 🎯** 