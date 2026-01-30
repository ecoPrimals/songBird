# Songbird systemd Service Deployment

**genomeBin Week 2:** Production-ready systemd service units for Linux deployment

## 📋 **Overview**

This directory contains systemd service units for deploying Songbird on Linux systems. Two deployment modes are supported:

1. **Single Instance** (`songbird.service`) - Standard deployment
2. **Multi-Instance** (`songbird@.service`) - Multiple families on same machine

---

## 🚀 **Quick Start - Single Instance**

### Installation:

```bash
# Copy service file
sudo cp songbird.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable auto-start on boot
sudo systemctl enable songbird

# Start service
sudo systemctl start songbird

# Check status
systemctl status songbird
```

### View Logs:

```bash
# Live logs
journalctl -u songbird -f

# Last 100 lines
journalctl -u songbird -n 100

# Since boot
journalctl -u songbird -b
```

---

## 🔀 **Multi-Instance Deployment**

Run multiple Songbird instances for different game families:

### Installation:

```bash
# Copy template service file
sudo cp songbird@.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload
```

### Start Multiple Instances:

```bash
# Start instance for "pixelgame" family
sudo systemctl start songbird@pixelgame

# Start instance for "tournament" family
sudo systemctl start songbird@tournament

# Start instance for "dev" family
sudo systemctl start songbird@dev

# Enable auto-start for specific families
sudo systemctl enable songbird@pixelgame
sudo systemctl enable songbird@tournament
```

### Instance Isolation:

Each instance gets its own:
- **Family ID:** `%i` (instance name)
- **Socket Path:** `/run/songbird-%i/songbird.sock`
- **State Directory:** `/var/lib/songbird-%i/`
- **Cache Directory:** `/var/cache/songbird-%i/`
- **Log Identifier:** `songbird@%i`

### View Instance Logs:

```bash
# Live logs for "pixelgame" family
journalctl -u songbird@pixelgame -f

# Status for specific instance
systemctl status songbird@tournament
```

---

## 📂 **Directory Structure**

```
/usr/local/bin/songbird              # Binary
/etc/systemd/system/songbird.service # Service unit
/run/songbird/songbird.sock          # Runtime socket (XDG_RUNTIME_DIR)
/var/lib/songbird/                   # State (persistent data)
/var/cache/songbird/                 # Cache (temporary data)
/etc/songbird/                       # Configuration (optional)
```

**Multi-Instance:**
```
/run/songbird-pixelgame/songbird.sock
/var/lib/songbird-pixelgame/
/var/cache/songbird-pixelgame/
```

---

## 🔐 **Security Hardening**

### Features Enabled:

✅ **Process Isolation:**
- `PrivateTmp=yes` - Private /tmp directory
- `NoNewPrivileges=yes` - Prevent privilege escalation
- `ProtectSystem=strict` - Read-only system directories
- `ProtectHome=yes` - No access to /home

✅ **Namespace Isolation:**
- `PrivateDevices=yes` - No access to /dev
- `ProtectKernelTunables=yes` - No access to /sys
- `ProtectKernelModules=yes` - Cannot load kernel modules
- `ProtectControlGroups=yes` - Read-only /sys/fs/cgroup

✅ **System Call Filtering:**
- `SystemCallFilter=@system-service` - Allow only service syscalls
- `SystemCallFilter=~@privileged @resources` - Block privileged/resource syscalls
- `SystemCallArchitectures=native` - Prevent foreign arch execution

✅ **Capabilities:**
- `CAP_NET_BIND_SERVICE` only (minimal required for networking)
- All other capabilities dropped

✅ **Network Restrictions:**
- `RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX` - Only required protocols

### Recommended: Dedicated User

For maximum security, run as dedicated user:

```bash
# Create songbird user (no login, no home directory)
sudo useradd -r -s /bin/false -d /var/lib/songbird songbird

# Create directories with correct ownership
sudo mkdir -p /var/lib/songbird /var/cache/songbird
sudo chown songbird:songbird /var/lib/songbird /var/cache/songbird
sudo chmod 0700 /var/lib/songbird /var/cache/songbird

# Edit service file to enable User/Group directives:
# User=songbird
# Group=songbird
```

---

## 🎯 **Environment Variables**

### Default Variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SONGBIRD_FAMILY_ID` | `default` | Family identifier (instance isolation) |
| `RUST_LOG` | `info` | Logging level (trace/debug/info/warn/error) |
| `SONGBIRD_MODE` | `daemon` | Operation mode |

### Override Variables:

Edit service file to add custom environment variables:

