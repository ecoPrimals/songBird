#!/usr/bin/env bash
# create_genome.sh - Build universal self-extracting songbird.genome
# 
# This script creates a self-extracting, multi-architecture genomeBin wrapper
# that auto-detects platform and architecture, then deploys the correct binary.
#
# Usage: ./create_genome.sh [output_dir]
#
# Requirements:
#   - All target binaries built (x86_64-linux-musl, aarch64-linux-android, etc.)
#   - tar, gzip available
#   - Binaries in target/release or target/<triple>/release

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_DIR="${1:-${PROJECT_ROOT}/dist}"
GENOME_NAME="songbird.genome"
VERSION="$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}║        🧬 genomeBin Builder - Songbird v${VERSION}                   ║${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if binary exists
check_binary() {
    local target=$1
    local binary_name=$2
    local binary_path="${PROJECT_ROOT}/target/${target}/release/${binary_name}"
    
    if [[ -f "${binary_path}" ]]; then
        echo -e "${GREEN}✓${NC} Found: ${target}/${binary_name}" >&2
        echo "${binary_path}"
        return 0
    else
        echo -e "${YELLOW}⚠${NC}  Missing: ${target}/${binary_name}" >&2
        return 1
    fi
}

# Collect all available binaries
echo -e "${BLUE}📦 Collecting binaries...${NC}"
declare -A BINARIES

# Priority targets from genomeBin Week 1
TARGETS=(
    "x86_64-unknown-linux-musl:songbird"
    "aarch64-unknown-linux-musl:songbird"
    "aarch64-linux-android:songbird"
    "x86_64-pc-windows-gnu:songbird.exe"
)

FOUND_COUNT=0
for target_spec in "${TARGETS[@]}"; do
    target="${target_spec%%:*}"
    binary="${target_spec##*:}"
    if binary_path=$(check_binary "${target}" "${binary}"); then
        BINARIES["${target}"]="${binary_path}"
        ((FOUND_COUNT++)) || true
    fi
done

echo ""
if [[ ${FOUND_COUNT} -eq 0 ]]; then
    echo -e "${RED}✗ No binaries found!${NC}"
    echo -e "${YELLOW}Build targets first:${NC}"
    echo "  cargo build --release --target x86_64-unknown-linux-musl"
    echo "  cargo build --release --target aarch64-unknown-linux-musl"
    echo "  cargo build --release --target aarch64-linux-android"
    echo "  cargo build --release --target x86_64-pc-windows-gnu"
    exit 1
fi

echo -e "${GREEN}✓ Found ${FOUND_COUNT} binaries${NC}"
echo ""

# Create staging directory
STAGING_DIR=$(mktemp -d)
trap 'rm -rf "${STAGING_DIR}"' EXIT

echo -e "${BLUE}📁 Creating staging directory: ${STAGING_DIR}${NC}"

# Copy binaries to staging with normalized names
mkdir -p "${STAGING_DIR}/bin"
for target in "${!BINARIES[@]}"; do
    src="${BINARIES[${target}]}"
    # Normalize binary name to include target triple
    case "${target}" in
        x86_64-unknown-linux-musl)
            dst="${STAGING_DIR}/bin/songbird-x86_64-linux-musl"
            ;;
        aarch64-unknown-linux-musl)
            dst="${STAGING_DIR}/bin/songbird-aarch64-linux-musl"
            ;;
        aarch64-linux-android)
            dst="${STAGING_DIR}/bin/songbird-aarch64-linux-android"
            ;;
        x86_64-pc-windows-gnu)
            dst="${STAGING_DIR}/bin/songbird-x86_64-windows.exe"
            ;;
        *)
            echo -e "${YELLOW}⚠${NC}  Unknown target: ${target}, skipping"
            continue
            ;;
    esac
    
    cp "${src}" "${dst}"
    chmod +x "${dst}"
    echo -e "${GREEN}✓${NC} Staged: $(basename ${dst}) ($(du -h ${dst} | cut -f1))"
done

# Create metadata file
cat > "${STAGING_DIR}/GENOME_METADATA" <<EOF
# genomeBin Metadata
# Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

PRIMAL=songbird
VERSION=${VERSION}
GENOMEBIN_FORMAT=1.0
BUILD_DATE=$(date -u +"%Y-%m-%d")
BUILD_HOST=$(hostname)

# Included architectures:
$(for target in "${!BINARIES[@]}"; do echo "# - ${target}"; done)
EOF

# Create archive
ARCHIVE="${STAGING_DIR}/archive.tar.gz"
echo ""
echo -e "${BLUE}📦 Creating archive...${NC}"
(cd "${STAGING_DIR}" && tar czf "${ARCHIVE}" bin/ GENOME_METADATA)
ARCHIVE_SIZE=$(du -h "${ARCHIVE}" | cut -f1)
echo -e "${GREEN}✓${NC} Archive created: ${ARCHIVE_SIZE}"

