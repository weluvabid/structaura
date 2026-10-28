use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeKind(u32);

impl NodeKind {
    pub fn new(value: u32) -> Result<Self, StructuralNodeError> {
        if value == 0 {
            return Err(StructuralNodeError::InvalidNodeKind);
        }

        Ok(Self(value))
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Bytes(Vec<u8>),
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for PropertyValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for PropertyValue {
    fn from(value: i64) -> Self {
        Self::Signed(value)
    }
}

impl From<u64> for PropertyValue {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}

impl From<Vec<u8>> for PropertyValue {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    key: String,
    value: PropertyValue,
}

impl Property {
    pub fn new(key: impl Into<String>, value: impl Into<PropertyValue>) -> Self {
        Self { key: key.into(), value: value.into() }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalNode {
    kind: NodeKind,
    properties: Vec<Property>,
    children: Vec<Self>,
}

impl CanonicalNode {
    pub fn new(
        kind: NodeKind,
        mut properties: Vec<Property>,
        children: Vec<Self>,
    ) -> Result<Self, StructuralNodeError> {
        if properties.iter().any(|property| property.key.is_empty()) {
            return Err(StructuralNodeError::EmptyPropertyKey);
        }

        properties.sort_unstable_by(|left, right| left.key.cmp(&right.key));

        if let Some(pair) = properties.windows(2).find(|pair| pair[0].key == pair[1].key) {
            return Err(StructuralNodeError::DuplicatePropertyKey(pair[0].key.clone()));
        }

        Ok(Self { kind, properties, children })
    }

    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuralNodeError {
    InvalidNodeKind,
    EmptyPropertyKey,
    DuplicatePropertyKey(String),
}

impl fmt::Display for StructuralNodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNodeKind => f.write_str("node kind 0 is reserved"),
            Self::EmptyPropertyKey => f.write_str("property keys must not be empty"),
            Self::DuplicatePropertyKey(key) => write!(f, "duplicate property key: {key}"),
        }
    }
}

impl Error for StructuralNodeError {}

#[cfg(test)]
mod tests {
    use super::{CanonicalNode, NodeKind, Property, StructuralNodeError};

    #[test]
    fn property_order_is_canonicalized() {
        let first = CanonicalNode::new(
            NodeKind::new(1).unwrap(),
            vec![Property::new("b", "2"), Property::new("a", "1")],
            Vec::new(),
        )
        .unwrap();
        let second = CanonicalNode::new(
            NodeKind::new(1).unwrap(),
            vec![Property::new("a", "1"), Property::new("b", "2")],
            Vec::new(),
        )
        .unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn duplicate_property_keys_are_rejected() {
        let result = CanonicalNode::new(
            NodeKind::new(1).unwrap(),
            vec![Property::new("name", "a"), Property::new("name", "b")],
            Vec::new(),
        );

        assert_eq!(result, Err(StructuralNodeError::DuplicatePropertyKey("name".to_owned())));
    }

    #[test]
    fn empty_property_keys_are_rejected() {
        let result = CanonicalNode::new(
            NodeKind::new(1).unwrap(),
            vec![Property::new("", "value")],
            Vec::new(),
        );

        assert_eq!(result, Err(StructuralNodeError::EmptyPropertyKey));
    }

    #[test]
    fn zero_is_not_a_valid_node_kind() {
        assert_eq!(NodeKind::new(0), Err(StructuralNodeError::InvalidNodeKind));
    }
}
