use std::error::Error;
use std::fmt;
use std::ops::Range;

use structaura_core::{
    CanonicalNode, Fingerprint, FingerprintIdentity, SyntaxNodeRecord, canonicalize_syntax_nodes,
    fingerprint,
};
use tree_sitter::{Node, Parser, Tree};

const NORMALIZATION_VERSION: u32 = 1;

/// Fingerprints JavaScript source using Structaura's versioned canonical representation.
pub fn fingerprint_javascript(source: &str) -> Result<Fingerprint, JavaScriptError> {
    let root = parse(source)?;
    let canonical = canonicalize(root.root_node(), source)?;
    let identity = FingerprintIdentity::new("javascript", NORMALIZATION_VERSION)
        .expect("static fingerprint identity is valid");

    Ok(fingerprint(&canonical, identity))
}

/// Parses JavaScript and returns syntax diagnostics when the source is invalid.
pub fn parse(source: &str) -> Result<Tree, JavaScriptError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .map_err(|error| JavaScriptError::ParserSetup(error.to_string()))?;

    let tree = parser.parse(source, None).ok_or(JavaScriptError::Cancelled)?;

    if tree.root_node().has_error() {
        let mut diagnostics = Vec::new();
        collect_errors(tree.root_node(), &mut diagnostics);
        return Err(JavaScriptError::Syntax(diagnostics));
    }

    Ok(tree)
}

fn canonicalize(node: Node<'_>, source: &str) -> Result<CanonicalNode, JavaScriptError> {
    let mut pending = vec![node];
    let mut records = Vec::new();

    while let Some(node) = pending.pop() {
        let children =
            (0..node.child_count()).filter_map(|index| node.child(index)).collect::<Vec<_>>();
        records.push(SyntaxNodeRecord {
            syntax_kind: node.kind().to_owned(),
            text: if node.kind() == "string" || node.child_count() == 0 {
                node_text(node, source).to_owned()
            } else {
                String::new()
            },
            is_named: node.is_named(),
            child_count: u32::try_from(children.len()).map_err(|_| {
                JavaScriptError::Canonicalization("syntax node exceeds u32 child limit".to_owned())
            })?,
        });
        pending.extend(children.into_iter().rev());
    }

    canonicalize_syntax_nodes(&records)
        .map_err(|error| JavaScriptError::Canonicalization(error.to_string()))
}

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub fn canonicalize_for_benchmark(
    node: Node<'_>,
    source: &str,
) -> Result<CanonicalNode, JavaScriptError> {
    canonicalize(node, source)
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default()
}

fn collect_errors(node: Node<'_>, diagnostics: &mut Vec<SyntaxDiagnostic>) {
    let mut pending = vec![node];

    while let Some(node) = pending.pop() {
        if node.is_error() || node.is_missing() {
            diagnostics.push(SyntaxDiagnostic {
                message: if node.is_missing() {
                    format!("expected {}", node.kind())
                } else {
                    "unexpected syntax".to_owned()
                },
                byte_range: node.byte_range(),
            });
            continue;
        }

        for index in (0..node.child_count()).rev() {
            if let Some(child) = node.child(index) {
                if child.has_error() || child.is_error() || child.is_missing() {
                    pending.push(child);
                }
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
pub enum JavaScriptError {
    ParserSetup(String),
    Cancelled,
    Syntax(Vec<SyntaxDiagnostic>),
    Canonicalization(String),
}

impl fmt::Display for JavaScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParserSetup(message) => {
                write!(f, "could not initialize JavaScript parser: {message}")
            }
            Self::Cancelled => f.write_str("JavaScript parsing was cancelled"),
            Self::Syntax(diagnostics) => {
                write!(f, "JavaScript syntax error ({} diagnostic(s))", diagnostics.len())?;
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
                write!(f, "could not canonicalize JavaScript syntax tree: {message}")
            }
        }
    }
}

impl Error for JavaScriptError {}
