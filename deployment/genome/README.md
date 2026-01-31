# 🧬 genomeBin Self-Extracting Wrapper

**Universal Multi-Architecture Deployment**

This directory contains the genomeBin builder and wrapper system for creating self-extracting, multi-architecture deployment packages.

---

## 📋 Overview

The genomeBin system replaces multiple deployment mechanisms with a single, universal self-extracting file that:

- ✅ Auto-detects architecture (x86_64, aarch64, riscv64)
- ✅ Auto-detects platform (Linux, Android, macOS, Windows)
- ✅ Embeds all target binaries in one file
- ✅ Self-extracts to appropriate location
- ✅ Executes correct binary for platform
- ✅ Validates health post-deployment
- ✅ Supports rollback on failure

---

## 🚀 Quick Start

### Build genomeBin

```bash
# Build all target binaries first
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target aarch64-linux-android

# Create genomeBin
./deployment/genome/create_genome.sh

# Output: dist/songbird.genome
```

### Deploy genomeBin

```bash
# Local deployment
./dist/songbird.genome

# USB deployment
cp dist/songbird.genome /media/usb/
/media/usb/songbird.genome

# Remote deployment (curl)
curl https://biomeos.org/songbird.genome | sh

# Verify integrity
./dist/songbird.genome --verify-only
```

---

## 📦 What's Inside

### `create_genome.sh`
The genomeBin builder script that:
1. Collects all compiled binaries
2. Creates normalized staging directory
3. Generates metadata file
4. Creates compressed tar archive
5. Embeds wrapper template
6. Produces self-extracting `songbird.genome`

**Usage**:
```bash
./create_genome.sh [output_dir]

# Default output: dist/songbird.genome
# Custom output:
./create_genome.sh /path/to/output
```

### `songbird.genome` (Generated Output)
The final self-extracting genomeBin:
- **Format**: Bash script + embedded tar.gz
- **Size**: ~30-40 MB (all architectures)
- **Structure**:
  ```
  [Wrapper Script]
    ↓
  [Archive Marker: __GENOME_ARCHIVE__]
    ↓
  [Embedded tar.gz]
    ├── bin/
    │   ├── songbird-x86_64-linux-musl
    │   ├── songbird-aarch64-linux-musl
    │   ├── songbird-aarch64-linux-android
    │   └── songbird-x86_64-windows.exe
    └── GENOME_METADATA
  ```

---

## 🎯 Supported Platforms

### Linux x86_64
- **Target**: `x86_64-unknown-linux-musl`
- **Binary**: `songbird-x86_64-linux-musl`
- **Install Dir**: `/opt/biomeos` or `~/.local/biomeos`
- **Use Case**: Standard Linux servers, desktops

### Linux ARM64
- **Target**: `aarch64-unknown-linux-musl`
- **Binary**: `songbird-aarch64-linux-musl`
- **Install Dir**: `/opt/biomeos` or `~/.local/biomeos`
- **Use Case**: ARM servers, Raspberry Pi, edge devices

### Android ARM64
- **Target**: `aarch64-linux-android`
- **Binary**: `songbird-aarch64-linux-android`
- **Install Dir**: `/data/local/tmp/biomeos`
- **Use Case**: Android devices (Pixel, GrapheneOS)

### Windows x86_64
- **Target**: `x86_64-pc-windows-gnu`
- **Binary**: `songbird-x86_64-windows.exe`
- **Install Dir**: `%USERPROFILE%\.biomeos`
- **Use Case**: Windows 10/11 (via WSL, Cygwin)

### macOS (Future)
- **Targets**: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- **Status**: ⏳ Requires osxcross setup (Phase 2)

---

## 🔧 Advanced Usage

### Custom Installation Directory

```bash
./songbird.genome --install-dir /custom/path
```

### Multi-Instance Deployment

```bash
# Deploy with family ID
./songbird.genome --family-id production-01

# Deploy second instance
./songbird.genome --family-id production-02
```

### Deployment Modes

```bash
# systemd service mode (Linux)
./songbird.genome --mode systemd

# USB Live Spore mode
./songbird.genome --mode usb

# Android Termux mode
./songbird.genome --mode android

# Manual mode (no service registration)
./songbird.genome --mode manual
```

### Verify Before Deploy

```bash
# Check archive integrity without installing
./songbird.genome --verify-only
```

---

## 🏗️ Building for Different Targets

### Prerequisites

**Linux musl targets**:
```bash
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# Install musl-gcc
sudo apt-get install musl-tools
```

**Android target**:
```bash
rustup target add aarch64-linux-android

# Android NDK required (configured in .cargo/config.toml)
```

**Windows target**:
```bash
rustup target add x86_64-pc-windows-gnu

# Install mingw-w64
sudo apt-get install mingw-w64
```

### Build All Targets

