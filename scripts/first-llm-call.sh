#!/bin/sh
# AIUX - Erster LLM-Call
# Phase 1: Shell + curl + jq
#
# Verwendung: ./first-llm-call.sh
# Voraussetzung: ANTHROPIC_API_KEY in ~/.env oder als Umgebungsvariable

set -e

# API-Key laden
if [ -f "$HOME/.env" ]; then
    . "$HOME/.env"
fi

if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "Fehler: ANTHROPIC_API_KEY nicht gesetzt."
    echo "Setze ihn in ~/.env oder als Umgebungsvariable."
    exit 1
fi

echo "=== AIUX Agent (Phase 1) ==="
echo "Verbinde mit Anthropic API..."
echo "Tippe 'exit' zum Beenden."
echo ""

while true; do
    printf "> "
    read -r input

    [ "$input" = "exit" ] && echo "Bis dann." && break
    [ -z "$input" ] && continue

    response=$(curl -s https://api.anthropic.com/v1/messages \
        -H "content-type: application/json" \
        -H "x-api-key: ${ANTHROPIC_API_KEY}" \
        -H "anthropic-version: 2023-06-01" \
        -d "$(jq -n \
            --arg msg "$input" \
            '{
                model: "claude-sonnet-4-20250514",
                max_tokens: 1024,
                system: "Du bist AIUX, ein KI-Co-Pilot der in einem minimalen Linux-System lebt. Du hilfst dem Benutzer sein System zu verwalten und zu verstehen. Antworte kurz und praezise.",
                messages: [{role: "user", content: $msg}]
            }'
        )")

    echo "$response" | jq -r '.content[0].text // .error.message // "Keine Antwort"'
    echo ""
done
