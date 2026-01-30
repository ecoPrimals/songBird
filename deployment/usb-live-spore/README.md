# Songbird USB Live Spore Launcher

**genomeBin Week 2:** Portable launcher for USB/removable media deployment

## 🎯 **Overview**

The USB Live Spore launcher enables running Songbird from USB drives or other removable media without system installation. Perfect for:

- **Portable Gaming:** Carry your network bridge on USB
- **Testing:** Try Songbird without installation
- **Air-Gapped Systems:** No internet required after initial copy
- **Multiple Systems:** Same USB works on different Linux machines

---

## 📦 **USB Directory Structure**

```
USB_ROOT/
├── bin/
│   └── songbird              # Static musl binary (29MB)
├── deployment/
│   └── usb-live-spore/
│       ├── launch-songbird.sh    # This launcher
│       └── README.md             # This file
├── data/
│   └── songbird-{family}/    # Persistent data (created at runtime)
└── logs/
    └── songbird-{family}/    # Log files (created at runtime)
```

---

## 🚀 **Quick Start**

### 1. Prepare USB Drive:

```bash
# Build static binary (x86_64-unknown-linux-musl)
cargo build --release --target x86_64-unknown-linux-musl

# Create USB structure
USB_ROOT="/media/your-usb-drive"
mkdir -p "${USB_ROOT}"/{bin,data,logs}

# Copy binary
cp target/x86_64-unknown-linux-musl/release/songbird "${USB_ROOT}/bin/"

# Copy launcher
cp -r deployment/usb-live-spore "${USB_ROOT}/deployment/"

# Make launcher executable
chmod +x "${USB_ROOT}/deployment/usb-live-spore/launch-songbird.sh"
```

### 2. Launch on Any Linux System:

```bash
# Navigate to USB
cd /media/your-usb-drive/deployment/usb-live-spore

# Launch with default family ID
./launch-songbird.sh

# Or specify custom family ID
./launch-songbird.sh my-game-family
```

### 3. Stop:

Press `Ctrl+C` to gracefully stop Songbird and cleanup runtime files.

---

## 🎮 **Usage Examples**

### Default Family:

```bash
./launch-songbird.sh
# Family ID: "usb-spore" (default)
```

### Custom Family:

```bash
./launch-songbird.sh pixelgame
# Family ID: "pixelgame"

./launch-songbird.sh tournament
# Family ID: "tournament"
```

### With Debug Logging:

```bash
RUST_LOG=debug ./launch-songbird.sh my-family
```

### Multiple Instances:

```bash
# Terminal 1
./launch-songbird.sh family1

# Terminal 2
./launch-songbird.sh family2

# Each gets isolated directories and sockets!
```

---

## 📂 **Runtime Directories**

### Persistent Data (on USB):

```
/media/usb/data/songbird-{family}/
  └── (state, configuration, persistent data)

/media/usb/logs/songbird-{family}/
  └── songbird-YYYYMMDD-HHMMSS.log
```

### Ephemeral Runtime (system /tmp):

```
${XDG_RUNTIME_DIR}/songbird-{family}/
  ├── songbird.sock          # IPC socket
  ├── songbird.pid           # Process ID
  └── (temporary files)
```

**Cleanup:** Runtime files automatically deleted on exit!

---

## ✨ **Features**

### ✅ **Fully Portable:**
- Static musl binary (no dynamic dependencies)
- Works on any x86_64 Linux (kernel 2.6.32+)
- No installation or root access required

### ✅ **XDG-Compliant:**
- Uses `$XDG_RUNTIME_DIR` for ephemeral data
- Respects standard directory conventions
- Clean separation of persistent vs. temporary

### ✅ **Automatic Cleanup:**
- Graceful shutdown on `Ctrl+C`
- Removes runtime directories
- Kills child processes
- No leftover files

### ✅ **Zero Hardcoding:**
- Auto-detects USB mount point
- Runtime discovery of BearDog
- Environment-based configuration
- Capability-based architecture

### ✅ **Multi-Instance:**
- Run multiple families simultaneously
- Isolated data and sockets per family
- No conflicts between instances

---

## 🔧 **Configuration**

### Environment Variables:

Override via command line:

```bash
# Custom family ID
./launch-songbird.sh my-family

# Debug logging
RUST_LOG=debug ./launch-songbird.sh

# Custom runtime directory
XDG_RUNTIME_DIR=/custom/path ./launch-songbird.sh
```

### Auto-Discovery:

The launcher automatically:
- Detects USB mount point
- Finds Songbird binary
- Creates necessary directories
- Sets up runtime environment
- Discovers BearDog (if present)

---

## 🛠️ **Troubleshooting**

### Binary Not Found:

```
[ERROR] Songbird binary not found: /media/usb/bin/songbird
```

**Solution:** Verify USB structure matches expected layout. Binary must be at `USB_ROOT/bin/songbird`.

### Binary Won't Execute:

```
bash: ./songbird: cannot execute binary file: Exec format error
```

**Solution:** Wrong architecture! Use `x86_64-unknown-linux-musl` target:
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

### Permission Denied:

```
bash: ./launch-songbird.sh: Permission denied
```

**Solution:** Make launcher executable:
```bash
chmod +x launch-songbird.sh
```

### Failed to Start:

Check logs for details:
```bash
tail -50 /media/usb/logs/songbird-*/songbird-*.log
```

### Socket Connection Issues:

Verify socket exists:
```bash
ls -l ${XDG_RUNTIME_DIR}/songbird-*/songbird.sock
```

Test connection:
```bash
echo '{"jsonrpc":"2.0","method":"health_check","id":1}' | \
  nc -U ${XDG_RUNTIME_DIR}/songbird-*/songbird.sock
```

---

## 📊 **Compatibility**

### Tested Distributions:

✅ Ubuntu 20.04+  
✅ Debian 10+  
✅ Fedora 33+  
✅ Arch Linux (current)  
✅ Alpine Linux 3.12+  

### Requirements:

- **Architecture:** x86_64 (AMD64)
- **Kernel:** Linux 2.6.32+ (ancient!)
- **Glibc:** NONE (static musl binary)
- **Disk Space:** ~50MB (binary + data)
- **RAM:** ~50MB (typical usage)

---

## 🚀 **Advanced Usage**

### Integration with BearDog:

If BearDog binary is present at `USB_ROOT/bin/beardog`:

```bash
# Launcher automatically detects and configures
./launch-songbird.sh

# BearDog socket: ${XDG_RUNTIME_DIR}/songbird-{family}/beardog.sock
```

### Background Execution:

For background operation (not recommended for USB):

```bash
nohup ./launch-songbird.sh my-family > /dev/null 2>&1 &

# Note: You'll need to manually kill the process
# Better: Use systemd for background services
```

### Custom Data Directory:

Override persistent data location:

```bash
#!/bin/bash
# Edit launch-songbird.sh to use custom path
USB_DATA="/custom/path/songbird-data"
```

---

## 📚 **Related Documentation**

- [systemd Deployment](../systemd/README.md)
- [Windows Service](../windows-service/README.md)
- [genomeBin Evolution](../../GENOMEBIN_EVOLUTION_ANALYSIS_JAN_31_2026.md)
- [Cross-Compilation Progress](../../CROSS_COMPILATION_PROGRESS_JAN_31_2026.md)

---

**Status:** ✅ Production Ready  
**Target:** x86_64-unknown-linux-musl (static binary)  
**Binary Size:** 29MB  
**Compatibility:** Linux 2.6.32+ (99% coverage)  
**genomeBin:** Week 2 - USB Live Spore Deployment
