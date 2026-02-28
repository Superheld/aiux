#!/bin/sh
# Deploy AIUX Dateien auf den Raspi
# Usage: ./scripts/deploy.sh [user@host]
#
# Synct home/ nach /home/claude/ auf dem Raspi.
# API-Keys und config.yaml werden NICHT ueberschrieben.

set -e

HOST="${1:-root@192.168.178.57}"
TARGET="/home/claude"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Deploying to $HOST:$TARGET ..."

# Memory, Skills, Tools syncen
rsync -av "$PROJECT_DIR/home/memory/" "$HOST:$TARGET/memory/"
rsync -av "$PROJECT_DIR/home/skills/" "$HOST:$TARGET/skills/"
rsync -av "$PROJECT_DIR/home/tools/" "$HOST:$TARGET/tools/"

# Ownership fixen
ssh "$HOST" "chown -R claude:claude $TARGET/memory $TARGET/skills $TARGET/tools"

echo "Done. Vergiss nicht: lbu commit -d"
