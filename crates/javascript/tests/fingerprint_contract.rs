use structaura_javascript::{JavaScriptError, fingerprint_javascript};

#[test]
fn formatting_and_comments_do_not_change_fingerprint() {
    let compact = fingerprint_javascript("const answer = (left + right);\n").unwrap();
    let formatted =
        fingerprint_javascript("// explanation\nconst answer = ( left + right ); /* trailing */\n")
            .unwrap();

    assert_eq!(compact, formatted);
}

#[test]
fn changing_an_identifier_changes_fingerprint() {
    let first = fingerprint_javascript("const answer = 42;").unwrap();
    let second = fingerprint_javascript("const result = 42;").unwrap();

    assert_ne!(first.digest(), second.digest());
}

#[test]
fn child_order_changes_fingerprint() {
    let first = fingerprint_javascript("first(); second();").unwrap();
    let second = fingerprint_javascript("second(); first();").unwrap();

    assert_ne!(first.digest(), second.digest());
}

#[test]
fn quote_style_does_not_change_simple_string_fingerprint() {
    let single = fingerprint_javascript("const message = 'hello';").unwrap();
    let double = fingerprint_javascript("const message = \"hello\";").unwrap();

    assert_eq!(single, double);
}

#[test]
fn malformed_source_returns_diagnostics() {
    let error = fingerprint_javascript("const = ;").unwrap_err();

    assert!(matches!(error, JavaScriptError::Syntax(diagnostics) if !diagnostics.is_empty()));
}

#[test]
fn fingerprint_identity_uses_javascript_language() {
    let fingerprint = fingerprint_javascript("const value = true;").unwrap();

    assert_eq!(fingerprint.identity().language(), "javascript");
    assert_eq!(fingerprint.identity().normalization_version(), 1);
}

#[test]
fn redundant_expression_parentheses_do_not_change_fingerprint() {
    let plain = fingerprint_javascript("const total = left + right;").unwrap();
    let grouped = fingerprint_javascript("const total = (left + right);").unwrap();

    assert_eq!(plain, grouped);
}

#[test]
fn optional_semicolons_do_not_change_fingerprint() {
    let with_semicolons = fingerprint_javascript("const value = 1;\nvalue += 2;").unwrap();
    let without_semicolons = fingerprint_javascript("const value = 1\nvalue += 2").unwrap();

    assert_eq!(with_semicolons, without_semicolons);
}

#[test]
fn operator_changes_change_fingerprint() {
    let addition = fingerprint_javascript("const value = left + right;").unwrap();
    let subtraction = fingerprint_javascript("const value = left - right;").unwrap();

    assert_ne!(addition.digest(), subtraction.digest());
}

#[test]
fn deeply_nested_valid_expression_fingerprints_without_recursive_traversal() {
    let depth = 1024;
    let source = format!("const value = {}{};", "[".repeat(depth), "]".repeat(depth));

    assert!(fingerprint_javascript(&source).is_ok());
}

#[test]
fn deeply_nested_malformed_source_returns_syntax_diagnostics() {
    let depth = 1024;
    let source = format!("const value = {}0;", "[".repeat(depth));

    let error = fingerprint_javascript(&source).unwrap_err();
    match error {
        JavaScriptError::Syntax(diagnostics) => {
            assert!(!diagnostics.is_empty());
            assert!(
                diagnostics
                    .windows(2)
                    .all(|pair| { pair[0].byte_range().start <= pair[1].byte_range().start })
            );
        }
        other => panic!("expected syntax diagnostics, got {other:?}"),
    }
}
