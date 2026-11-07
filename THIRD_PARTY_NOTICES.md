# Third-party notices

StructAura includes or distributes the following third-party components. Their license texts are included in `pkg/grammars/licenses/` in the npm distribution.

| Component | Purpose | License |
| --- | --- | --- |
| [web-tree-sitter](https://github.com/tree-sitter/tree-sitter/tree/master/lib/binding_web) | Browser-side Tree-sitter runtime | MIT |
| [tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript) | JavaScript grammar compiled to WebAssembly | MIT |
| [tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript) | TypeScript and TSX grammars compiled to WebAssembly | MIT |

The Tree-sitter CLI is a development/build dependency and is not bundled into the published runtime package. Its package metadata and license are available from the npm registry.

The Rust dependency licenses are available from their respective package manifests and the Cargo registry. StructAura does not relicense third-party components; each remains subject to its own license.
