use sha2::{Digest, Sha256};

use crate::{CanonicalNode, encoding::write_node_prefix};

pub fn hash_node(node: &CanonicalNode) -> [u8; 32] {
    let mut hasher = Sha256::new();
    write_node_prefix(&mut hasher, node, node.children().len());

    for child in node.children() {
        hasher.update(hash_node(child));
    }

    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::hash_node;
    use crate::{CanonicalNode, NodeKind, Property};

    fn node(properties: Vec<Property>, children: Vec<CanonicalNode>) -> CanonicalNode {
        CanonicalNode::new(NodeKind::new(1).unwrap(), properties, children).unwrap()
    }

    #[test]
    fn streaming_hash_preserves_the_existing_node_digest() {
        let child = node(
            vec![Property::new("syntax_kind", "identifier"), Property::new("token", "x")],
            Vec::new(),
        );
        let root = node(vec![Property::new("syntax_kind", "program")], vec![child]);

        let digest = hash_node(&root);
        let actual = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();

        assert_eq!(actual, "84d695873ece9bc5acf742708522a12f3cc0ba12c671e5dd319699730f0b2d52");
    }
}
