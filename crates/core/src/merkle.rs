use sha2::{Digest, Sha256};

use crate::{CanonicalNode, encoding::write_node_prefix};

pub fn hash_node(node: &CanonicalNode) -> [u8; 32] {
    let mut pending = vec![(node, false)];
    let mut completed = Vec::new();

    while let Some((node, visited)) = pending.pop() {
        if !visited {
            pending.push((node, true));
            pending.extend(node.children().iter().rev().map(|child| (child, false)));
            continue;
        }

        let child_count = node.children().len();
        let split = completed.len() - child_count;
        let mut hasher = Sha256::new();
        write_node_prefix(&mut hasher, node, child_count);
        for hash in &completed[split..] {
            hasher.update(hash);
        }
        let digest = hasher.finalize().into();
        completed.truncate(split);
        completed.push(digest);
    }

    completed.pop().expect("root hash completed")
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
