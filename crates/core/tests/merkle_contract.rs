use structaura_core::{
    CanonicalNode, FingerprintIdentity, NodeKind, Property, compare_fingerprints, fingerprint,
};

fn node(kind: u32, value: &str, children: Vec<CanonicalNode>) -> CanonicalNode {
    CanonicalNode::new(NodeKind::new(kind).unwrap(), vec![Property::new("value", value)], children)
        .unwrap()
}

fn fingerprint_for(root: &CanonicalNode) -> structaura_core::Fingerprint {
    fingerprint(root, FingerprintIdentity::new("javascript", 1).unwrap())
}

#[test]
fn child_order_changes_the_root_fingerprint() {
    let left = node(1, "binary", vec![node(2, "a", vec![]), node(2, "b", vec![])]);
    let right = node(1, "binary", vec![node(2, "b", vec![]), node(2, "a", vec![])]);

    assert_ne!(fingerprint_for(&left), fingerprint_for(&right));
}

#[test]
fn node_kind_and_child_count_are_structural() {
    let leaf = node(1, "x", vec![]);
    let different_kind = node(2, "x", vec![]);
    let parent = node(1, "x", vec![node(2, "child", vec![])]);

    assert_ne!(fingerprint_for(&leaf), fingerprint_for(&different_kind));
    assert_ne!(fingerprint_for(&leaf), fingerprint_for(&parent));
}

#[test]
fn comparison_is_false_for_different_structures() {
    let left = fingerprint_for(&node(1, "a", vec![]));
    let right = fingerprint_for(&node(1, "b", vec![]));

    assert_eq!(compare_fingerprints(&left, &right), Ok(false));
}
