#!/bin/sh
# Deploy AIUX Dateien auf den Raspi
# Usage: ./scripts/deploy.sh [user@host]
#
# Synct home/ nach /home/claude/ auf dem Raspi.

set -e

HOST="${1:-root@192.168.178.57}"
TARGET="/home/claude"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Deploying to $HOST:$TARGET ..."

# Home-Verzeichnis syncen
rsync -av "$PROJECT_DIR/home/" "$HOST:$TARGET/"

# Ownership fixen
ssh "$HOST" "chown -R claude:claude $TARGET"

echo "Done. Vergiss nicht: lbu commit -d"
