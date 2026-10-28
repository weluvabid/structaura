use sha2::{Digest, Sha256};

use crate::{CanonicalNode, PropertyValue};

const NODE_DOMAIN: &[u8] = b"structaura:node:v1\0";

pub(crate) trait ByteSink {
    fn write(&mut self, bytes: &[u8]);
}

impl ByteSink for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

impl ByteSink for Sha256 {
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

#[cfg(test)]
pub(crate) fn encode_node(node: &CanonicalNode, child_hashes: &[[u8; 32]]) -> Vec<u8> {
    let mut output = Vec::new();
    write_node_prefix(&mut output, node, child_hashes.len());
    for hash in child_hashes {
        output.write(hash);
    }
    output
}

pub(crate) fn write_node_prefix(
    sink: &mut impl ByteSink,
    node: &CanonicalNode,
    child_count: usize,
) {
    sink.write(NODE_DOMAIN);
    sink.write(&node.kind().value().to_be_bytes());
    sink.write(&(node.properties().len() as u64).to_be_bytes());

    for property in node.properties() {
        write_bytes(sink, property.key().as_bytes());
        match property.value() {
            PropertyValue::Text(value) => {
                sink.write(&[1]);
                write_bytes(sink, value.as_bytes());
            }
            PropertyValue::Bool(value) => {
                sink.write(&[2]);
                sink.write(&[u8::from(*value)]);
            }
            PropertyValue::Signed(value) => {
                sink.write(&[3]);
                sink.write(&value.to_be_bytes());
            }
            PropertyValue::Unsigned(value) => {
                sink.write(&[4]);
                sink.write(&value.to_be_bytes());
            }
            PropertyValue::Bytes(value) => {
                sink.write(&[5]);
                write_bytes(sink, value);
            }
        }
    }

    sink.write(&(child_count as u64).to_be_bytes());
}

fn write_bytes(sink: &mut impl ByteSink, value: &[u8]) {
    sink.write(&(value.len() as u64).to_be_bytes());
    sink.write(value);
}

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub fn encode_node_for_benchmark(node: &CanonicalNode, child_hashes: &[[u8; 32]]) -> Vec<u8> {
    encode_node(node, child_hashes)
}

#[cfg(test)]
mod tests {
    use super::encode_node;
    use crate::{CanonicalNode, NodeKind, Property};

    fn node(properties: Vec<Property>) -> CanonicalNode {
        CanonicalNode::new(NodeKind::new(1).unwrap(), properties, Vec::new()).unwrap()
    }

    #[test]
    fn property_order_does_not_change_encoding() {
        let left = node(vec![Property::new("a", "1"), Property::new("b", "2")]);
        let right = node(vec![Property::new("b", "2"), Property::new("a", "1")]);

        assert_eq!(encode_node(&left, &[]), encode_node(&right, &[]));
    }

    #[test]
    fn field_boundaries_are_unambiguous() {
        let left = node(vec![Property::new("ab", "c")]);
        let right = node(vec![Property::new("a", "bc")]);

        assert_ne!(encode_node(&left, &[]), encode_node(&right, &[]));
    }

    #[test]
    fn streamed_encoding_matches_buffered_encoding() {
        use sha2::{Digest, Sha256};

        let value = node(vec![Property::new("name", "value")]);
        let encoded = encode_node(&value, &[]);
        let mut streamed = Sha256::new();
        super::write_node_prefix(&mut streamed, &value, 0);
        let streamed_digest: [u8; 32] = streamed.finalize().into();
        let encoded_digest: [u8; 32] = Sha256::digest(&encoded).into();

        assert_eq!(streamed_digest, encoded_digest);
    }
}
