# 🚀 **SONGBIRD PRODUCTION DEPLOYMENT GUIDE**

**Version**: 1.0.0  
**Date**: September 15, 2025  
**Status**: ✅ **PRODUCTION READY**

---

## 📋 **QUICK START CHECKLIST**

- [ ] **Environment Setup** (15 minutes)
- [ ] **Security Configuration** (10 minutes)
- [ ] **Database Setup** (5 minutes)
- [ ] **Network Configuration** (10 minutes)
- [ ] **Monitoring Setup** (15 minutes)
- [ ] **Deployment** (10 minutes)
- [ ] **Validation** (10 minutes)

**Total Time**: ~75 minutes

---

## 🔧 **1. ENVIRONMENT SETUP**

### **Step 1: Copy Configuration Template**
```bash
# Copy the comprehensive configuration template
cp config.env.example .env

# Make it secure (readable only by owner)
chmod 600 .env
```

### **Step 2: Generate Security Keys**
```bash
# Generate JWT secret (required)
openssl rand -hex 64 > jwt_secret.txt

# Generate encryption key (required)
openssl rand -hex 32 > encryption_key.txt

# Store securely and update .env
echo "SONGBIRD_JWT_SECRET=$(cat jwt_secret.txt)" >> .env
echo "SONGBIRD_ENCRYPTION_KEY=$(cat encryption_key.txt)" >> .env

# Clean up temporary files
rm jwt_secret.txt encryption_key.txt
```

### **Step 3: Configure Environment Variables**
Edit `.env` with your specific values:

```bash
# ===============================================
# 🔧 CORE NETWORK CONFIGURATION
# ===============================================
SONGBIRD_BIND_ADDRESS=0.0.0.0              # Production: bind to all interfaces
SONGBIRD_PORT=443                           # Production: use HTTPS port
SONGBIRD_ENABLE_TLS=true                    # Production: enable TLS

# ===============================================
# 🔐 SECURITY CONFIGURATION (REQUIRED)
# ===============================================
SONGBIRD_JWT_SECRET=<your-generated-secret>
SONGBIRD_ENCRYPTION_KEY=<your-generated-key>
SONGBIRD_ENABLE_2FA=true                    # Production: enable 2FA

# ===============================================
# 🏭 PRIMAL ENDPOINTS
# ===============================================
PRIMAL_SECURITY_ENDPOINT=https://security.yourcompany.com
PRIMAL_COMPUTE_ENDPOINT=https://compute.yourcompany.com
PRIMAL_STORAGE_ENDPOINT=https://storage.yourcompany.com
PRIMAL_AI_ENDPOINT=https://ai.yourcompany.com

# ===============================================
# 💾 DATABASE CONFIGURATION
# ===============================================
DATABASE_URL=postgresql://user:password@localhost:5432/songbird
DATABASE_POOL_SIZE=20
```

---

## 🔐 **2. SECURITY CONFIGURATION**

### **TLS Certificate Setup**
```bash
# Option 1: Use Let's Encrypt (recommended for public deployments)
certbot certonly --standalone -d your-domain.com

# Option 2: Use your existing certificates
# Copy your certificates to:
# /etc/ssl/certs/songbird.crt
# /etc/ssl/private/songbird.key

# Update .env with certificate paths
echo "SONGBIRD_TLS_CERT_PATH=/etc/ssl/certs/songbird.crt" >> .env
echo "SONGBIRD_TLS_KEY_PATH=/etc/ssl/private/songbird.key" >> .env
```

### **Firewall Configuration**
```bash
# Allow HTTPS traffic
sudo ufw allow 443/tcp

# Allow HTTP for redirects (optional)
sudo ufw allow 80/tcp

# Allow specific primal endpoints
sudo ufw allow 8001:8005/tcp

# Enable firewall
sudo ufw enable
```

---

## 💾 **3. DATABASE SETUP**

### **PostgreSQL Setup** (Recommended)
```bash
# Install PostgreSQL
sudo apt update && sudo apt install postgresql postgresql-contrib

# Create database and user
sudo -u postgres createuser --interactive songbird
sudo -u postgres createdb songbird -O songbird

# Set password
sudo -u postgres psql -c "ALTER USER songbird PASSWORD 'your-secure-password';"

# Update DATABASE_URL in .env
DATABASE_URL=postgresql://songbird:your-secure-password@localhost:5432/songbird
```

### **Alternative: SQLite** (Development/Small Deployments)
```bash
# For smaller deployments, SQLite is sufficient
DATABASE_URL=sqlite://./songbird.db
```

---

## 🌐 **4. NETWORK CONFIGURATION**

### **Reverse Proxy Setup** (Nginx)
```nginx
# /etc/nginx/sites-available/songbird
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/ssl/certs/songbird.crt;
    ssl_certificate_key /etc/ssl/private/songbird.key;

    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check endpoint
    location /health {
        proxy_pass http://127.0.0.1:8080/health;
        access_log off;
    }
}
```

