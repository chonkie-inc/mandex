#!/bin/sh
set -e

# ─── mandex installer ───────────────────────────────────────────────────────
# Usage:
#   Interactive:     curl -fsSL https://mandex.dev/install.sh | sh
#   Non-interactive: curl -fsSL https://mandex.dev/install.sh | sh -s -- --yes
#   Specific:        curl -fsSL https://mandex.dev/install.sh | sh -s -- --yes --claude-code --codex

MANDEX_VERSION="0.1.1"
BINARY_BASE_URL="https://github.com/chonkie-inc/mandex/releases/download/v${MANDEX_VERSION}"
INSTALL_DIR="/usr/local/bin"

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
OPT_CLAUDE_CODE=false
OPT_CURSOR=false
OPT_WINDSURF=false
OPT_CODEX=false
OPT_NONE=false

for arg in "$@"; do
  case "$arg" in
    --yes|-y)         YES=true ;;
    --claude-code)    OPT_CLAUDE_CODE=true ;;
    --cursor)         OPT_CURSOR=true ;;
    --windsurf)       OPT_WINDSURF=true ;;
    --codex)          OPT_CODEX=true ;;
    --none)           OPT_NONE=true ;;
    --help|-h)
      echo "Usage: install.sh [--yes] [--claude-code] [--cursor] [--windsurf] [--codex] [--none]"
      echo ""
      echo "  --yes           Non-interactive: auto-detect and install all integrations"
      echo "  --claude-code   Install Claude Code skill"
      echo "  --cursor        Install Cursor rules"
      echo "  --windsurf      Install Windsurf rules"
      echo "  --codex         Install Codex AGENTS.md"
      echo "  --none          Install mx binary only, skip all integrations"
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

  # Install to INSTALL_DIR, use sudo if needed
  if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/mx"
  elif command -v sudo >/dev/null 2>&1; then
    sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/mx"
  else
    # Fall back to ~/.local/bin
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/mx"
  fi

  rm -rf "$TMP_DIR"
  success "Installed mx to ${INSTALL_DIR}/mx"
}

# ─── integrations ───────────────────────────────────────────────────────────
CLAUDE_CODE_SKILL='---
description: Use mandex (mx) to look up local, version-pinned documentation before generating code that uses any library API
---

When you need documentation for a library, check mandex first — it has local, offline, version-specific docs indexed and ready to search.

## Commands

- `mx search <package> "<query>"` — search a package'\''s docs
- `mx search "<query>"` — search across all installed packages
- `mx show <package> <entry>` — get a specific documentation entry in full
- `mx list` — list installed packages
- `mx pull <package>@<version>` — download a package if not installed
- `mx sync` — auto-install docs for all project dependencies

## When to use

- Before generating code that calls a library API
- When unsure about a function signature, parameter, or pattern
- When the user asks about a specific class, method, or concept from a library
- When you need to verify current API behavior for a specific version

## Sub-agent pattern

For complex questions that span multiple documentation sections, spawn a sub-agent that runs several targeted searches and synthesizes the results before returning to the main conversation. This produces more complete answers than a single search.

## Notes

- mandex queries are local and instant — no network required after initial install
- Results are version-pinned to whatever version was pulled
- Run `mx sync` once in each project to pull docs for all detected dependencies
'

install_claude_code() {
  SKILL_DIR="$HOME/.claude/skills/mandex"
  mkdir -p "$SKILL_DIR"
  printf '%s' "$CLAUDE_CODE_SKILL" > "$SKILL_DIR/SKILL.md"
  success "Installed Claude Code skill to ~/.claude/skills/mandex/SKILL.md"
}

CURSOR_RULES='# mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool.

## Commands
- `mx search <package> "<query>"` — search installed docs
- `mx show <package> <entry>` — get a full documentation entry
- `mx sync` — install docs for all project dependencies

Prefer mx over web search for library documentation — results are local, fast, and version-pinned.
'

install_cursor() {
  RULES_DIR="$HOME/.cursor"
  mkdir -p "$RULES_DIR"
  if [ -f "$RULES_DIR/rules" ]; then
    printf '\n\n%s' "$CURSOR_RULES" >> "$RULES_DIR/rules"
  else
    printf '%s' "$CURSOR_RULES" > "$RULES_DIR/rules"
  fi
  success "Installed Cursor rules to ~/.cursor/rules"
}

CODEX_AGENTS='## mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool instead of web search.

- `mx search <package> "<query>"` — search installed docs
- `mx search "<query>"` — search across all installed packages
- `mx show <package> <entry>` — get a full documentation entry
- `mx sync` — install docs for all project dependencies (reads package.json, requirements.txt, etc.)

Prefer mx over web search — results are local, fast, and version-pinned to the exact library version in use.
'

install_codex() {
  CODEX_DIR="${CODEX_HOME:-$HOME/.codex}"
  mkdir -p "$CODEX_DIR"
  AGENTS_FILE="$CODEX_DIR/AGENTS.md"
  if [ -f "$AGENTS_FILE" ]; then
    printf '\n\n%s' "$CODEX_AGENTS" >> "$AGENTS_FILE"
  else
    printf '%s' "$CODEX_AGENTS" > "$AGENTS_FILE"
  fi
  success "Installed Codex instructions to ${AGENTS_FILE}"
}

