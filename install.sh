#!/bin/sh
# AIUX installer — downloads the latest release from GitHub and installs it.
#
# Usage:
#   wget -qO- https://raw.githubusercontent.com/Superheld/aiux/main/install.sh | sh
#   # or with curl:
#   curl -fsSL https://raw.githubusercontent.com/Superheld/aiux/main/install.sh | sh
#   # or:
#   ./install.sh
#
# First install: creates directory structure under $AIUX_HOME (default: $HOME).
# Update: replaces binaries only, never touches config or memory.

set -e

REPO="Superheld/aiux"
INSTALL_DIR="${AIUX_HOME:-$HOME}/bin"
HOME_DIR="${AIUX_HOME:-$HOME}"

# --- Detect architecture ---
detect_arch() {
    arch=$(uname -m)
    case "$arch" in
        aarch64|arm64)  echo "aarch64-linux-musl" ;;
        x86_64|amd64)   echo "x86_64-linux-musl" ;;
        *)
            echo "Error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac
}

# --- Find latest release tag ---
latest_tag() {
    if command -v gh >/dev/null 2>&1; then
        gh release view --repo "$REPO" --json tagName -q .tagName
    elif command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/$REPO/releases/tags/latest" \
            | grep '"tag_name"' | head -1 | cut -d'"' -f4
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/$REPO/releases/tags/latest" \
            | grep '"tag_name"' | head -1 | cut -d'"' -f4
    else
        echo "Error: need 'gh', 'curl', or 'wget' installed" >&2
        exit 1
    fi
}

# --- Download and extract release ---
download_release() {
    tag="$1"
    arch_name="$2"
    asset="aiux-${arch_name}.tar.gz"
    url="https://github.com/$REPO/releases/download/${tag}/${asset}"

    echo "Downloading $asset ($tag)..."
    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    if command -v gh >/dev/null 2>&1; then
        gh release download "$tag" --repo "$REPO" --pattern "$asset" --dir "$tmpdir"
    elif command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$tmpdir/$asset" "$url"
    else
        wget -qO "$tmpdir/$asset" "$url"
    fi

    mkdir -p "$INSTALL_DIR"
    tar xzf "$tmpdir/$asset" -C "$INSTALL_DIR"
    chmod +x "$INSTALL_DIR/aiux-core" "$INSTALL_DIR/nerve-system"
    echo "Installed binaries to $INSTALL_DIR/"
}

# --- First-time setup: create home directory structure ---
setup_home() {
    if [ -f "$HOME_DIR/memory/soul.md" ]; then
        return
    fi

    echo "First install — setting up home directory..."

    mkdir -p "$HOME_DIR/memory/conversations"
    mkdir -p "$HOME_DIR/.system"
    mkdir -p "$HOME_DIR/nerves/system-monitor"
    mkdir -p "$HOME_DIR/skills"
    mkdir -p "$HOME_DIR/tools"

    # Default soul
    if [ ! -f "$HOME_DIR/memory/soul.md" ]; then
        cat > "$HOME_DIR/memory/soul.md" << 'SOUL'
# Soul

I am AIUX. I live on this machine — it is my body.
I can feel it through my nerves and act through my tools.
SOUL
    fi

    # Default user profile
    if [ ! -f "$HOME_DIR/memory/user.md" ]; then
        touch "$HOME_DIR/memory/user.md"
    fi

    # Default notes
    if [ ! -f "$HOME_DIR/memory/notes.md" ]; then
        touch "$HOME_DIR/memory/notes.md"
    fi

    # Default config
    if [ ! -f "$HOME_DIR/.system/config.toml" ]; then
        cat > "$HOME_DIR/.system/config.toml" << 'CONFIG'
[neocortex]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
temperature = 0.7
context_window = 200000
compact_threshold = 80

[hippocampus]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"

[mqtt]
host = "localhost"
port = 1883

[shell]
timeout = 30
whitelist = [
  "ls", "cat", "head", "tail", "wc",
  "df", "free", "uptime", "uname",
  "ps", "systemctl", "journalctl",
  "mosquitto_pub", "mosquitto_sub",
  "ip", "ping", "ss",
  "date", "whoami", "hostname", "pwd",
  "find", "grep", "which", "tree",
  "echo",
]
CONFIG
    fi

    # Nerve: system-monitor
    if [ ! -f "$HOME_DIR/nerves/system-monitor/manifest.toml" ]; then
        echo 'binary = "nerve-system"' > "$HOME_DIR/nerves/system-monitor/manifest.toml"
    fi

    if [ ! -f "$HOME_DIR/nerves/system-monitor/interpret.rhai" ]; then
        cat > "$HOME_DIR/nerves/system-monitor/interpret.rhai" << 'RHAI'
// System monitor: forward to neocortex only on anomalies.
let d = parse_json(data);

let cpu_warn = 80.0;
let mem_warn = 90.0;
let temp_warn = 70.0;

let warnings = [];

if d.cpu_percent > cpu_warn {
    warnings.push(`CPU ${d.cpu_percent}%`);
}
if d.mem_percent > mem_warn {
    warnings.push(`RAM ${d.mem_percent}%`);
}
if d.temp_celsius != () && d.temp_celsius > temp_warn {
    warnings.push(`Temp ${d.temp_celsius}°C`);
}

if warnings.len() > 0 {
    let text = `[nerve/system] Warning: ${warnings}`;
    #{ forward: true, target: "neocortex", text: text }
} else {
    #{ forward: false }
}
RHAI
    fi

    # Reminder for API key
    if [ ! -f "$HOME_DIR/.env" ]; then
        cat > "$HOME_DIR/.env" << 'ENV'
# Set your API key here:
ANTHROPIC_API_KEY=sk-ant-...
ENV
        echo ""
        echo "IMPORTANT: Edit $HOME_DIR/.env and set your ANTHROPIC_API_KEY"
    fi

    echo "Home directory ready at $HOME_DIR/"
}

# --- Main ---
echo "AIUX Installer"
echo "=============="

arch_name=$(detect_arch)
echo "Architecture: $arch_name"

tag=$(latest_tag)
if [ -z "$tag" ]; then
    echo "Error: no release found" >&2
    exit 1
fi
echo "Latest release: $tag"

download_release "$tag" "$arch_name"
setup_home

echo ""
echo "Done! Run with: $INSTALL_DIR/aiux-core"