Enable the configuration:
```bash
sudo ln -s /etc/nginx/sites-available/songbird /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

---

## 📊 **5. MONITORING SETUP**

### **System Monitoring**
```bash
# Update .env with monitoring configuration
echo "SONGBIRD_METRICS_INTERVAL_SECS=60" >> .env
echo "SONGBIRD_HEALTH_CHECK_INTERVAL_SECS=30" >> .env
echo "SONGBIRD_STRUCTURED_LOGGING=true" >> .env
```

### **Log Aggregation** (Optional)
```bash
# Configure log rotation
sudo tee /etc/logrotate.d/songbird <<EOF
/var/log/songbird/*.log {
    daily
    missingok
    rotate 30
    compress
    notifempty
    create 0644 songbird songbird
    postrotate
        systemctl reload songbird
    endscript
}
EOF
```

---

## 🚀 **6. DEPLOYMENT**

### **Build for Production**
```bash
# Build optimized release version
cargo build --release --workspace

# Create deployment directory
sudo mkdir -p /opt/songbird
sudo chown $USER:$USER /opt/songbird

# Copy binaries
cp target/release/songbird* /opt/songbird/
cp .env /opt/songbird/
```

### **Systemd Service Setup**
```bash
# Create systemd service file
sudo tee /etc/systemd/system/songbird.service <<EOF
[Unit]
Description=Songbird Universal Orchestrator
After=network.target postgresql.service

[Service]
Type=simple
User=songbird
Group=songbird
WorkingDirectory=/opt/songbird
Environment=RUST_LOG=info
EnvironmentFile=/opt/songbird/.env
ExecStart=/opt/songbird/songbird
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/songbird

[Install]
WantedBy=multi-user.target
EOF

# Create songbird user
sudo useradd --system --shell /bin/false songbird
sudo chown -R songbird:songbird /opt/songbird

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable songbird
sudo systemctl start songbird
```

---

## ✅ **7. DEPLOYMENT VALIDATION**

### **Health Check**
```bash
# Check service status
sudo systemctl status songbird

# Test health endpoint
curl -k https://your-domain.com/health

# Expected response:
# {"status":"healthy","timestamp":"2025-09-15T...","version":"1.0.0"}
```

### **Security Validation**
```bash
# Test HTTPS redirect
curl -I http://your-domain.com
# Should return: HTTP/1.1 301 Moved Permanently

# Test TLS configuration
curl -I https://your-domain.com
# Should return: HTTP/2 200

# Verify certificate
openssl s_client -connect your-domain.com:443 -servername your-domain.com
```

### **Performance Validation**
```bash
# Test response time
curl -w "@curl-format.txt" -o /dev/null -s https://your-domain.com/health

# Create curl-format.txt:
echo "time_total: %{time_total}" > curl-format.txt
```

### **Federation Validation**
```bash
# Test service discovery
curl -k https://your-domain.com/api/discovery/services

# Test primal connectivity
curl -k https://your-domain.com/api/primals/status
```

---

## 🔧 **8. POST-DEPLOYMENT MAINTENANCE**

### **Monitoring Commands**
```bash
# View logs
sudo journalctl -u songbird -f

# Check metrics
curl -k https://your-domain.com/metrics

# Monitor resource usage
sudo systemctl status songbird
htop -p $(pgrep songbird)
```

### **Backup Procedures**
```bash
# Database backup (PostgreSQL)
pg_dump songbird > songbird_backup_$(date +%Y%m%d).sql

# Configuration backup
cp /opt/songbird/.env /opt/songbird/backups/.env.$(date +%Y%m%d)

# Binary backup
tar -czf songbird_binaries_$(date +%Y%m%d).tar.gz /opt/songbird/songbird*
```

### **Update Procedures**
```bash
# Stop service
sudo systemctl stop songbird

# Backup current version
cp /opt/songbird/songbird /opt/songbird/songbird.backup

# Deploy new version
cp target/release/songbird /opt/songbird/

# Start service
sudo systemctl start songbird

# Verify deployment
curl -k https://your-domain.com/health
```

---

## 🚨 **9. TROUBLESHOOTING**

### **Common Issues**

**Service won't start:**
```bash
# Check logs
sudo journalctl -u songbird -n 50

# Check configuration
songbird --check-config

# Verify permissions
ls -la /opt/songbird/
```

**TLS certificate issues:**
```bash
# Verify certificate
openssl x509 -in /etc/ssl/certs/songbird.crt -text -noout

# Check certificate expiry
openssl x509 -in /etc/ssl/certs/songbird.crt -noout -dates
```

**Database connection issues:**
```bash
# Test database connection
psql $DATABASE_URL -c "SELECT 1;"

# Check database logs
sudo tail -f /var/log/postgresql/postgresql-*.log
```

**Performance issues:**
```bash
# Monitor resource usage
htop
iostat -x 1
netstat -tulpn | grep songbird

# Check for memory leaks
valgrind --leak-check=full /opt/songbird/songbird
```

---

## 📞 **10. SUPPORT & RESOURCES**

### **Documentation**
- **API Reference**: `docs/API_REFERENCE.md`
- **Configuration Guide**: `config.env.example`
- **Architecture Overview**: `ARCHITECTURE_OVERVIEW.md`

### **Monitoring Endpoints**
- **Health Check**: `https://your-domain.com/health`
- **Metrics**: `https://your-domain.com/metrics`
- **Service Discovery**: `https://your-domain.com/api/discovery`

### **Emergency Procedures**
- **Emergency Stop**: `sudo systemctl stop songbird`
- **Rollback**: `cp /opt/songbird/songbird.backup /opt/songbird/songbird && sudo systemctl restart songbird`
- **Database Recovery**: `psql songbird < songbird_backup_YYYYMMDD.sql`

---

**🎯 DEPLOYMENT COMPLETE! Your Songbird ecosystem is now running in production! 🎯**

*For additional support, refer to the comprehensive documentation in the `docs/` directory or the troubleshooting guides.* 