#!/bin/sh
# Activate git hooks from hooks/ directory.
# Run once after cloning.

git config core.hooksPath hooks
echo "Git hooks active (hooks/)"
