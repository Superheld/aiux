#!/bin/bash
set -euo pipefail

INPUT=$(cat)
FILE=$(echo "$INPUT" | jq -r '.input.file_path // empty')

[[ -n "$FILE" ]] || exit 0

# Sensitive file patterns
BLOCKED=(
  "*.env"
  "*.env.*"
  "*.pem"
  "*.key"
  "*credentials*"
  "*secret*"
)

# Protected directories (agent memory)
BLOCKED_DIRS=(
  "home/memory/"
  "/home/claude/memory/"
)

BASENAME=$(basename "$FILE")

for pattern in "${BLOCKED[@]}"; do
  case "$BASENAME" in
    $pattern)
      echo "Blocked: $FILE matches sensitive pattern ($pattern)" >&2
      exit 2
      ;;
  esac
done

for dir in "${BLOCKED_DIRS[@]}"; do
  if [[ "$FILE" == *"$dir"* ]]; then
    echo "Blocked: $FILE is in protected directory ($dir)" >&2
    exit 2
  fi
done

exit 0
