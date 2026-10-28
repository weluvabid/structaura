use std::error::Error;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::{CanonicalNode, hash_node};

pub const FINGERPRINT_FORMAT_VERSION: u16 = 1;
const ROOT_DOMAIN: &[u8] = b"structaura:root:v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FingerprintIdentity {
    language: String,
    normalization_version: u32,
}

impl FingerprintIdentity {
    pub fn new(
        language: impl Into<String>,
        normalization_version: u32,
    ) -> Result<Self, FingerprintIdentityError> {
        let language = language.into();
        if !valid_language_id(&language) {
            return Err(FingerprintIdentityError::InvalidLanguageId);
        }
        if normalization_version == 0 {
            return Err(FingerprintIdentityError::InvalidNormalizationVersion);
        }

        Ok(Self { language, normalization_version })
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub const fn normalization_version(&self) -> u32 {
        self.normalization_version
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fingerprint {
    format_version: u16,
    identity: FingerprintIdentity,
    digest: [u8; 32],
}

impl Fingerprint {
    pub const fn format_version(&self) -> u16 {
        self.format_version
    }

    pub fn identity(&self) -> &FingerprintIdentity {
        &self.identity
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "structaura:v{}:{}:n{}:",
            self.format_version, self.identity.language, self.identity.normalization_version
        )?;

        for byte in self.digest {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

pub fn fingerprint(root: &CanonicalNode, identity: FingerprintIdentity) -> Fingerprint {
    let root_hash = hash_node(root);
    let mut hasher = Sha256::new();
    hasher.update(ROOT_DOMAIN);
    hasher.update(FINGERPRINT_FORMAT_VERSION.to_be_bytes());
    write_bytes(&mut hasher, identity.language.as_bytes());
    hasher.update(identity.normalization_version.to_be_bytes());
    hasher.update(root_hash);

    Fingerprint {
        format_version: FINGERPRINT_FORMAT_VERSION,
        identity,
        digest: hasher.finalize().into(),
    }
}

pub fn compare_fingerprints(
    left: &Fingerprint,
    right: &Fingerprint,
) -> Result<bool, FingerprintComparisonError> {
    if left.format_version != right.format_version {
        return Err(FingerprintComparisonError::FormatVersionMismatch);
    }
    if left.identity.language != right.identity.language {
        return Err(FingerprintComparisonError::LanguageMismatch);
    }
    if left.identity.normalization_version != right.identity.normalization_version {
        return Err(FingerprintComparisonError::NormalizationVersionMismatch);
    }

    Ok(left.digest == right.digest)
}

fn valid_language_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value.as_bytes()[0] != b'-'
        && value.as_bytes()[value.len() - 1] != b'-'
}

fn write_bytes(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FingerprintIdentityError {
    InvalidLanguageId,
    InvalidNormalizationVersion,
}

impl fmt::Display for FingerprintIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLanguageId => f.write_str(
                "language id must contain lowercase ASCII letters, digits, or internal hyphens",
            ),
            Self::InvalidNormalizationVersion => {
                f.write_str("normalization version must be greater than zero")
            }
        }
    }
}

impl Error for FingerprintIdentityError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FingerprintComparisonError {
    FormatVersionMismatch,
    LanguageMismatch,
    NormalizationVersionMismatch,
}

impl fmt::Display for FingerprintComparisonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormatVersionMismatch => f.write_str("fingerprint format versions differ"),
            Self::LanguageMismatch => f.write_str("fingerprints use different languages"),
            Self::NormalizationVersionMismatch => {
                f.write_str("fingerprints use different normalization versions")
            }
        }
    }
}

impl Error for FingerprintComparisonError {}

#[cfg(test)]
mod tests {
    use super::{
        FingerprintComparisonError, FingerprintIdentity, compare_fingerprints, fingerprint,
    };
    use crate::{CanonicalNode, NodeKind, Property};

    fn node(identifier: &str) -> CanonicalNode {
        CanonicalNode::new(
            NodeKind::new(1).unwrap(),
            vec![Property::new("name", identifier)],
            Vec::new(),
        )
        .unwrap()
    }

    fn identity(language: &str, version: u32) -> FingerprintIdentity {
        FingerprintIdentity::new(language, version).unwrap()
    }

    #[test]
    fn invalid_identity_values_are_rejected() {
        assert!(FingerprintIdentity::new("", 1).is_err());
        assert!(FingerprintIdentity::new("JavaScript", 1).is_err());
        assert!(FingerprintIdentity::new("javascript", 0).is_err());
    }

    #[test]
    fn identical_structures_have_identical_fingerprints() {
        let first = fingerprint(&node("result"), identity("javascript", 1));
        let second = fingerprint(&node("result"), identity("javascript", 1));

        assert_eq!(first, second);
        assert_eq!(compare_fingerprints(&first, &second), Ok(true));
        assert!(first.to_string().starts_with("structaura:v1:javascript:n1:"));
    }

    #[test]
    fn structural_changes_change_the_fingerprint() {
        let first = fingerprint(&node("result"), identity("javascript", 1));
        let second = fingerprint(&node("value"), identity("javascript", 1));

        assert_ne!(first.digest(), second.digest());
        assert_eq!(compare_fingerprints(&first, &second), Ok(false));
    }

    #[test]
    fn incompatible_languages_cannot_be_compared() {
        let javascript = fingerprint(&node("x"), identity("javascript", 1));
        let typescript = fingerprint(&node("x"), identity("typescript", 1));

        assert_eq!(
            compare_fingerprints(&javascript, &typescript),
            Err(FingerprintComparisonError::LanguageMismatch)
        );
        assert_ne!(javascript.digest(), typescript.digest());
    }

    #[test]
    fn incompatible_normalization_versions_cannot_be_compared() {
        let first = fingerprint(&node("x"), identity("javascript", 1));
        let second = fingerprint(&node("x"), identity("javascript", 2));

        assert_eq!(
            compare_fingerprints(&first, &second),
            Err(FingerprintComparisonError::NormalizationVersionMismatch)
        );
    }
}
