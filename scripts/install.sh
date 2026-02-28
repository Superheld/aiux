#!/bin/sh
# AIUX Installer
# Usage: ./scripts/install.sh [--local]
#
# Richtet das Zielsystem fuer AIUX ein:
# - Prueft Voraussetzungen
# - Legt User und Verzeichnisse an
# - Fragt nach API Key
#
# --local: Binaries aus lokalem Build statt Download verwenden

set -e

# --- Konfiguration ---

AIUX_USER="claude"
AIUX_HOME="/home/$AIUX_USER"
AIUX_ENV="$AIUX_HOME/.env"

for arg in "$@"; do
    case "$arg" in
        --local) ;; # reserviert fuer spaeter
    esac
done

# --- Hilfsfunktionen ---

info()  { echo "  [+] $1"; }
warn()  { echo "  [!] $1"; }
error() { echo "  [x] $1" >&2; exit 1; }

detect_pkg_manager() {
    if command -v apk >/dev/null 2>&1; then
        echo "apk"
    elif command -v apt-get >/dev/null 2>&1; then
        echo "apt"
    elif command -v pacman >/dev/null 2>&1; then
        echo "pacman"
    else
        echo "unknown"
    fi
}

# --- Checks ---

echo ""
echo "AIUX Installer"
echo "=============="
echo ""

if [ "$(id -u)" -ne 0 ]; then
    error "Bitte als root ausfuehren (sudo ./scripts/install.sh)"
fi

info "System: $(uname -s) $(uname -m)"
PKG=$(detect_pkg_manager)
info "Paketmanager: $PKG"

if [ "$PKG" = "unknown" ]; then
    warn "Unbekannter Paketmanager. Pakete muessen manuell installiert werden."
fi

# --- User anlegen ---

echo ""
if id "$AIUX_USER" >/dev/null 2>&1; then
    info "User '$AIUX_USER' existiert bereits."
else
    info "Lege User '$AIUX_USER' an..."
    case "$PKG" in
        apk)    adduser -D -s /bin/bash "$AIUX_USER" ;;
        apt)    useradd -m -s /bin/bash "$AIUX_USER" ;;
        pacman) useradd -m -s /bin/bash "$AIUX_USER" ;;
        *)      useradd -m -s /bin/bash "$AIUX_USER" ;;
    esac
    info "User '$AIUX_USER' angelegt."
fi

# --- Verzeichnisstruktur ---

echo ""
info "Richte Verzeichnisse ein..."

mkdir -p "$AIUX_HOME/memory/context"
mkdir -p "$AIUX_HOME/memory/journal"
mkdir -p "$AIUX_HOME/skills"
mkdir -p "$AIUX_HOME/tools"

# Default soul.md kopieren wenn nicht vorhanden
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

if [ ! -f "$AIUX_HOME/memory/soul.md" ]; then
    if [ -f "$PROJECT_DIR/home/memory/soul.md" ]; then
        cp "$PROJECT_DIR/home/memory/soul.md" "$AIUX_HOME/memory/soul.md"
        info "Default soul.md kopiert."
    else
        warn "Keine soul.md gefunden. Agent hat keine Persoenlichkeit."
    fi
else
    info "soul.md existiert bereits (wird nicht ueberschrieben)."
fi

if [ ! -f "$AIUX_HOME/memory/user.md" ]; then
    if [ -f "$PROJECT_DIR/home/memory/user.md" ]; then
        cp "$PROJECT_DIR/home/memory/user.md" "$AIUX_HOME/memory/user.md"
        info "Default user.md kopiert."
    fi
fi

chown -R "$AIUX_USER:$AIUX_USER" "$AIUX_HOME"
info "Verzeichnisse bereit."

# --- API Key ---

echo ""
if [ -f "$AIUX_ENV" ] && grep -q "ANTHROPIC_API_KEY" "$AIUX_ENV"; then
    info "API Key ist bereits konfiguriert."
else
    printf "  Anthropic API Key (Enter zum Ueberspringen): "
    read -r api_key
    if [ -n "$api_key" ]; then
        echo "ANTHROPIC_API_KEY=$api_key" > "$AIUX_ENV"
        chmod 600 "$AIUX_ENV"
        chown "$AIUX_USER:$AIUX_USER" "$AIUX_ENV"
        info "API Key gespeichert in $AIUX_ENV"
    else
        warn "Kein API Key gesetzt. Setze ihn spaeter in $AIUX_ENV"
    fi
fi

# --- Zusammenfassung ---

echo ""
echo "Fertig!"
echo ""
echo "  User:    $AIUX_USER"
echo "  Home:    $AIUX_HOME"
echo "  Soul:    $AIUX_HOME/memory/soul.md"
echo "  Env:     $AIUX_ENV"
echo ""
echo "Naechste Schritte:"
echo "  1. aiux-core bauen: cargo build --release -p aiux-core"
echo "  2. Binary kopieren: cp target/release/aiux-core /usr/local/bin/"
echo "  3. Starten: su - $AIUX_USER -c 'aiux-core'"
echo ""
