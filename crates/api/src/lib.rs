use std::error::Error;
use std::fmt;

pub use structaura_core::{
    FINGERPRINT_FORMAT_VERSION, Fingerprint, FingerprintComparisonError, FingerprintIdentity,
    FingerprintIdentityError, compare_fingerprints,
};

/// The source language used to construct a fingerprint.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Language {
    JavaScript,
    TypeScript,
}

impl Language {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
        }
    }
}

/// Fingerprints source using the adapter associated with `language`.
pub fn fingerprint_source(
    source: &str,
    language: Language,
) -> Result<Fingerprint, FingerprintError> {
    match language {
        Language::JavaScript => structaura_javascript::fingerprint_javascript(source)
            .map_err(FingerprintError::JavaScript),
        Language::TypeScript => structaura_typescript::fingerprint_typescript(source)
            .map_err(FingerprintError::TypeScript),
    }
}

/// Fingerprints JavaScript source.
pub fn fingerprint_javascript(source: &str) -> Result<Fingerprint, FingerprintError> {
    fingerprint_source(source, Language::JavaScript)
}

/// Fingerprints TypeScript source.
pub fn fingerprint_typescript(source: &str) -> Result<Fingerprint, FingerprintError> {
    fingerprint_source(source, Language::TypeScript)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FingerprintError {
    JavaScript(structaura_javascript::JavaScriptError),
    TypeScript(structaura_typescript::TypeScriptError),
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JavaScript(error) => fmt::Display::fmt(error, f),
            Self::TypeScript(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl Error for FingerprintError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::JavaScript(error) => Some(error),
            Self::TypeScript(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FingerprintError, Language, fingerprint_javascript, fingerprint_source,
        fingerprint_typescript,
    };

    #[test]
    fn dispatches_to_the_selected_language_adapter() {
        let javascript = fingerprint_source("const value = 1;", Language::JavaScript).unwrap();
        let typescript =
            fingerprint_source("const value: number = 1;", Language::TypeScript).unwrap();

        assert_eq!(javascript.identity().language(), "javascript");
        assert_eq!(typescript.identity().language(), "typescript");
    }

    #[test]
    fn convenience_functions_match_dispatch() {
        assert_eq!(
            fingerprint_javascript("const value = 1;").unwrap(),
            fingerprint_source("const value = 1;", Language::JavaScript).unwrap()
        );
        assert_eq!(
            fingerprint_typescript("const value: number = 1;").unwrap(),
            fingerprint_source("const value: number = 1;", Language::TypeScript).unwrap()
        );
    }

    #[test]
    fn errors_preserve_the_adapter_and_diagnostics() {
        assert!(matches!(
            fingerprint_source("const = ;", Language::JavaScript),
            Err(FingerprintError::JavaScript(_))
        ));
        assert!(matches!(
            fingerprint_source("const value: = 1;", Language::TypeScript),
            Err(FingerprintError::TypeScript(_))
        ));
    }

    #[test]
    fn fingerprints_remain_language_specific() {
        let javascript = fingerprint_javascript("const value = 1;").unwrap();
        let typescript = fingerprint_typescript("const value: number = 1;").unwrap();

        assert_ne!(javascript.identity().language(), typescript.identity().language());
    }
}
