#!/bin/bash
set -euo pipefail

# GitHub Action installation script for mdbook-lint
# Downloads and installs prebuilt binaries for the current platform

REPO="joshrotenberg/mdbook-lint"
INSTALL_DIR="${HOME}/.local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            os="windows"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            if [[ "$os" == "macos" ]]; then
                arch="aarch64"
            else
                log_error "ARM64 only supported on macOS"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    # Determine binary name and extension
    local binary_name="mdbook-lint"
    local file_ext=""
    
    if [[ "$os" == "windows" ]]; then
        file_ext=".exe"
        binary_name="${binary_name}${file_ext}"
    fi
    
    # Determine asset name based on platform
    case "$os-$arch" in
        linux-x86_64)
            # Prefer musl for better compatibility
            ASSET_NAME="mdbook-lint-linux-x86_64-musl"
            ;;
        macos-x86_64)
            ASSET_NAME="mdbook-lint-macos-x86_64"
            ;;
        macos-aarch64)
            ASSET_NAME="mdbook-lint-macos-aarch64"
            ;;
        windows-x86_64)
            ASSET_NAME="mdbook-lint-windows-x86_64.exe"
            ;;
        *)
            log_error "No prebuilt binary available for $os-$arch"
            exit 1
            ;;
    esac
    
    BINARY_NAME="$binary_name"
    log_info "Detected platform: $os-$arch"
    log_info "Asset name: $ASSET_NAME"
}

# Get latest release version
get_version() {
    local version="${VERSION:-latest}"
    
    if [[ "$version" == "latest" ]]; then
        log_info "Fetching latest release version..."
        version=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
        
        if [[ -z "$version" ]]; then
            log_error "Failed to get latest version"
            exit 1
        fi
    fi
    
    RELEASE_VERSION="$version"
    log_info "Using version: $RELEASE_VERSION"
}

# Download and install binary
install_binary() {
    local download_url="https://github.com/$REPO/releases/download/$RELEASE_VERSION/$ASSET_NAME"
    local temp_file
    
    log_info "Downloading from: $download_url"
    
    # Create temp file
    temp_file=$(mktemp)
    trap "rm -f '$temp_file'" EXIT
    
    # Download binary
    if ! curl -sL "$download_url" -o "$temp_file"; then
        log_error "Failed to download $ASSET_NAME"
        log_error "URL: $download_url"
        exit 1
    fi
    
    # Verify download
    if [[ ! -s "$temp_file" ]]; then
        log_error "Downloaded file is empty"
        exit 1
    fi
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Install binary
    local install_path="$INSTALL_DIR/$BINARY_NAME"
    cp "$temp_file" "$install_path"
    chmod +x "$install_path"
    
    log_info "Installed to: $install_path"
    
    # Add to PATH for this action
    echo "$INSTALL_DIR" >> "$GITHUB_PATH"
    
    # Verify installation with more detailed error reporting
    if "$install_path" --version >/dev/null 2>&1; then
        log_info "Installation successful!"
        "$install_path" --version
    else
        log_warn "Installation verification failed, but binary exists. This may be due to version incompatibility."
        # Check if binary exists and is executable
        if [[ -x "$install_path" ]]; then
            log_info "Binary is executable, attempting to continue..."
            # Try running without capturing stderr to see what the error is
            echo "Debug: attempting to run binary directly:"
            "$install_path" --version || log_warn "Binary execution failed, but proceeding"
        else
            log_error "Binary is not executable or doesn't exist"
            exit 1
        fi
    fi
}

# Check if already installed
check_existing() {
    if command -v mdbook-lint >/dev/null 2>&1; then
        local current_version
        current_version=$(mdbook-lint --version 2>/dev/null | head -1 || echo "unknown")
        log_info "mdbook-lint already available: $current_version"
        
        # If version matches what we want, skip installation
        if [[ "$current_version" == *"$RELEASE_VERSION"* ]] || [[ "$VERSION" == "latest" ]]; then
            log_info "Using existing installation"
            return 0
        fi
    fi
    return 1
}

main() {
    log_info "Installing mdbook-lint for GitHub Actions..."
    
    detect_platform
    get_version
    
    if ! check_existing; then
        install_binary
    fi
    
    log_info "mdbook-lint is ready to use!"
}

main "$@"