use sha2::{Digest, Sha256};

use crate::{CanonicalNode, encoding::encode_node};

pub fn hash_node(node: &CanonicalNode) -> [u8; 32] {
    let child_hashes: Vec<[u8; 32]> = node.children().iter().map(hash_node).collect();
    let encoded = encode_node(node, &child_hashes);
    Sha256::digest(encoded).into()
}