# Get archive line number for self-extraction
WRAPPER_TEMPLATE="${SCRIPT_DIR}/genome_wrapper_template.sh"
if [[ ! -f "${WRAPPER_TEMPLATE}" ]]; then
    echo -e "${YELLOW}⚠${NC}  Wrapper template not found, creating default..."
    cat > "${WRAPPER_TEMPLATE}" <<'WRAPPER_EOF'
#!/usr/bin/env bash
# songbird.genome - Universal Self-Deploying genomeBin
# 
# This is a self-extracting, multi-architecture deployment wrapper
# that auto-detects platform and architecture, then deploys the correct binary.
#
# Architecture Detection: x86_64, aarch64, armv7l, riscv64
# Platform Detection: Linux, Android, macOS, Windows (WSL/Cygwin)
# 
# Usage:
#   ./songbird.genome [options]
#
# Options:
#   --install-dir DIR    Installation directory (default: auto-detect)
#   --family-id ID       Family ID for multi-instance (default: default)
#   --mode MODE          Deployment mode (systemd|usb|android|manual)
#   --verify-only        Verify archive integrity, don't install
#   --help               Show this help

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
INSTALL_DIR=""
FAMILY_ID="default"
DEPLOY_MODE="auto"
VERIFY_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --family-id)
            FAMILY_ID="$2"
            shift 2
            ;;
        --mode)
            DEPLOY_MODE="$2"
            shift 2
            ;;
        --verify-only)
            VERIFY_ONLY=true
            shift
            ;;
        --help)
            sed -n '2,/^$/p' "$0" | sed 's/^# //; s/^#//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}║        🧬 songbird.genome - Self-Deploying genomeBin           ║${NC}"
echo -e "${BLUE}║                                                                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Detect architecture
ARCH=$(uname -m)
echo -e "${BLUE}🔍 Architecture:${NC} ${ARCH}"

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
echo -e "${BLUE}🔍 Platform:${NC} ${PLATFORM}"

# Detect Android
if [[ -d /system/bin ]] && [[ -f /system/build.prop ]]; then
    PLATFORM="android"
    echo -e "${BLUE}🔍 Android detected${NC}"
fi

# Select binary based on platform and architecture
case "${PLATFORM}_${ARCH}" in
    linux_x86_64)
        BINARY="songbird-x86_64-linux-musl"
        ;;
    linux_aarch64)
        BINARY="songbird-aarch64-linux-musl"
        ;;
    android_aarch64)
        BINARY="songbird-aarch64-linux-android"
        ;;
    darwin_x86_64)
        BINARY="songbird-x86_64-darwin"
        ;;
    darwin_aarch64|darwin_arm64)
        BINARY="songbird-aarch64-darwin"
        ;;
    *mingw*_x86_64|*cygwin*_x86_64)
        BINARY="songbird-x86_64-windows.exe"
        ;;
    *)
        echo -e "${RED}✗ Unsupported platform/architecture: ${PLATFORM}_${ARCH}${NC}"
        echo ""
        echo "Supported combinations:"
        echo "  - linux_x86_64 (Linux 64-bit Intel/AMD)"
        echo "  - linux_aarch64 (Linux 64-bit ARM)"
        echo "  - android_aarch64 (Android 64-bit ARM)"
        echo "  - darwin_x86_64 (macOS Intel)"
        echo "  - darwin_aarch64 (macOS Apple Silicon)"
        exit 1
        ;;
esac

echo -e "${GREEN}✓${NC} Selected binary: ${BINARY}"
echo ""

# Determine installation directory
if [[ -z "${INSTALL_DIR}" ]]; then
    case "${PLATFORM}" in
        android)
            INSTALL_DIR="/data/local/tmp/biomeos"
            ;;
        linux)
            if [[ -w /opt ]]; then
                INSTALL_DIR="/opt/biomeos"
            else
                INSTALL_DIR="${HOME}/.local/biomeos"
            fi
            ;;
        darwin)
            INSTALL_DIR="${HOME}/Library/Application Support/biomeos"
            ;;
        *)
            INSTALL_DIR="${HOME}/.biomeos"
            ;;
    esac
fi

echo -e "${BLUE}📁 Installation directory:${NC} ${INSTALL_DIR}"

# Find archive start marker
ARCHIVE_LINE=$(awk '/^__GENOME_ARCHIVE__/ {print NR + 1; exit 0; }' "$0")
if [[ -z "${ARCHIVE_LINE}" ]]; then
    echo -e "${RED}✗ Archive marker not found!${NC}"
    exit 1
fi