```ini
[Service]
Environment="SONGBIRD_FAMILY_ID=production"
Environment="RUST_LOG=debug"
Environment="SONGBIRD_HTTP_PORT=8080"
Environment="BEARDOG_SOCKET=/custom/path/beardog.sock"
```

Or use override file (recommended):

```bash
# Create override directory
sudo systemctl edit songbird

# Add your overrides:
[Service]
Environment="RUST_LOG=debug"
Environment="SONGBIRD_FAMILY_ID=production"
```

---

## 🔧 **Troubleshooting**

### Service Won't Start:

```bash
# Check status
systemctl status songbird

# View detailed logs
journalctl -u songbird -n 100 --no-pager

# Check for port conflicts
sudo ss -tulpn | grep songbird

# Verify binary exists
ls -l /usr/local/bin/songbird

# Test binary manually
/usr/local/bin/songbird --help
```

### Permission Issues:

```bash
# Check directory ownership
ls -la /var/lib/songbird /var/cache/songbird /run/songbird

# Fix ownership (if running as songbird user)
sudo chown -R songbird:songbird /var/lib/songbird /var/cache/songbird

# Check SELinux context (if applicable)
ls -Z /usr/local/bin/songbird
```

### Socket Connection Issues:

```bash
# Verify socket exists
ls -l /run/songbird/songbird.sock

# Check socket permissions
stat /run/songbird/songbird.sock

# Test socket connection
echo '{"jsonrpc":"2.0","method":"health_check","params":[],"id":1}' | \
  nc -U /run/songbird/songbird.sock
```

---

## 📊 **Monitoring**

### Health Check:

```bash
# Via systemd (requires WatchdogSec support in binary)
systemctl status songbird

# Via JSON-RPC (requires socket)
echo '{"jsonrpc":"2.0","method":"health_check","params":[],"id":1}' | \
  nc -U /run/songbird/songbird.sock
```

### Resource Usage:

```bash
# CPU and memory
systemctl status songbird

# Detailed resource usage
systemd-cgtop -1 | grep songbird

# Open files and sockets
sudo lsof -p $(systemctl show -p MainPID --value songbird)
```

### Log Analysis:

```bash
# Error messages only
journalctl -u songbird -p err

# Count log levels
journalctl -u songbird --since today | \
  grep -oP '(ERROR|WARN|INFO|DEBUG|TRACE)' | sort | uniq -c

# Failed connection attempts
journalctl -u songbird | grep -i "connection.*failed"
```

---

## 🔄 **Restart Policies**

### Automatic Restart:

Service automatically restarts on failure:
- **RestartSec:** 5 seconds delay
- **StartLimitBurst:** 3 attempts
- **StartLimitInterval:** 60 seconds

After 3 failures in 60 seconds, service enters failed state.

### Manual Restart:

```bash
# Restart service
sudo systemctl restart songbird

# Reload configuration (if supported)
sudo systemctl reload songbird

# Stop service
sudo systemctl stop songbird
```

### Reset Failed State:

```bash
# Reset failure count
sudo systemctl reset-failed songbird

# Try starting again
sudo systemctl start songbird
```

---

## 🚀 **Production Deployment Checklist**

- [ ] Binary installed at `/usr/local/bin/songbird`
- [ ] Service file installed at `/etc/systemd/system/`
- [ ] `systemctl daemon-reload` executed
- [ ] Dedicated user created (recommended)
- [ ] Directories created with correct ownership
- [ ] Environment variables configured
- [ ] Service enabled for auto-start
- [ ] Service started successfully
- [ ] Health check passing
- [ ] Logs verified in journald
- [ ] Firewall rules configured (if needed)
- [ ] BearDog integration verified (if available)
- [ ] Monitoring/alerting configured

---

## 📚 **Related Documentation**

- [genomeBin Evolution Analysis](../../GENOMEBIN_EVOLUTION_ANALYSIS_JAN_31_2026.md)
- [genomeBin Week 1 Victory](../../GENOMEBIN_WEEK1_VICTORY_JAN_31_2026.md)
- [Cross-Compilation Progress](../../CROSS_COMPILATION_PROGRESS_JAN_31_2026.md)
- [biomeOS Socket Standard](../../BIOMEOS_SOCKET_STANDARD_COMPLIANCE_JAN_30_2026.md)

---

**Status:** ✅ Production Ready  
**genomeBin:** Week 2 - Deployment Wrappers  
**Compliance:** XDG Base Directory Specification  
**Security:** TRUE ecoBin Security Standards  
**Tested:** Linux x86_64 (glibc + musl)
