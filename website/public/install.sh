#!/bin/sh
set -e

# ─── mandex installer ───────────────────────────────────────────────────────
# Usage:
#   Interactive:     curl -fsSL https://mandex.dev/install.sh | sh
#   Non-interactive: curl -fsSL https://mandex.dev/install.sh | sh -s -- --yes
#   Binary only:     curl -fsSL https://mandex.dev/install.sh | sh -s -- --none

MANDEX_VERSION="0.1.10"
BINARY_BASE_URL="https://github.com/chonkie-inc/mandex/releases/download/v${MANDEX_VERSION}"
INSTALL_DIR="$HOME/.local/bin"

# ─── colours ────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
  GREEN="\033[0;32m"
  DIM="\033[2m"
  BOLD="\033[1m"
  RESET="\033[0m"
else
  GREEN="" DIM="" BOLD="" RESET=""
fi

info()    { printf "  ${DIM}%s${RESET}\n" "$1"; }
success() { printf "  ${GREEN}✓${RESET} %s\n" "$1"; }
header()  { printf "\n${BOLD}%s${RESET}\n" "$1"; }
die()     { printf "\nError: %s\n" "$1" >&2; exit 1; }

# ─── parse arguments ────────────────────────────────────────────────────────
YES=false
OPT_NONE=false

for arg in "$@"; do
  case "$arg" in
    --yes|-y)  YES=true ;;
    --none)    OPT_NONE=true ;;
    --help|-h)
      echo "Usage: install.sh [--yes] [--none]"
      echo ""
      echo "  --yes   Non-interactive: auto-detect and install all integrations"
      echo "  --none  Install mx binary only, skip all integrations"
      exit 0
      ;;
  esac
done

# ─── detect platform ────────────────────────────────────────────────────────
detect_platform() {
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)

  case "$ARCH" in
    x86_64)  ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) die "Unsupported architecture: $ARCH" ;;
  esac

  case "$OS" in
    linux)
      # Detect musl vs gnu libc
      if command -v ldd >/dev/null 2>&1 && ldd /bin/sh 2>&1 | grep -qi musl; then
        LIBC="musl"
      else
        LIBC="gnu"
      fi
      TARGET="${ARCH}-unknown-linux-${LIBC}"
      BINARY_NAME="mx"
      ;;
    darwin)
      TARGET="${ARCH}-apple-darwin"
      BINARY_NAME="mx"
      ;;
    msys*|mingw*|cygwin*|windows*)
      TARGET="${ARCH}-pc-windows-msvc"
      BINARY_NAME="mx.exe"
      ;;
    *) die "Unsupported OS: $OS" ;;
  esac
}

# ─── install binary ─────────────────────────────────────────────────────────
install_binary() {
  header "Installing mandex (mx)"

  detect_platform

  DOWNLOAD_URL="${BINARY_BASE_URL}/mx-${TARGET}.tar.gz"
  TMP_DIR=$(mktemp -d)
  TMP_FILE="${TMP_DIR}/mx.tar.gz"

  info "Downloading mx v${MANDEX_VERSION} for ${TARGET}..."

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TMP_FILE" || die "Download failed: $DOWNLOAD_URL"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$TMP_FILE" || die "Download failed: $DOWNLOAD_URL"
  else
    die "Neither curl nor wget found. Please install one and retry."
  fi

  tar -xzf "$TMP_FILE" -C "$TMP_DIR"
  chmod +x "${TMP_DIR}/${BINARY_NAME}"

  mkdir -p "$INSTALL_DIR"
  mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/mx"

  rm -rf "$TMP_DIR"
  success "Installed mx to ${INSTALL_DIR}/mx"
}

# ─── PATH setup ───────────────────────────────────────────────────────────
ENV_FILE="$HOME/.local/bin/env.mandex"
ENV_SCRIPT='# mandex PATH setup
case ":${PATH}:" in
    *:"$HOME/.local/bin":*)
        ;;
    *)
        export PATH="$HOME/.local/bin:$PATH"
        ;;
esac'
SOURCE_LINE='. "$HOME/.local/bin/env.mandex"'

setup_path() {
  # Skip if already in PATH
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) return ;;
  esac

  # Write the env script
  mkdir -p "$(dirname "$ENV_FILE")"
  printf '%s\n' "$ENV_SCRIPT" > "$ENV_FILE"

  # Append source line to shell rc files
  MODIFIED_ANY=false

  # .profile — always write (POSIX fallback)
  _add_source_line "$HOME/.profile" true
  # .bashrc, .bash_profile — only if they exist
  _add_source_line "$HOME/.bashrc" false
  _add_source_line "$HOME/.bash_profile" false
  # .zshrc, .zshenv — detect zsh
  if _has_shell zsh; then
    ZSHENV="${ZDOTDIR:-$HOME}/.zshenv"
    _add_source_line "$ZSHENV" true
  fi

  if $MODIFIED_ANY; then
    success "Added ~/.local/bin to PATH in shell profile"
    info "Restart your shell or run: source ${ENV_FILE}"
  fi
}

_has_shell() {
  case "$SHELL" in *"$1"*) return 0 ;; esac
  command -v "$1" >/dev/null 2>&1
}

_add_source_line() {
  TARGET_FILE="$1"
  CREATE_IF_MISSING="$2"

  if [ -f "$TARGET_FILE" ]; then
    # Skip if source line already present
    case "$(cat "$TARGET_FILE")" in
      *"$SOURCE_LINE"*) return ;;
    esac
    # Ensure trailing newline before appending
    if [ -s "$TARGET_FILE" ] && [ "$(tail -c 1 "$TARGET_FILE" | wc -l)" -eq 0 ]; then
      printf '\n' >> "$TARGET_FILE"
    fi
    printf '%s\n' "$SOURCE_LINE" >> "$TARGET_FILE"
    MODIFIED_ANY=true
  elif $CREATE_IF_MISSING; then
    printf '%s\n' "$SOURCE_LINE" >> "$TARGET_FILE"
    MODIFIED_ANY=true
  fi
}

# ─── main ────────────────────────────────────────────────────────────────────
main() {
  printf "\n${BOLD}mandex installer${RESET}\n"

  install_binary
  setup_path

  # Run mx init for integrations
  if $OPT_NONE; then
    : # skip integrations
  elif $YES; then
    "${INSTALL_DIR}/mx" init --yes
  else
    "${INSTALL_DIR}/mx" init
  fi
}

main
