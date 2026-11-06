# Fingerprint contract

## Identity

A Structaura fingerprint is not just a digest. Its compatibility identity is:

- fingerprint format version;
- source language identifier;
- language-specific normalization version.

The rendered value has this shape:

```text
structaura:v<format-version>:<language>:n<normalization-version>:<64-hex-digest>
```

`compareFingerprints` rejects comparisons when any identity component differs. A digest comparison without checking identity is not a supported compatibility check.

## Determinism

For the same source, language adapter and implementation versions, Structaura produces the same fingerprint. The canonical encoding uses:

- domain-separated node and root hashes;
- length-prefixed byte sequences;
- fixed-width big-endian integer encodings;
- sorted property keys;
- ordered child hashes;
- SHA-256.

## Normalization version 1

The current adapters intentionally normalize a limited set of syntax differences, including:

- comments;
- selected punctuation used for formatting or delimiters;
- redundant parenthesized-expression wrappers;
- quote delimiters for simple string literals.

They preserve syntax node kinds, leaf tokens, string contents and child order. This is a syntax-tree normalization policy, not a complete JavaScript or TypeScript semantic model.

Do not assume that all semantically equivalent source produces the same fingerprint. For example, escape-sequence equivalence, identifier binding equivalence, constant folding, type inference, control-flow equivalence and runtime behaviour are not normalized.

Likewise, a matching fingerprint is evidence that the current canonical representations match; it is not a proof that two programs behave identically.

## Versioning rules

Any change that can alter canonical output for an existing input must increment that language adapter's normalization version. Any incompatible change to the canonical byte encoding or fingerprint envelope must increment the fingerprint format version.

Do not silently change normalization rules while retaining the same version. Persisted fingerprints are only comparable when their complete identities match.
