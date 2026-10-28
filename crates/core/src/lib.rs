mod encoding;
mod fingerprint;
mod ir;
mod merkle;

pub use fingerprint::{
    FINGERPRINT_FORMAT_VERSION, Fingerprint, FingerprintComparisonError, FingerprintIdentity,
    FingerprintIdentityError, compare_fingerprints, fingerprint,
};
pub use ir::{CanonicalNode, NodeKind, Property, PropertyValue, StructuralNodeError};
pub use merkle::hash_node;

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub use encoding::encode_node_for_benchmark;
