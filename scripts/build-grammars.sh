#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT INT TERM
mkdir -p "$ROOT/pkg/grammars/licenses"
# npm publishes the runtime asset under its package-qualified filename.
# Keep StructAura's internal asset name stable for its locateFile mapping.
RUNTIME_WASM="$ROOT/node_modules/web-tree-sitter/web-tree-sitter.wasm"
if [ ! -s "$RUNTIME_WASM" ]; then
  echo "error: web-tree-sitter runtime WASM not found at: $RUNTIME_WASM" >&2
  echo "       verify web-tree-sitter@0.27.1 is installed" >&2
  exit 1
fi
cp "$RUNTIME_WASM" "$ROOT/pkg/grammars/tree-sitter.wasm"
cp "$ROOT/node_modules/web-tree-sitter/LICENSE" "$ROOT/pkg/grammars/licenses/web-tree-sitter-MIT.txt"

# Build pinned upstream grammar revisions with the same CLI generation used by web-tree-sitter.
git clone --depth 1 --branch v0.23.1 https://github.com/tree-sitter/tree-sitter-javascript.git "$WORK/javascript"
git clone --depth 1 --branch v0.23.2 https://github.com/tree-sitter/tree-sitter-typescript.git "$WORK/typescript"
cp "$WORK/javascript/LICENSE" "$ROOT/pkg/grammars/licenses/tree-sitter-javascript-MIT.txt"
cp "$WORK/typescript/LICENSE" "$ROOT/pkg/grammars/licenses/tree-sitter-typescript-MIT.txt"

cd "$ROOT"
"$ROOT/node_modules/.bin/tree-sitter" build --wasm \
  --output "$ROOT/pkg/grammars/tree-sitter-javascript.wasm" "$WORK/javascript"
test -s "$ROOT/pkg/grammars/tree-sitter-javascript.wasm"

"$ROOT/node_modules/.bin/tree-sitter" build --wasm \
  --output "$ROOT/pkg/grammars/tree-sitter-typescript.wasm" "$WORK/typescript/typescript"
test -s "$ROOT/pkg/grammars/tree-sitter-typescript.wasm"
