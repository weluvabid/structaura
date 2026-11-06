# Architecture

StructAura has two parser front ends and one shared Rust fingerprinting core:

```text
Native Rust path                         Browser ESM path
────────────────                         ────────────────
source                                   source
  │                                        │
Tree-sitter Rust bindings                 web-tree-sitter WASM runtime
  │                                        │
flat preorder syntax records ◀────────────┘
  │
structaura-core canonicalization
  │
Canonical IR → deterministic encoding → SHA-256 Merkle hash
  │
versioned fingerprint
```

## Workspace boundaries

- `crates/core` owns the language-neutral canonical representation, flat syntax-record reconstruction, deterministic byte encoding, Merkle hashing, fingerprint identity and compatibility checks.
- `crates/javascript` and `crates/typescript` own native Tree-sitter parsing, diagnostics and language-specific extraction of flat syntax records.
- `crates/api` exposes the unified native Rust API.
- `crates/wasm` exposes the shared Rust canonicalization and hashing functions through `wasm-bindgen`; it does not link the native Tree-sitter C grammar crates.
- `src/index.js` owns browser parser initialization, syntax diagnostics and extraction of the same flat syntax-record contract. The official `web-tree-sitter` runtime parses JavaScript and TypeScript using grammar WASM modules built from pinned upstream grammar revisions.

## Flat syntax boundary

The parser adapters emit preorder records containing a syntax kind, optional token or string-literal content, and a count of retained direct children. The Rust core reconstructs the canonical tree in reverse preorder using an explicit vector. This boundary is non-recursive and serializes without nested JSON objects, avoiding recursive deserialization and keeping canonicalization and hashing identical across native and browser execution. Malformed record streams fail explicitly.

The extraction policy preserves child order, omits comments and selected anonymous punctuation, strips string delimiters without decoding escapes, and makes a single-expression parenthesized wrapper transparent. These are normalization rules, not claims of semantic equivalence. Changes to these rules require a normalization-version increment.

## Hashing

Node encodings use explicit domain tags, fixed-width integer encodings, length-prefixed byte strings and typed property values. A node hash incorporates its canonical encoding and ordered child hashes. The root fingerprint adds a separate root domain tag and compatibility identity. Canonicalization and Merkle traversal are iterative to avoid recursive application-level traversal over deeply nested syntax.

## Runtime and failure behavior

`init()` initializes the Rust WASM module, Tree-sitter runtime and both grammar modules once. Fingerprinting is synchronous after initialization. Parser cancellation, syntax errors, unsupported language identifiers, invalid flat records and incompatible fingerprint comparisons are surfaced as errors. Consumers handling large files should call the synchronous API from a Web Worker.
