# Songbird Configuration

XDG-compliant configuration templates

## 📋 **Overview**

Songbird uses a hierarchical configuration system following XDG Base Directory Specification:

1. **Environment Variables** (highest priority)
2. **User Config File** (`~/.config/songbird/songbird.toml`)
3. **System Config File** (`/etc/songbird/songbird.toml`)
4. **Built-in Defaults** (lowest priority)

---

## 🚀 **Quick Start**

### Create User Configuration:

```bash
# Create config directory
mkdir -p ~/.config/songbird

# Copy example config
cp songbird.toml.example ~/.config/songbird/songbird.toml

# Edit configuration
nano ~/.config/songbird/songbird.toml
```

### Override with Environment Variables:

```bash
# Override family ID
export SONGBIRD_FAMILY_ID="my-game"

# Override log level
export RUST_LOG=debug

# Override HTTP port
export SONGBIRD_HTTP_PORT=8081

# Run Songbird
songbird
```

---

## 📂 **Configuration Locations**

### Linux (XDG-Compliant):

```
User Config:     ~/.config/songbird/songbird.toml
System Config:   /etc/songbird/songbird.toml
Runtime Dir:     $XDG_RUNTIME_DIR/songbird/
State Dir:       ~/.local/state/songbird/ or /var/lib/songbird/
Cache Dir:       ~/.cache/songbird/ or /var/cache/songbird/
```

### Windows:

```
User Config:     %APPDATA%\songbird\songbird.toml
System Config:   %PROGRAMDATA%\songbird\songbird.toml
Runtime Dir:     %LOCALAPPDATA%\Temp\songbird\
State Dir:       %LOCALAPPDATA%\songbird\state\
Cache Dir:       %LOCALAPPDATA%\songbird\cache\
```

### Android (Termux):

```
User Config:     ~/.config/songbird/songbird.toml
Runtime Dir:     /data/data/com.termux/files/home/.songbird/runtime/
State Dir:       ~/.local/state/songbird/
Cache Dir:       ~/.cache/songbird/
```

---

## ⚙️ **Configuration Sections**

### `[general]`

Basic configuration:

```toml
[general]
family_id = "my-game"     # Instance identifier
mode = "daemon"           # daemon, console, usb-live-spore, android
log_level = "info"        # trace, debug, info, warn, error
```

### `[network]`

Network settings:

```toml
[network]
http_port = 8080          # HTTP server port
mdns_enabled = true       # Enable mDNS discovery
discovery_port = 5353     # Discovery service port
```

### `[ipc]`

Inter-Process Communication:

```toml
[ipc]
# Unix socket path (Linux/macOS)
socket_path = "/run/user/1000/songbird/songbird.sock"

# TCP fallback (Windows)
tcp_address = "127.0.0.1:9876"

ipc_enabled = true
```

### `[security-provider]`

Security provider crypto delegation (capability-based discovery):

```toml
[security-provider]
socket_path = "/run/user/1000/biomeos/security.sock"
enabled = true
```

### `[security]`

Security settings:

```toml
[security]
tls_enabled = true
btsp_enabled = true       # BTSP — handshake on UDS accept when FAMILY_ID set
```

### `[federation]`

Multi-instance federation:

```toml
[federation]
enabled = true
discovery_method = "mdns"  # mdns, neural-api, manual
```

### `[monitoring]`

Health and metrics:

```toml
[monitoring]
health_check_enabled = true
health_check_interval = 30
```

### `[performance]`

Performance tuning:

```toml
[performance]
connection_pool_size = 100
request_timeout_ms = 30000
max_concurrent_requests = 1000
```

### `[storage]`

Storage directories:

```toml
[storage]
state_dir = "/var/lib/songbird"
cache_dir = "/var/cache/songbird"
cache_ttl = 3600
max_cache_size_mb = 100
```

---

## 🔐 **Environment Variable Mapping**

