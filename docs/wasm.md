# WebAssembly package

The npm package uses a hybrid architecture:

- The official `web-tree-sitter` WebAssembly runtime parses JavaScript and TypeScript.
- StructAura’s Rust core performs canonical-tree reconstruction, canonical encoding, Merkle hashing, and fingerprint identity construction.

The browser package does **not** compile the native Tree-sitter Rust/C grammar crates.

## Consumer usage

```ts
import {
  init,
  fingerprintJavaScript,
  fingerprintTypeScript,
  compareFingerprints,
} from "@weluvabid/structaura";

await init();

const left = fingerprintJavaScript("const total = left + right;");
const right = fingerprintJavaScript("// note\nconst total=(left+right)");
console.log(compareFingerprints(left, right));
```

- `init()` is idempotent and **must** be awaited before any fingerprint calls.
- Parser and Rust fingerprint operations are synchronous after initialization.
- The published API is browser ESM. Node execution is supported for integration testing only; it is not the advertised production runtime.
- Package name: `@weluvabid/structaura`
- License: dual-licensed under MIT OR Apache-2.0

## Build pipeline

| Component              | Notes                                                                 |
|------------------------|-----------------------------------------------------------------------|
| `structaura-wasm`      | Depends only on `structaura-core`, `wasm-bindgen`, and serialization glue |
| `web-tree-sitter`      | Pinned to 0.27.1. Runtime `tree-sitter.wasm` is copied into package assets |
| Grammars               | Cloned at fixed upstream tags and compiled with Tree-sitter CLI 0.27.1 (ABI-compatible with the runtime). Uses the CLI’s WASI SDK integration |
| Output                 | Generated Rust bindings, Rust WASM binary, and grammar/runtime assets are written under `pkg/` (included in the published package) |

**Requirements**

- Rust toolchain from `rust-toolchain.toml`
- `wasm32-unknown-unknown` target
- `wasm-pack`
- Node.js 26 and npm
- Git (for grammar generation)

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
npm install
npm run build
wasm-pack test --node crates/wasm
npm test
npm pack --dry-run
```

## Runtime notes

The browser runtime needs to fetch:

- the Tree-sitter runtime WASM
- both grammar WASM assets

Initialization configures the runtime asset path relative to the published ESM entry point.

For large inputs, execute fingerprinting in a Web Worker to avoid blocking the browser main thread.

## Reproducibility and generated assets

`scripts/build-grammars.sh` fetches the JavaScript grammar at `v0.23.1` and the TypeScript
grammar at `v0.23.2`, then compiles them with the pinned Tree-sitter CLI dependency. The
`web-tree-sitter` runtime is pinned in `package-lock.json`. These versions are part of the
parser compatibility surface: grammar updates can change syntax trees and therefore
fingerprints even when StructAura's Rust core is unchanged.

The build script uses a temporary checkout and writes generated WASM assets and upstream
license notices under `pkg/`. Review grammar/runtime upgrades with normalization-contract
tests before changing pins. Do not edit generated `pkg/` assets by hand.
