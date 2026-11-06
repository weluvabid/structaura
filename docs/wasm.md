# WebAssembly package

The npm package uses a hybrid architecture: the official `web-tree-sitter` WebAssembly runtime parses JavaScript and TypeScript, while StructAura's Rust core performs canonical-tree reconstruction, canonical encoding, Merkle hashing and fingerprint identity construction. The browser package does not compile the native Tree-sitter Rust/C grammar crates.

## Consumer usage

```ts
import {
  init,
  fingerprintJavaScript,
  fingerprintTypeScript,
  compareFingerprints,
} from "structaura";

await init();
const left = fingerprintJavaScript("const total = left + right;");
const right = fingerprintJavaScript("// note\nconst total=(left+right)");
console.log(compareFingerprints(left, right));
```

`init()` is idempotent and must be awaited before fingerprint calls. The parser and Rust fingerprint operations are synchronous after initialization. The API is browser ESM; Node execution is supported for integration testing, not advertised as the production runtime. The npm manifest remains private under the original placeholder package name until a publishable name and license are selected.

## Build pipeline

- `structaura-wasm` depends only on `structaura-core`, `wasm-bindgen` and serialization glue.
- `web-tree-sitter` is pinned to 0.27.1. Its runtime `tree-sitter.wasm` is copied into the package assets.
- Grammar sources are cloned at fixed upstream tags and compiled with Tree-sitter CLI 0.27.1, ensuring parser ABI compatibility with the runtime. The CLI's WASI SDK integration is used for grammar compilation.
- Generated Rust bindings, the Rust WASM binary and grammar/runtime assets are written under `pkg/`, which is included in the published npm package.

Requirements: Rust toolchain from `rust-toolchain.toml`, `wasm32-unknown-unknown`, `wasm-pack`, Node.js 26+ and npm.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
npm install
npm run build
wasm-pack test --node crates/wasm
npm test
npm pack --dry-run
```

The browser runtime needs to fetch the Tree-sitter runtime WASM and both grammar WASM assets. The package's initialization configures the runtime asset path relative to the published ESM entry point. For large inputs, execute fingerprinting in a Web Worker to avoid blocking the browser main thread.