| Config Key | Environment Variable | Example |
|------------|---------------------|---------|
| `general.family_id` | `SONGBIRD_FAMILY_ID` | `export SONGBIRD_FAMILY_ID=my-game` |
| `general.log_level` | `RUST_LOG` | `export RUST_LOG=debug` |
| `general.mode` | `SONGBIRD_MODE` | `export SONGBIRD_MODE=daemon` |
| `network.http_port` | `SONGBIRD_HTTP_PORT` | `export SONGBIRD_HTTP_PORT=8081` |
| `ipc.socket_path` | `SONGBIRD_SOCKET` | `export SONGBIRD_SOCKET=/custom/path.sock` |
| `security-provider.socket_path` | `SECURITY_PROVIDER_SOCKET` | `export SECURITY_PROVIDER_SOCKET=/run/user/1000/biomeos/security.sock` |

---

## 📝 **Example Configurations**

### Development (Debug Logging):

```toml
[general]
family_id = "dev"
mode = "console"
log_level = "debug"

[network]
http_port = 8080
mdns_enabled = true

[ipc]
ipc_enabled = true

[security-provider]
enabled = true

[federation]
enabled = false
```

### Production (Optimized):

```toml
[general]
family_id = "production"
mode = "daemon"
log_level = "info"

[network]
http_port = 8080
mdns_enabled = true

[ipc]
ipc_enabled = true

[security-provider]
enabled = true

[security]
tls_enabled = true
btsp_enabled = true

[federation]
enabled = true
discovery_method = "neural-api"

[monitoring]
health_check_enabled = true
health_check_interval = 30

[performance]
connection_pool_size = 200
request_timeout_ms = 30000
max_concurrent_requests = 2000

[storage]
cache_ttl = 7200
max_cache_size_mb = 500
```

### USB Live Spore:

```toml
[general]
family_id = "usb-spore"
mode = "usb-live-spore"
log_level = "info"

[network]
http_port = 8080
mdns_enabled = true

[ipc]
ipc_enabled = true
# Uses ephemeral runtime directory

[storage]
# Uses USB drive for persistent data
state_dir = "/media/usb/data/songbird"
cache_dir = "/tmp/songbird-cache"
cache_ttl = 3600
max_cache_size_mb = 50
```

### Multi-Instance (Family 1):

```toml
[general]
family_id = "pixelgame"
mode = "daemon"
log_level = "info"

[network]
http_port = 8081  # Different port per instance

[ipc]
socket_path = "/run/songbird-pixelgame/songbird.sock"

[storage]
state_dir = "/var/lib/songbird-pixelgame"
cache_dir = "/var/cache/songbird-pixelgame"
```

---

## 🔄 **Configuration Priority**

Songbird uses this priority order (highest first):

1. **Command-line arguments** (future)
2. **Environment variables**
3. **User config file** (`~/.config/songbird/songbird.toml`)
4. **System config file** (`/etc/songbird/songbird.toml`)
5. **Built-in defaults**

Example:
```bash
# Config file: family_id = "prod"
# But environment overrides:
export SONGBIRD_FAMILY_ID="dev"
songbird
# Result: Uses "dev" (environment wins!)
```

---

## 🛠️ **Validation**

### Check Current Configuration:

```bash
# Run with --config-check flag (future feature)
songbird --config-check

# Or check logs at startup:
songbird 2>&1 | grep "Configuration loaded"
```

### Validate TOML Syntax:

```bash
# Install TOML validator
cargo install taplo-cli

# Validate config
taplo check ~/.config/songbird/songbird.toml
```

---

## 📚 **Related Documentation**

- [systemd Deployment](../systemd/README.md)
- [USB Live Spore](../usb-live-spore/README.md)
- [Windows Service](../windows-service/README.md)
- [Android Deployment](../android/README.md)

---

**Status:** Production Ready  
**Format:** TOML  
**Standard:** XDG Base Directory Specification
