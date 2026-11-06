# Structaura

Deterministic AST fingerprinting for JavaScript and TypeScript, implemented in Rust and exposed to browser JavaScript through WebAssembly.

Structaura parses source into a syntax tree, normalizes selected syntax-only differences, and computes a versioned SHA-256 Merkle fingerprint. Formatting and comments can change without changing the fingerprint; structural changes should change it.

> **Scope:** Structaura is a structural fingerprinting tool, not a JavaScript runtime, type checker, semantic-equivalence prover, or source formatter.

## Features

- JavaScript and TypeScript parsing with Tree-sitter.
- Deterministic canonical representation and domain-separated SHA-256 hashing.
- Explicit fingerprint-format and language-normalization versions.
- Compatibility checks that reject fingerprints from different languages or versions.
- Syntax errors surfaced as JavaScript `Error` objects with useful diagnostics.
- Browser-facing WebAssembly package backed by the same Rust implementation.
- Iterative AST canonicalization and Merkle traversal to avoid recursive traversal in the application code.

## Install

```sh
npm install structaura
```

Structaura currently ships an ESM browser package built for `wasm32-unknown-unknown`. It is intended for browsers and browser bundlers. The package is not a native Node.js implementation.

## Quick start

```ts
import {
  init,
  fingerprintJavaScript,
  fingerprintTypeScript,
  compareFingerprints,
} from "structaura";

await init();

const before = fingerprintJavaScript(`
  // A comment does not affect the fingerprint.
  const total = left + right;
`);

const after = fingerprintJavaScript("const total=(left+right)");

console.log(before.value === after.value); // true
console.log(before.digest);                // 64-character lowercase SHA-256 hex
console.log(before.language);              // "javascript"
console.log(before.formatVersion);         // 1
console.log(before.normalizationVersion);  // 1

const compatible = compareFingerprints(before, after);
console.log(compatible); // true
```

TypeScript source uses the TypeScript grammar:

```ts
const typed = fingerprintTypeScript("const count: number = 3;");
console.log(typed.value);
```

The generic entry point is also available:

```ts
import { init, fingerprintSource } from "structaura";

await init();

const result = fingerprintSource("const enabled = true;", "javascript");
// Supported language identifiers: "javascript", "typescript".
```

Initialization is asynchronous; fingerprint operations are synchronous after `await init()`. For large inputs, run fingerprinting in a Web Worker so parsing does not block the UI thread.

## Fingerprint contract

A fingerprint contains:

- `value`: complete versioned fingerprint string.
- `digest`: SHA-256 digest as 64 lowercase hexadecimal characters.
- `language`: `javascript` or `typescript`.
- `formatVersion`: fingerprint serialization format version.
- `normalizationVersion`: language-specific canonicalization version.

`compareFingerprints(left, right)` returns `true` or `false` for compatible fingerprints. It throws when format, language, or normalization versions differ. Do not compare only `digest` across different identities.

See [the fingerprint contract](docs/fingerprint-contract.md) for the compatibility model and normalization boundaries.

## Architecture

The Rust workspace is split by responsibility, with browser parsing handled by the official Tree-sitter WebAssembly runtime:

| Crate | Responsibility |
| --- | --- |
| `structaura-core` | Canonical IR, deterministic encoding, Merkle hashing and fingerprint identity |
| `structaura-javascript` | JavaScript parsing, diagnostics and canonicalization |
| `structaura-typescript` | TypeScript parsing, diagnostics and canonicalization |
| `structaura` | Unified Rust API and language dispatch |
| `structaura-wasm` | Shared Rust canonicalization, Merkle hashing and wasm-bindgen API |

The browser adapter uses `web-tree-sitter` to parse source and sends a flat preorder syntax representation into Rust WASM. Native Rust adapters and the browser package both use `structaura-core` for canonicalization and hashing. See [architecture](docs/architecture.md) and [WebAssembly package development](docs/wasm.md).

## Development

Requirements:

- Rust toolchain specified in `rust-toolchain.toml`.
- `wasm32-unknown-unknown` Rust target.
- Node.js 26+ and npm.
- `wasm-pack`.

Run native checks:

```sh
./scripts/check.sh
```

Install the pinned JavaScript build dependencies, then build and test the browser package:

```sh
npm install
npm run build
wasm-pack test --node crates/wasm
npm test
```

Build the browser package:

```sh
npm run build
```

The generated `pkg/` directory is intentionally ignored by Git. It is included in the npm package produced during publishing.

## Benchmarks

Run the end-to-end JavaScript and TypeScript fingerprinting benchmark in release mode:

```sh
./scripts/bench.sh
```

The harness measures the public API across small, medium, and large generated source files. It reports source size, median latency, p95 sample latency, and input throughput. Each reported latency is the per-operation average within a sample; the median and p95 are calculated across 15 samples after five warm-up operations.

For comparable results, record `rustc -Vv`, the operating system, and CPU model, and run on an otherwise idle machine. Compare results from the same machine and toolchain. These numbers measure the full parse-to-fingerprint path; they are not isolated parser or hashing timings.
