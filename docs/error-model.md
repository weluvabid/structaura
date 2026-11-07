# Error model

StructAura rejects invalid source and invalid fingerprint inputs explicitly. It does not
return a best-effort fingerprint for a syntax tree that contains parser errors.

## Parsing

The JavaScript and TypeScript adapters use their respective Tree-sitter grammars. A parser
setup failure, cancelled parse, or syntax error is returned as a typed Rust error. Syntax
errors contain one or more diagnostics; each diagnostic exposes a message and a byte range
into the original UTF-8 source.

The browser API maps invalid source to a JavaScript `SyntaxError`. Invalid argument types,
unsupported language identifiers, use before `await init()`, and WASM/canonicalization
failures are surfaced as JavaScript errors. The API does not silently fall back to another
grammar or language.

Byte ranges are UTF-8 byte offsets, not JavaScript UTF-16 code-unit offsets or line/column
coordinates. Consumers that need editor positions must convert offsets using the source text.

## Canonicalization boundary

Browser parsing produces flat preorder syntax records. The Rust core validates and
reconstructs the records before encoding. Invalid child counts, malformed record streams,
invalid node kinds, empty or duplicate property keys, and unsupported language identifiers
fail rather than producing a partial fingerprint.

## Comparison

`compareFingerprints` returns `false` when two compatible fingerprints represent different
canonical structures. It returns `true` when their digests match under the same compatibility
identity. It throws/returns an error for mismatched format versions, languages, or
normalization versions. A digest match across different identities is not treated as
compatibility.

## Resource and execution model

Parsing and fingerprinting are synchronous after initialization. Large inputs can block the
calling thread; browser applications should move work to a Web Worker. The implementation
uses iterative application-level tree traversal to avoid recursion proportional to AST depth.
This does not remove the memory cost of parsing, storing syntax records, or constructing the
canonical representation.
