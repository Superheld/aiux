#!/bin/sh
# AIUX Image Builder
# Baut ein flashbares SD-Karten-Image aus Alpine + AIUX-Overlay.
#
# Verwendung: sudo ./build-image.sh [output.img]
#
# TODO: Noch nicht implementiert - Platzhalter für Phase 1

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/alpine.conf"

OUTPUT="${1:-${SCRIPT_DIR}/output/aiux-${ALPINE_VERSION}.img}"

echo "=== AIUX Image Builder ==="
echo "Alpine: ${ALPINE_VERSION} (${ALPINE_ARCH})"
echo "Output: ${OUTPUT}"
echo ""
echo "TODO: Image-Build noch nicht implementiert."
echo "Aktuell: Manuelle Installation gemäss docs/ROADMAP.md"
