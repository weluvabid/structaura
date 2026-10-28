use std::error::Error;
use std::fmt;
use std::ops::Range;

use structaura_core::{
    CanonicalNode, Fingerprint, FingerprintIdentity, NodeKind, Property, fingerprint,
};
use tree_sitter::{Node, Parser, Tree};

const SYNTAX_NODE_KIND: u32 = 1;
const NORMALIZATION_VERSION: u32 = 1;

/// Fingerprints TypeScript source using Structaura's versioned canonical representation.
pub fn fingerprint_typescript(source: &str) -> Result<Fingerprint, TypeScriptError> {
    let root = parse(source)?;
    let canonical = canonicalize(root.root_node(), source)?;
    let identity = FingerprintIdentity::new("typescript", NORMALIZATION_VERSION)
        .expect("static fingerprint identity is valid");

    Ok(fingerprint(&canonical, identity))
}

/// Parses TypeScript and returns syntax diagnostics when the source is invalid.
pub fn parse(source: &str) -> Result<Tree, TypeScriptError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .map_err(|error| TypeScriptError::ParserSetup(error.to_string()))?;

    let tree = parser.parse(source, None).ok_or(TypeScriptError::Cancelled)?;

    if tree.root_node().has_error() {
        let mut diagnostics = Vec::new();
        collect_errors(tree.root_node(), &mut diagnostics);
        return Err(TypeScriptError::Syntax(diagnostics));
    }

    Ok(tree)
}

fn canonicalize(node: Node<'_>, source: &str) -> Result<CanonicalNode, TypeScriptError> {
    if node.kind() == "parenthesized_expression" {
        let mut expressions = (0..node.named_child_count())
            .filter_map(|index| node.named_child(index))
            .filter(|child| !is_comment(*child));
        let first = expressions.next();
        let second = expressions.next();
        if let (Some(expression), None) = (first, second) {
            return canonicalize(expression, source);
        }
    }

    let kind = NodeKind::new(SYNTAX_NODE_KIND).expect("static node kind is valid");
    let mut properties = vec![Property::new("syntax_kind", node.kind())];
    let children = if node.kind() == "string" {
        properties.push(Property::new("literal", string_contents(node, source)));
        Vec::new()
    } else {
        if node.child_count() == 0 {
            properties.push(Property::new("token", node_text(node, source)));
        }

        let mut children = Vec::with_capacity(node.child_count());
        for index in 0..node.child_count() {
            let child = node.child(index).expect("child index is in range");
            if is_comment(child) || is_formatting_punctuation(child, source) {
                continue;
            }
            children.push(canonicalize(child, source)?);
        }
        children
    };

    CanonicalNode::new(kind, properties, children)
        .map_err(|error| TypeScriptError::Canonicalization(error.to_string()))
}

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub fn canonicalize_for_benchmark(
    node: Node<'_>,
    source: &str,
) -> Result<CanonicalNode, TypeScriptError> {
    canonicalize(node, source)
}

fn string_contents(node: Node<'_>, source: &str) -> String {
    let raw = node_text(node, source);
    if raw.len() >= 2 { raw[1..raw.len() - 1].to_owned() } else { raw.to_owned() }
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default()
}

fn is_comment(node: Node<'_>) -> bool {
    node.kind() == "comment"
}

fn is_formatting_punctuation(node: Node<'_>, source: &str) -> bool {
    if node.is_named() || node.child_count() != 0 {
        return false;
    }

    matches!(node_text(node, source), ";" | "," | "(" | ")" | "{" | "}" | "[" | "]" | ":")
}

fn collect_errors(node: Node<'_>, diagnostics: &mut Vec<SyntaxDiagnostic>) {
    if node.is_error() || node.is_missing() {
        diagnostics.push(SyntaxDiagnostic {
            message: if node.is_missing() {
                format!("expected {}", node.kind())
            } else {
                "unexpected syntax".to_owned()
            },
            byte_range: node.byte_range(),
        });
        return;
    }

    for index in 0..node.child_count() {
        if let Some(child) = node.child(index) {
            if child.has_error() || child.is_error() || child.is_missing() {
                collect_errors(child, diagnostics);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxDiagnostic {
    message: String,
    byte_range: Range<usize>,
}

impl SyntaxDiagnostic {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.byte_range.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeScriptError {
    ParserSetup(String),
    Cancelled,
    Syntax(Vec<SyntaxDiagnostic>),
    Canonicalization(String),
}

impl fmt::Display for TypeScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParserSetup(message) => {
                write!(f, "could not initialize TypeScript parser: {message}")
            }
            Self::Cancelled => f.write_str("TypeScript parsing was cancelled"),
            Self::Syntax(diagnostics) => {
                write!(f, "TypeScript syntax error ({} diagnostic(s))", diagnostics.len())?;
                if let Some(first) = diagnostics.first() {
                    write!(
                        f,
                        ": {} at bytes {}..{}",
                        first.message, first.byte_range.start, first.byte_range.end
                    )?;
                }
                Ok(())
            }
            Self::Canonicalization(message) => {
                write!(f, "could not canonicalize TypeScript syntax tree: {message}")
            }
        }
    }
}

impl Error for TypeScriptError {}