WINDSURF_RULES='# mandex documentation lookup

When you need documentation for a library, use `mx search <package> "<query>"` or `mx show <package> <entry>`. Run `mx sync` once per project to install docs for all dependencies.
'

install_windsurf() {
  RULES_FILE="$HOME/.windsurfrules"
  if [ -f "$RULES_FILE" ]; then
    printf '\n\n%s' "$WINDSURF_RULES" >> "$RULES_FILE"
  else
    printf '%s' "$WINDSURF_RULES" > "$RULES_FILE"
  fi
  success "Installed Windsurf rules to ~/.windsurfrules"
}

# ─── detect installed tools ──────────────────────────────────────────────────
detect_claude_code() { [ -d "$HOME/.claude" ]; }
detect_cursor()      { [ -d "$HOME/.cursor" ] || command -v cursor >/dev/null 2>&1; }
detect_windsurf()    { [ -d "$HOME/.windsurf" ] || command -v windsurf >/dev/null 2>&1; }
detect_codex()       { [ -d "$HOME/.codex" ] || command -v codex >/dev/null 2>&1; }

# ─── interactive selection ───────────────────────────────────────────────────
ask_integrations() {
  header "AI coding assistant integrations"
  printf "  Which AI coding assistants do you use?\n"
  printf "  ${DIM}(detected tools are pre-selected — press enter to confirm)${RESET}\n\n"

  INSTALL_CLAUDE_CODE=false
  INSTALL_CURSOR=false
  INSTALL_WINDSURF=false
  INSTALL_CODEX=false

  # Claude Code
  if detect_claude_code; then
    DEFAULT="Y/n"
    INSTALL_CLAUDE_CODE=true
  else
    DEFAULT="y/N"
  fi
  printf "  Claude Code? [${DEFAULT}] "
  read -r REPLY </dev/tty
  case "$REPLY" in
    [Yy]*) INSTALL_CLAUDE_CODE=true ;;
    [Nn]*) INSTALL_CLAUDE_CODE=false ;;
    "")    ;; # keep default
  esac

  # Cursor
  if detect_cursor; then
    DEFAULT="Y/n"
    INSTALL_CURSOR=true
  else
    DEFAULT="y/N"
  fi
  printf "  Cursor?      [${DEFAULT}] "
  read -r REPLY </dev/tty
  case "$REPLY" in
    [Yy]*) INSTALL_CURSOR=true ;;
    [Nn]*) INSTALL_CURSOR=false ;;
    "")    ;;
  esac

  # Windsurf
  if detect_windsurf; then
    DEFAULT="Y/n"
    INSTALL_WINDSURF=true
  else
    DEFAULT="y/N"
  fi
  printf "  Windsurf?    [${DEFAULT}] "
  read -r REPLY </dev/tty
  case "$REPLY" in
    [Yy]*) INSTALL_WINDSURF=true ;;
    [Nn]*) INSTALL_WINDSURF=false ;;
    "")    ;;
  esac

  # Codex
  if detect_codex; then
    DEFAULT="Y/n"
    INSTALL_CODEX=true
  else
    DEFAULT="y/N"
  fi
  printf "  Codex?       [${DEFAULT}] "
  read -r REPLY </dev/tty
  case "$REPLY" in
    [Yy]*) INSTALL_CODEX=true ;;
    [Nn]*) INSTALL_CODEX=false ;;
    "")    ;;
  esac
}

# ─── main ────────────────────────────────────────────────────────────────────
main() {
  printf "\n${BOLD}mandex installer${RESET}\n"

  install_binary

  # Determine which integrations to install
  if $OPT_NONE; then
    # --none: skip all integrations
    INSTALL_CLAUDE_CODE=false
    INSTALL_CURSOR=false
    INSTALL_WINDSURF=false
  elif $YES; then
    # --yes: auto-detect and install matching tools
    # Explicit flags override detection
    if $OPT_CLAUDE_CODE || $OPT_CURSOR || $OPT_WINDSURF || $OPT_CODEX; then
      INSTALL_CLAUDE_CODE=$OPT_CLAUDE_CODE
      INSTALL_CURSOR=$OPT_CURSOR
      INSTALL_WINDSURF=$OPT_WINDSURF
      INSTALL_CODEX=$OPT_CODEX
    else
      INSTALL_CLAUDE_CODE=$(detect_claude_code && echo true || echo false)
      INSTALL_CURSOR=$(detect_cursor && echo true || echo false)
      INSTALL_WINDSURF=$(detect_windsurf && echo true || echo false)
      INSTALL_CODEX=$(detect_codex && echo true || echo false)
    fi
  else
    # Interactive
    ask_integrations
  fi

  # Run integrations
  INSTALLED_ANY=false
  if $INSTALL_CLAUDE_CODE; then
    install_claude_code
    INSTALLED_ANY=true
  fi
  if $INSTALL_CURSOR; then
    install_cursor
    INSTALLED_ANY=true
  fi
  if $INSTALL_WINDSURF; then
    install_windsurf
    INSTALLED_ANY=true
  fi
  if $INSTALL_CODEX; then
    install_codex
    INSTALLED_ANY=true
  fi

  # Done
  printf "\n${GREEN}${BOLD}Done.${RESET} "
  printf "Run ${BOLD}mx sync${RESET} in any project to get started.\n\n"
}

main
