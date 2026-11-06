use serde::Deserialize;
use std::{error::Error, fmt};

use crate::{CanonicalNode, NodeKind, Property, StructuralNodeError};

/// Flat preorder syntax record shared by native and browser parser adapters.
/// `text` is populated only for leaves and string literals to avoid copying overlapping
/// source spans for every nested syntax node.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SyntaxNodeRecord {
    pub syntax_kind: String,
    pub text: String,
    pub is_named: bool,
    pub child_count: u32,
}

/// Applies StructAura's normalization policy and reconstructs the canonical tree
/// without recursion. The input contains the complete raw parser tree in preorder.
pub fn canonicalize_syntax_nodes(
    records: &[SyntaxNodeRecord],
) -> Result<CanonicalNode, SyntaxCanonicalizationError> {
    if records.is_empty() {
        return Err(SyntaxCanonicalizationError::EmptyInput);
    }

    let mut completed: Vec<CompletedRecord> = Vec::with_capacity(records.len());
    for record in records.iter().rev() {
        let child_count = record.child_count as usize;
        if child_count > completed.len() {
            return Err(SyntaxCanonicalizationError::InvalidChildCount {
                syntax_kind: record.syntax_kind.clone(),
                declared: child_count,
                available: completed.len(),
            });
        }

        let mut children = completed.split_off(completed.len() - child_count);
        children.reverse();
        let is_comment = record.syntax_kind == "comment";
        let is_formatting_punctuation = !record.is_named
            && child_count == 0
            && matches!(record.text.as_str(), ";" | "," | "(" | ")" | "{" | "}" | "[" | "]" | ":");

        let transparent_child = if record.syntax_kind == "parenthesized_expression" {
            let mut expressions =
                children.iter().filter(|child| child.is_named && !child.is_comment);
            match (expressions.next(), expressions.next()) {
                (Some(expression), None) => Some(expression.canonical.clone()),
                _ => None,
            }
        } else {
            None
        };

        let canonical = if is_comment || is_formatting_punctuation {
            None
        } else if let Some(canonical) = transparent_child {
            canonical
        } else {
            Some(build_node(record, children)?)
        };

        completed.push(CompletedRecord { canonical, is_named: record.is_named, is_comment });
    }

    if completed.len() != 1 {
        return Err(SyntaxCanonicalizationError::InvalidRootCount(completed.len()));
    }

    completed.pop().and_then(|root| root.canonical).ok_or(SyntaxCanonicalizationError::RootOmitted)
}

fn build_node(
    record: &SyntaxNodeRecord,
    children: Vec<CompletedRecord>,
) -> Result<CanonicalNode, SyntaxCanonicalizationError> {
    let mut properties = vec![Property::new("syntax_kind", record.syntax_kind.clone())];
    let retained_children = if record.syntax_kind == "string" {
        properties.push(Property::new("literal", string_contents(&record.text)));
        Vec::new()
    } else {
        if record.child_count == 0 {
            properties.push(Property::new("token", record.text.clone()));
        }
        children.into_iter().filter_map(|child| child.canonical).collect()
    };

    Ok(CanonicalNode::new(
        NodeKind::new(1).expect("static canonical node kind is valid"),
        properties,
        retained_children,
    )?)
}

fn string_contents(raw: &str) -> String {
    if raw.len() >= 2 { raw[1..raw.len() - 1].to_owned() } else { raw.to_owned() }
}

struct CompletedRecord {
    canonical: Option<CanonicalNode>,
    is_named: bool,
    is_comment: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyntaxCanonicalizationError {
    EmptyInput,
    InvalidChildCount { syntax_kind: String, declared: usize, available: usize },
    InvalidRootCount(usize),
    RootOmitted,
    Structural(StructuralNodeError),
}

impl From<StructuralNodeError> for SyntaxCanonicalizationError {
    fn from(value: StructuralNodeError) -> Self {
        Self::Structural(value)
    }
}

impl fmt::Display for SyntaxCanonicalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("syntax record input is empty"),
            Self::InvalidChildCount { syntax_kind, declared, available } => write!(
                f,
                "node '{}' declares {} children but only {} are available",
                syntax_kind, declared, available,
            ),
            Self::InvalidRootCount(count) => {
                write!(f, "syntax records describe {count} roots; exactly one is required")
            }
            Self::RootOmitted => f.write_str("the syntax root was removed by normalization"),
            Self::Structural(error) => write!(f, "invalid canonical structure: {error}"),
        }
    }
}

impl Error for SyntaxCanonicalizationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(kind: &str, text: &str, is_named: bool, child_count: u32) -> SyntaxNodeRecord {
        SyntaxNodeRecord { syntax_kind: kind.into(), text: text.into(), is_named, child_count }
    }

    #[test]
    fn reconstructs_preorder_records_and_preserves_child_order() {
        let records = vec![
            record("program", "", true, 2),
            record("identifier", "left", true, 0),
            record("identifier", "right", true, 0),
        ];
        let tree = canonicalize_syntax_nodes(&records).unwrap();
        assert_eq!(tree.children().len(), 2);
        assert_eq!(
            tree.children()[0].properties()[1].value(),
            &crate::PropertyValue::Text("left".into())
        );
        assert_eq!(
            tree.children()[1].properties()[1].value(),
            &crate::PropertyValue::Text("right".into())
        );
    }

    #[test]
    fn removes_comments_punctuation_and_parentheses() {
        let records = vec![
            record("program", "", true, 2),
            record("parenthesized_expression", "", true, 3),
            record("(", "(", false, 0),
            record("identifier", "value", true, 0),
            record(")", ")", false, 0),
            record("comment", "", true, 0),
        ];
        let tree = canonicalize_syntax_nodes(&records).unwrap();
        assert_eq!(tree.children().len(), 1);
        assert_eq!(
            tree.children()[0].properties()[1].value(),
            &crate::PropertyValue::Text("value".into())
        );
    }

    #[test]
    fn strips_string_delimiters_without_decoding_escapes() {
        let records = vec![record("string", "'a\\nb'", true, 0)];
        let tree = canonicalize_syntax_nodes(&records).unwrap();
        assert_eq!(
            tree.properties().iter().find(|property| property.key() == "literal").unwrap().value(),
            &crate::PropertyValue::Text("a\\nb".into())
        );
    }

    #[test]
    fn rejects_malformed_flat_tree_shapes() {
        let records = vec![record("program", "", true, 1)];
        assert!(matches!(
            canonicalize_syntax_nodes(&records),
            Err(SyntaxCanonicalizationError::InvalidChildCount { .. })
        ));
    }
}