# Verify-only mode
if [[ "${VERIFY_ONLY}" == "true" ]]; then
    echo ""
    echo -e "${BLUE}🔍 Verifying archive integrity...${NC}"
    if tail -n "+${ARCHIVE_LINE}" "$0" | tar tzf - >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Archive integrity verified${NC}"
        exit 0
    else
        echo -e "${RED}✗ Archive verification failed${NC}"
        exit 1
    fi
fi

# Create installation directory
echo ""
echo -e "${BLUE}📦 Extracting genomeBin...${NC}"
mkdir -p "${INSTALL_DIR}"

# Extract archive
if ! tail -n "+${ARCHIVE_LINE}" "$0" | tar xzf - -C "${INSTALL_DIR}"; then
    echo -e "${RED}✗ Extraction failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓${NC} Extracted to: ${INSTALL_DIR}"

# Verify binary exists
BINARY_PATH="${INSTALL_DIR}/bin/${BINARY}"
if [[ ! -f "${BINARY_PATH}" ]]; then
    echo -e "${RED}✗ Binary not found: ${BINARY_PATH}${NC}"
    echo -e "${YELLOW}Available binaries:${NC}"
    ls -lh "${INSTALL_DIR}/bin/" || true
    exit 1
fi

chmod +x "${BINARY_PATH}"
echo -e "${GREEN}✓${NC} Binary ready: ${BINARY_PATH}"

# Display metadata
if [[ -f "${INSTALL_DIR}/GENOME_METADATA" ]]; then
    echo ""
    echo -e "${BLUE}📋 genomeBin Metadata:${NC}"
    grep -v '^#' "${INSTALL_DIR}/GENOME_METADATA" | grep -v '^$' || true
fi

# Health check
echo ""
echo -e "${BLUE}🏥 Health check...${NC}"
if "${BINARY_PATH}" --version >/dev/null 2>&1; then
    VERSION_INFO=$("${BINARY_PATH}" --version)
    echo -e "${GREEN}✓${NC} ${VERSION_INFO}"
else
    echo -e "${YELLOW}⚠${NC}  Version check failed (non-critical)"
fi

# Success
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                                ║${NC}"
echo -e "${GREEN}║        ✅ songbird.genome DEPLOYED SUCCESSFULLY! ✅            ║${NC}"
echo -e "${GREEN}║                                                                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}🚀 To start Songbird:${NC}"
echo "   ${BINARY_PATH}"
echo ""
echo -e "${BLUE}📚 For more options:${NC}"
echo "   ${BINARY_PATH} --help"
echo ""
echo -e "${BLUE}🔧 Installation:${NC} ${INSTALL_DIR}"
echo -e "${BLUE}👪 Family ID:${NC} ${FAMILY_ID}"
echo -e "${BLUE}🎯 Mode:${NC} ${DEPLOY_MODE}"
echo ""

exit 0

# Archive marker - DO NOT REMOVE
__GENOME_ARCHIVE__
WRAPPER_EOF
fi

# Combine wrapper and archive
mkdir -p "${OUTPUT_DIR}"
GENOME_PATH="${OUTPUT_DIR}/${GENOME_NAME}"

echo ""
echo -e "${BLUE}🧬 Creating genomeBin: ${GENOME_NAME}${NC}"

# Copy wrapper template
cp "${WRAPPER_TEMPLATE}" "${GENOME_PATH}"

# Append archive
cat "${ARCHIVE}" >> "${GENOME_PATH}"

# Make executable
chmod +x "${GENOME_PATH}"

# Calculate checksums
GENOME_SIZE=$(du -h "${GENOME_PATH}" | cut -f1)
GENOME_SHA256=$(sha256sum "${GENOME_PATH}" | cut -d' ' -f1)

# Success!
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                                ║${NC}"
echo -e "${GREEN}║        ✅ genomeBin CREATED SUCCESSFULLY! ✅                   ║${NC}"
echo -e "${GREEN}║                                                                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📦 Output:${NC}       ${GENOME_PATH}"
echo -e "${BLUE}📊 Size:${NC}         ${GENOME_SIZE}"
echo -e "${BLUE}🔑 SHA-256:${NC}      ${GENOME_SHA256}"
echo -e "${BLUE}🧬 Architectures:${NC} ${FOUND_COUNT} target(s)"
echo ""
echo -e "${BLUE}🚀 Usage:${NC}"
echo "   # Deploy locally"
echo "   ${GENOME_PATH}"
echo ""
echo "   # Deploy to USB"
echo "   cp ${GENOME_PATH} /media/usb/"
echo "   /media/usb/${GENOME_NAME}"
echo ""
echo "   # Deploy via curl"
echo "   curl https://biomeos.org/${GENOME_NAME} | sh"
echo ""
echo -e "${BLUE}🔍 Verify integrity:${NC}"
echo "   ${GENOME_PATH} --verify-only"
echo ""
echo -e "${GREEN}✨ TRUE genomeBin - Works everywhere! ✨${NC}"
echo ""
