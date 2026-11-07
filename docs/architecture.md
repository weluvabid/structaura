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

| Crate / path           | Responsibility                                                                 |
|------------------------|---------------------------------------------------------------------------------|
| `crates/core`          | Language-neutral canonical representation, flat syntax-record reconstruction, deterministic byte encoding, Merkle hashing, fingerprint identity and compatibility checks |
| `crates/javascript`    | Native Tree-sitter parsing, diagnostics, and language-specific extraction of flat syntax records |
| `crates/typescript`    | Native Tree-sitter parsing, diagnostics, and language-specific extraction of flat syntax records |
| `crates/api`           | Unified native Rust API                                                        |
| `crates/wasm`          | Shared Rust canonicalization and hashing functions via `wasm-bindgen` (does not link native Tree-sitter C grammar crates) |
| `src/index.js`         | Browser parser initialization, syntax diagnostics, and extraction of the same flat syntax-record contract |

The browser package uses the official `web-tree-sitter` runtime with grammar WASM modules built from pinned upstream grammar revisions. The browser and native paths deliberately have separate parser adapters but converge on the same core record contract; this keeps parser/runtime concerns out of the fingerprint format implementation.

## Flat syntax boundary

Parser adapters emit preorder records containing:

- a syntax kind
- optional token or string-literal content
- a count of retained direct children

The Rust core reconstructs the canonical tree in reverse preorder using an explicit vector. This boundary is non-recursive and serializes without nested JSON objects, which avoids recursive deserialization and keeps canonicalization and hashing identical across native and browser execution. Malformed record streams fail explicitly. The core validates the preorder stream before constructing canonical nodes; it does not accept truncated trees or silently discard unconsumed records.

### Normalization policy (current)

The extraction policy:

- preserves child order
- omits comments and selected anonymous punctuation
- strips string delimiters without decoding escapes
- makes a single-expression parenthesized wrapper transparent

These are **normalization rules**, not claims of semantic equivalence. Changes to these rules require a normalization-version increment.

## Hashing

Node encodings use:

- explicit domain tags
- fixed-width integer encodings
- length-prefixed byte strings
- typed property values

A node hash incorporates its canonical encoding and ordered child hashes. The root fingerprint adds a separate root domain tag and compatibility identity. Canonicalization and Merkle traversal are iterative to avoid recursive application-level traversal over deeply nested syntax.

## Runtime and failure behavior

`init()` initializes the Rust WASM module, Tree-sitter runtime, and both grammar modules once. Fingerprinting is synchronous after initialization.

The following conditions are surfaced as errors:

- parser cancellation
- syntax errors
- unsupported language identifiers
- invalid flat records
- incompatible fingerprint comparisons

Consumers handling large files should call the synchronous API from a Web Worker.

## Determinism and compatibility

The canonical representation sorts property keys, preserves child order, and uses typed
property values. Encoding is unambiguous: values are length-prefixed, integers use fixed-width
big-endian representation, and node/root hashes use distinct domain tags. The root hash
includes the fingerprint format version, language identifier, and normalization version.

The format version governs the canonical byte encoding and fingerprint envelope. The
normalization version governs language-adapter transformations. Keeping these version axes
separate allows a normalization-policy change to be identified without conflating it with a
serialization change.
