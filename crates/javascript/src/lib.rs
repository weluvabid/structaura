use std::error::Error;
use std::fmt;
use std::ops::Range;

use structaura_core::{
    CanonicalNode, Fingerprint, FingerprintIdentity, NodeKind, Property, fingerprint,
};
use tree_sitter::{Node, Parser, Tree};

const SYNTAX_NODE_KIND: u32 = 1;
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
    let mut pending = vec![CanonicalizeTask::Visit(node)];
    let mut completed = Vec::new();

    while let Some(task) = pending.pop() {
        match task {
            CanonicalizeTask::Visit(node) => {
                if let Some(expression) = transparent_parenthesized_expression(node) {
                    pending.push(CanonicalizeTask::Visit(expression));
                    continue;
                }

                let children = canonical_children(node, source);
                pending.push(CanonicalizeTask::Build { node, child_count: children.len() });
                pending.extend(children.into_iter().rev().map(CanonicalizeTask::Visit));
            }
            CanonicalizeTask::Build { node, child_count } => {
                let mut children = Vec::with_capacity(child_count);
                for _ in 0..child_count {
                    children.push(completed.pop().expect("child canonicalization completed"));
                }
                children.reverse();

                let kind = NodeKind::new(SYNTAX_NODE_KIND).expect("static node kind is valid");
                let mut properties = vec![Property::new("syntax_kind", node.kind())];

                if node.kind() == "string" {
                    properties.push(Property::new("literal", string_contents(node, source)));
                } else if node.child_count() == 0 {
                    properties.push(Property::new("token", node_text(node, source)));
                }

                let canonical = CanonicalNode::new(kind, properties, children)
                    .map_err(|error| JavaScriptError::Canonicalization(error.to_string()))?;
                completed.push(canonical);
            }
        }
    }

    Ok(completed.pop().expect("root canonicalization completed"))
}

enum CanonicalizeTask<'tree> {
    Visit(Node<'tree>),
    Build { node: Node<'tree>, child_count: usize },
}

fn transparent_parenthesized_expression<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    if node.kind() != "parenthesized_expression" {
        return None;
    }

    let mut expressions = (0..node.named_child_count())
        .filter_map(|index| node.named_child(index))
        .filter(|child| !is_comment(*child));
    let expression = expressions.next()?;
    expressions.next().is_none().then_some(expression)
}

fn canonical_children<'tree>(node: Node<'tree>, source: &str) -> Vec<Node<'tree>> {
    if node.kind() == "string" {
        return Vec::new();
    }

    (0..node.child_count())
        .filter_map(|index| node.child(index))
        .filter(|child| !is_comment(*child) && !is_formatting_punctuation(*child, source))
        .collect()
}

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub fn canonicalize_for_benchmark(
    node: Node<'_>,
    source: &str,
) -> Result<CanonicalNode, JavaScriptError> {
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