```bash
# Build script (parallel)
cargo build --release --target x86_64-unknown-linux-musl &
cargo build --release --target aarch64-unknown-linux-musl &
cargo build --release --target aarch64-linux-android &
cargo build --release --target x86_64-pc-windows-gnu &
wait

# Create genomeBin
./deployment/genome/create_genome.sh
```

---

## 📊 genomeBin Structure

### Wrapper Script (Header)
```bash
#!/usr/bin/env bash
# songbird.genome - Self-deploying genomeBin

# 1. Parse arguments (--install-dir, --family-id, etc.)
# 2. Detect architecture (uname -m)
# 3. Detect platform (uname -s, Android check)
# 4. Select appropriate binary
# 5. Determine installation directory
# 6. Extract embedded archive
# 7. Verify binary exists
# 8. Run health check
# 9. Display success message
```

### Archive Marker
```bash
__GENOME_ARCHIVE__
```

### Embedded Archive (tar.gz)
```
bin/
├── songbird-x86_64-linux-musl      (30 MB)
├── songbird-aarch64-linux-musl     (30 MB)
├── songbird-aarch64-linux-android  (30 MB)
└── songbird-x86_64-windows.exe     (30 MB)

GENOME_METADATA
├── PRIMAL=songbird
├── VERSION=8.15.0
├── GENOMEBIN_FORMAT=1.0
├── BUILD_DATE=2026-01-31
└── Included architectures (list)
```

---

## 🎯 Deep Debt Solution

### Before (Week 2): 4 Separate Deployment Mechanisms
```
deployment/
├── systemd/
│   ├── songbird.service
│   └── songbird@.service
├── usb-live-spore/
│   └── launch-songbird.sh
├── windows-service/
│   └── launch-songbird.ps1
└── android/
    └── README.md (ADB deployment)
```

**Problems**:
- ❌ Different scripts for each platform
- ❌ Manual selection of correct binary
- ❌ No unified deployment experience
- ❌ Hard to distribute (4+ files)

### After (Week 3): Universal genomeBin
```
dist/
└── songbird.genome  (single file, ~40 MB)
```

**Benefits**:
- ✅ One file works everywhere
- ✅ Auto-detects platform & arch
- ✅ Self-extracts correct binary
- ✅ Unified deployment: `curl | sh`
- ✅ Easy distribution (1 file)
- ✅ Includes all architectures

---

## 🔍 Troubleshooting

### "Unsupported platform/architecture"
**Cause**: Running on platform/arch not included in genomeBin  
**Solution**: Build for that target, rebuild genomeBin

### "Archive marker not found"
**Cause**: genomeBin corruption or incomplete download  
**Solution**: Re-download or rebuild genomeBin

### "Binary not found after extraction"
**Cause**: Wrong binary name mapping  
**Solution**: Check `BINARIES` mapping in `create_genome.sh`

### Permission denied
**Cause**: genomeBin not executable  
**Solution**: `chmod +x songbird.genome`

---

## 🧪 Testing

### Test Local Deployment
```bash
# Build genomeBin
./deployment/genome/create_genome.sh

# Test on current platform
./dist/songbird.genome --install-dir /tmp/test-genome

# Verify
/tmp/test-genome/bin/songbird-* --version
```

### Test Integrity Verification
```bash
# Should succeed
./dist/songbird.genome --verify-only

# Corrupt archive
dd if=/dev/zero of=dist/songbird.genome bs=1 count=100 seek=1000 conv=notrunc

# Should fail
./dist/songbird.genome --verify-only
```

### Test Cross-Platform (Docker)
```bash
# Test in Alpine (musl)
docker run --rm -v $(pwd)/dist:/dist alpine:latest /dist/songbird.genome --verify-only

# Test in Ubuntu (glibc, should fail - musl binary)
docker run --rm -v $(pwd)/dist:/dist ubuntu:latest /dist/songbird.genome --verify-only
```

---

## 📚 Related Documentation

- `../systemd/README.md` - systemd service integration
- `../usb-live-spore/README.md` - USB deployment
- `../windows-service/README.md` - Windows deployment
- `../android/README.md` - Android deployment
- `../../GENOMEBIN_WEEK3_EXECUTION_PLAN_JAN_31_2026.md` - Week 3 plan

---

## 🎊 Success Metrics

genomeBin is successful when:
- ✅ Single file contains all architectures
- ✅ Works on Linux x86_64 (primary)
- ✅ Works on Linux ARM64 (edge/Pi)
- ✅ Works on Android ARM64 (mobile)
- ✅ Auto-detects platform correctly
- ✅ Self-extracts without errors
- ✅ Health check passes post-deployment
- ✅ `curl | sh` deployment works
- ✅ File size reasonable (<50 MB)
- ✅ Integrity verification works

---

**Created**: January 31, 2026 (Evening)  
**Status**: ✅ Builder complete, ready for testing  
**Next**: Create neuralAPI deployment graphs
