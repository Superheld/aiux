#!/bin/sh
# Lokales aichat Setup fuer Entwicklung
# Usage: ./scripts/setup-local.sh
#
# Kopiert Role und ggf. Config-Vorlage nach ~/.config/aichat/

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
AICHAT_DIR="$HOME/.config/aichat"

# Roles-Verzeichnis anlegen und Soul syncen
mkdir -p "$AICHAT_DIR/roles"
cp "$PROJECT_DIR/home/memory/soul.md" "$AICHAT_DIR/roles/aiux.md"
echo "Role 'aiux' installiert."

# Config nur anlegen wenn noch keine existiert
if [ ! -f "$AICHAT_DIR/config.yaml" ]; then
    cp "$PROJECT_DIR/system/aichat/config.example.yaml" "$AICHAT_DIR/config.yaml"
    echo "Config angelegt - bitte API-Key eintragen in $AICHAT_DIR/config.yaml"
else
    echo "Config existiert bereits, nicht ueberschrieben."
fi

echo "Fertig. Teste mit: ./target/release/aichat -r aiux 'Wer bist du?'"
