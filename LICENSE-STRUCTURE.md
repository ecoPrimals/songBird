# 📜 **Songbird Orchestrator Licensing Structure**

## 🦀 **Core Gaming Bridge: AGPL 3.0 (FREE FOREVER)**

The heart of Songbird Orchestrator - the gaming network bridge functionality - is **100% AGPL 3.0 licensed** and will remain **free forever**.

### ✅ **What's FREE under AGPL 3.0:**
- **🎮 Gaming Protocol Detection**: IPX, DirectPlay, NetBIOS, UDP/TCP
- **🌉 Network Bridging**: Universal game protocol translation
- **🔄 NAT Traversal**: UPnP, STUN, TURN support
- **⚡ Performance Optimization**: <50ms latency (achieved 1.1ms!)
- **🔗 Session Management**: LAN gaming session creation/joining
- **📊 Built-in Metrics**: Basic performance monitoring
- **🔧 HTTP API**: Control and status endpoints
- **🐳 Docker Support**: Core containerized deployment

### 📦 **Free Deployment:**
```bash
# Use the core AGPL 3.0 gaming bridge
docker-compose -f docker-compose.core.yml up
```

---

## 🔐 **Biome Edge Services: BearDog Crypto-Locked**

External monitoring, production deployment, and enterprise features are **crypto-locked** and require **BearDog signed licenses** for commercial use.

### 🏢 **What Requires BearDog Signed License:**
- **📊 Grafana Dashboards**: Advanced monitoring visualization
- **📈 Prometheus Metrics**: Enterprise metrics collection
- **⚖️ HAProxy Load Balancer**: High availability and load distribution
- **💾 Redis Caching**: Session storage and caching
- **📝 Fluentd Logging**: Centralized log aggregation
- **🏭 Production Pipeline**: Enterprise deployment automation

### 💰 **BearDog Signed License Costs:**
- **🎓 FREE**: Universities, research institutions, educational use
- **👨‍💻 FREE**: Individual developers, power users, open source projects
- **💼 PAID**: Commercial companies and organizations

### 🔒 **Enterprise Deployment:**
```bash
# Requires valid BearDog license for commercial use
docker-compose -f docker-compose.production.yml up
```

---

## 🎯 **Quick Start Guide**

### 🏃‍♂️ **Option 1: Free Gaming Bridge (AGPL 3.0)**

Perfect for individuals, students, researchers, and open source projects:

```bash
# Clone repository
git clone https://github.com/ecoPrimals/SongBird.git
cd SongBird

# Start core gaming bridge (100% free)
docker-compose -f docker-compose.core.yml up -d

# Gaming bridge available at:
# - UDP ports 7000-8000 (gaming traffic)
# - HTTP API: http://localhost:8080
# - Built-in metrics: http://localhost:8081/metrics
```

### 🏢 **Option 2: Enterprise Production (BearDog Licensed)**

For commercial organizations requiring enterprise monitoring and deployment:

```bash
# Ensure you have a valid BearDog license
export BEARDOG_LICENSE_KEY="your-license-key"

# Start full production deployment
docker-compose -f docker-compose.production.yml up -d

# Enterprise services:
# - Gaming bridge: localhost:80 (load balanced)
# - Grafana dashboards: http://localhost:3000
# - Prometheus metrics: http://localhost:9090
# - HAProxy stats: http://localhost:8404/stats
```

---

## 🎓 **Get Your FREE BearDog License**

### ✅ **Automatically FREE for:**
- **Universities**: .edu email addresses
- **Research Institutions**: .org research organizations
- **Educational Use**: Student projects and academic research

### 📝 **Request FREE License:**
If you're an individual developer, power user, or working on open source projects:

1. **Email**: licenses@beardog.dev
2. **Subject**: "Free BearDog License Request - Songbird Orchestrator"
3. **Include**:
   - Name and organization (if any)
   - Use case description
   - Confirmation this is for non-commercial use

**Response Time**: Usually within 24 hours

---

## 🤝 **Why This Licensing Model?**

### 🎯 **Our Goals:**
1. **Keep Gaming Free**: Core gaming functionality stays AGPL 3.0 forever
2. **Support Open Source**: Free access for education, research, and individual use
3. **Fund Development**: Commercial licenses fund continued development
4. **Prevent Lock-in**: You can always use the free core version

### 🌟 **Benefits:**
- **Developers**: Full gaming bridge functionality for free
- **Students**: Learn and experiment without cost barriers
- **Researchers**: Use advanced networking for academic projects
- **Companies**: Pay for enterprise features that provide business value

---

## 🔍 **License Compliance**

### ✅ **AGPL 3.0 Compliance (Core Gaming Bridge):**
- Source code must remain available if you distribute
- Network use triggers AGPL requirements (provide source to users)
- Modifications must be shared under AGPL 3.0
- Commercial use of core gaming bridge is **completely free**

### 🔐 **BearDog License Compliance (Biome Edge):**
- Commercial use requires valid paid license
- Free licenses for education/research/individual use
- 30-day grace period for license acquisition
- License validation automatic for whitelisted domains

---

## 💡 **FAQ**

### **Q: Can I use the gaming bridge commercially for free?**
**A:** Yes! The core gaming bridge (all the gaming functionality) is AGPL 3.0 and free for commercial use. You only need BearDog licenses for the external monitoring/production tools.

### **Q: What if I just want to play games with friends?**
**A:** Use the free core deployment! It has everything you need for gaming.

### **Q: I'm a startup - do I need to pay?**
**A:** Core gaming bridge is free. For enterprise monitoring, contact us - we have startup-friendly pricing.

### **Q: Can I contribute to the project?**
**A:** Absolutely! All contributions to the core AGPL 3.0 codebase are welcome.

### **Q: What happens if my BearDog license expires?**
**A:** The core gaming bridge keeps working. Only external monitoring services are affected.

---

## 📞 **Contact & Support**

- **General Questions**: support@songbird-orchestrator.dev
- **Licensing**: licenses@beardog.dev
- **Commercial Sales**: sales@beardog.dev
- **Technical Support**: docs.songbird-orchestrator.dev

**🎮 Happy Gaming! The core functionality will always be free.** 🎮 