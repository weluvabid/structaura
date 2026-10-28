use structaura::{
    FingerprintComparisonError, FingerprintError, Language, compare_fingerprints,
    fingerprint_javascript, fingerprint_source, fingerprint_typescript,
};

#[test]
fn public_api_normalizes_formatting_and_comments_for_both_languages() {
    let javascript_compact = fingerprint_javascript("const answer = left + right;").unwrap();
    let javascript_formatted =
        fingerprint_source("// comment\nconst answer = ( left + right )\n", Language::JavaScript)
            .unwrap();
    assert_eq!(javascript_compact, javascript_formatted);

    let typescript_compact =
        fingerprint_typescript("const answer: number = left + right;").unwrap();
    let typescript_formatted = fingerprint_source(
        "/* comment */\nconst answer : number = ( left + right )\n",
        Language::TypeScript,
    )
    .unwrap();
    assert_eq!(typescript_compact, typescript_formatted);
}

#[test]
fn public_api_detects_structural_changes() {
    let original = fingerprint_javascript("const value = left + right;").unwrap();
    let changed_identifier = fingerprint_javascript("const result = left + right;").unwrap();
    let changed_operator = fingerprint_javascript("const value = left - right;").unwrap();

    assert_eq!(compare_fingerprints(&original, &original), Ok(true));
    assert_eq!(compare_fingerprints(&original, &changed_identifier), Ok(false));
    assert_eq!(compare_fingerprints(&original, &changed_operator), Ok(false));
}

#[test]
fn language_identity_is_part_of_the_public_fingerprint_contract() {
    let javascript = fingerprint_javascript("const value = 1;").unwrap();
    let typescript = fingerprint_typescript("const value: number = 1;").unwrap();

    assert_eq!(javascript.identity().language(), "javascript");
    assert_eq!(typescript.identity().language(), "typescript");
    assert_ne!(javascript.digest(), typescript.digest());
    assert_eq!(
        compare_fingerprints(&javascript, &typescript),
        Err(FingerprintComparisonError::LanguageMismatch)
    );
}

#[test]
fn dispatch_and_language_specific_helpers_return_the_same_fingerprint() {
    let javascript = fingerprint_javascript("const value = 1;").unwrap();
    assert_eq!(javascript, fingerprint_source("const value = 1;", Language::JavaScript).unwrap());

    let typescript = fingerprint_typescript("const value: number = 1;").unwrap();
    assert_eq!(
        typescript,
        fingerprint_source("const value: number = 1;", Language::TypeScript).unwrap()
    );
}

#[test]
fn malformed_source_preserves_which_adapter_failed() {
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
fn fingerprint_display_contains_compatibility_identity_and_digest() {
    let fingerprint = fingerprint_javascript("const value = true;").unwrap();
    let rendered = fingerprint.to_string();

    assert!(rendered.starts_with("structaura:v1:javascript:n1:"));
    let digest = rendered.rsplit(':').next().unwrap();
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
}
