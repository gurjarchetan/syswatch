#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# SysWatch installer — downloads the correct pre-built binary and installs it.
# Supports: Linux (x86_64, arm64)  and  macOS (Apple Silicon M1/M2/M3/M4 only)
# Usage:  curl -fsSL https://raw.githubusercontent.com/gurjarchetan/syswatch/main/install.sh | bash
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

REPO="gurjarchetan/syswatch"
BINARY="syswatch"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# ── helpers ───────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()  { echo -e "${GREEN}[syswatch]${NC} $*"; }
warn()  { echo -e "${YELLOW}[syswatch]${NC} $*"; }
error() { echo -e "${RED}[syswatch] ERROR:${NC} $*" >&2; exit 1; }

need_cmd() { command -v "$1" &>/dev/null || error "required command not found: $1"; }

# ── detect OS ────────────────────────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux*)  echo "linux" ;;
    Darwin*) echo "macos" ;;
    *)       error "Unsupported OS: $(uname -s). Build from source instead." ;;
  esac
}

# ── detect architecture ───────────────────────────────────────────────────────
detect_arch() {
  local os="$1"
  local arch
  arch=$(uname -m)
  if [ "$os" = "linux" ]; then
    case "$arch" in
      x86_64)        echo "amd64" ;;
      aarch64|arm64) echo "arm64" ;;
      *)             error "Unsupported Linux architecture: $arch. Build from source instead." ;;
    esac
  else
    # macOS — Apple Silicon (M1/M2/M3/M4) only
    case "$arch" in
      arm64|aarch64) echo "aarch64" ;;
      x86_64) error "Intel Macs are not supported. Pre-built binaries are only available for Apple Silicon (M1/M2/M3/M4). Build from source instead: cargo install --git https://github.com/${REPO}" ;;
      *)      error "Unsupported macOS architecture: $arch. Build from source instead." ;;
    esac
  fi
}

detect_pkg_manager() {
  if command -v dpkg &>/dev/null && command -v apt-get &>/dev/null; then
    echo "deb"
  else
    echo "tar"
  fi
}

# ── fetch latest release version ─────────────────────────────────────────────
fetch_latest_version() {
  local url="https://api.github.com/repos/${REPO}/releases/latest"
  local version
  if command -v curl &>/dev/null; then
    version=$(curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
  elif command -v wget &>/dev/null; then
    version=$(wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
  else
    error "Neither curl nor wget found. Please install one and retry."
  fi
  [ -n "$version" ] || error "Could not determine latest version. Check your internet connection."
  echo "$version"
}

download() {
  local url="$1" dest="$2"
  if command -v curl &>/dev/null; then
    curl -fsSL --progress-bar -o "$dest" "$url"
  else
    wget -q --show-progress -O "$dest" "$url"
  fi
}

# ── install tar.gz binary to INSTALL_DIR ─────────────────────────────────────
install_tarball() {
  local tarball="$1" version="$2"
  local url="https://github.com/${REPO}/releases/download/${version}/${tarball}"
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT

  info "Downloading $tarball ..."
  download "$url" "$tmp/$tarball"

  info "Extracting..."
  tar -xzf "$tmp/$tarball" -C "$tmp"

  mkdir -p "$INSTALL_DIR"
  install -m 755 "$tmp/$BINARY" "$INSTALL_DIR/$BINARY"

  # PATH check
  if ! echo ":${PATH}:" | grep -q ":${INSTALL_DIR}:"; then
    warn "$INSTALL_DIR is not in your PATH."
    warn "Add this line to your shell config (~/.zshrc, ~/.bashrc, etc.):"
    warn ""
    warn "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    warn ""
  fi

  info "SysWatch $version installed to $INSTALL_DIR/$BINARY"
  info "Run: syswatch"
}

# ── main ──────────────────────────────────────────────────────────────────────
main() {
  info "Detecting system..."
  OS=$(detect_os)
  ARCH=$(detect_arch "$OS")
  VERSION=$(fetch_latest_version)
  VERSION_NUM="${VERSION#v}"

  info "Latest version: $VERSION  |  OS: $OS  |  Arch: $ARCH"

  if [ "$OS" = "macos" ]; then
    # ── macOS: always use tar.gz ───────────────────────────────────────────
    TARBALL="${BINARY}-${VERSION_NUM}-macos-${ARCH}.tar.gz"
    install_tarball "$TARBALL" "$VERSION"
    return
  fi

  # ── Linux ─────────────────────────────────────────────────────────────────

  # Prefer .deb on Debian/Ubuntu amd64
  PKG=$(detect_pkg_manager)
  if [ "$PKG" = "deb" ] && [ "$ARCH" = "amd64" ]; then
    DEB_FILE="${BINARY}_${VERSION_NUM}_${ARCH}.deb"
    DEB_URL="https://github.com/${REPO}/releases/download/${VERSION}/${DEB_FILE}"
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT

    info "Downloading $DEB_FILE ..."
    download "$DEB_URL" "$TMP/$DEB_FILE"

    info "Installing via dpkg (requires sudo)..."
    sudo dpkg -i "$TMP/$DEB_FILE"
    info "Installed! Run: syswatch"
    return
  fi

  # Fallback: tar.gz → ~/.local/bin
  TARBALL="${BINARY}-${VERSION_NUM}-linux-${ARCH}.tar.gz"
  install_tarball "$TARBALL" "$VERSION"
}

main "$@"

