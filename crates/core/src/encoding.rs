use crate::{CanonicalNode, PropertyValue};

const NODE_DOMAIN: &[u8] = b"structaura:node:v1\0";

pub(crate) fn encode_node(node: &CanonicalNode, child_hashes: &[[u8; 32]]) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(NODE_DOMAIN);
    output.extend_from_slice(&node.kind().value().to_be_bytes());
    output.extend_from_slice(&(node.properties().len() as u64).to_be_bytes());

    for property in node.properties() {
        write_bytes(&mut output, property.key().as_bytes());
        match property.value() {
            PropertyValue::Text(value) => {
                output.push(1);
                write_bytes(&mut output, value.as_bytes());
            }
            PropertyValue::Bool(value) => {
                output.push(2);
                output.push(u8::from(*value));
            }
            PropertyValue::Signed(value) => {
                output.push(3);
                output.extend_from_slice(&value.to_be_bytes());
            }
            PropertyValue::Unsigned(value) => {
                output.push(4);
                output.extend_from_slice(&value.to_be_bytes());
            }
            PropertyValue::Bytes(value) => {
                output.push(5);
                write_bytes(&mut output, value);
            }
        }
    }

    output.extend_from_slice(&(child_hashes.len() as u64).to_be_bytes());
    for hash in child_hashes {
        output.extend_from_slice(hash);
    }

    output
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
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
}
