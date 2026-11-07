#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null
git -C "$ROOT" config core.hooksPath .githooks

echo "Configured .githooks as the repository hooks directory."
echo "The pre-commit hook runs ./scripts/check.sh (fmt, Clippy, workspace tests)."
