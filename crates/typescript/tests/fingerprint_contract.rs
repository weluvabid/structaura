use structaura_typescript::{TypeScriptError, fingerprint_typescript};

#[test]
fn formatting_and_comments_do_not_change_fingerprint() {
    let compact = fingerprint_typescript("const answer: number = (left + right);\n").unwrap();
    let formatted = fingerprint_typescript(
        "// explanation\nconst answer : number = ( left + right ); /* trailing */\n",
    )
    .unwrap();

    assert_eq!(compact, formatted);
}

#[test]
fn changing_an_identifier_changes_fingerprint() {
    let first = fingerprint_typescript("const answer: number = 42;").unwrap();
    let second = fingerprint_typescript("const result: number = 42;").unwrap();

    assert_ne!(first.digest(), second.digest());
}

#[test]
fn type_annotation_changes_fingerprint() {
    let number = fingerprint_typescript("const value: number = 1;").unwrap();
    let string = fingerprint_typescript("const value: string = 1;").unwrap();

    assert_ne!(number.digest(), string.digest());
}

#[test]
fn generic_type_arguments_are_preserved() {
    let string = fingerprint_typescript("const values: Array<string> = [];").unwrap();
    let number = fingerprint_typescript("const values: Array<number> = [];").unwrap();

    assert_ne!(string.digest(), number.digest());
}

#[test]
fn interface_member_changes_fingerprint() {
    let first = fingerprint_typescript("interface User { id: string }").unwrap();
    let second = fingerprint_typescript("interface User { id: number }").unwrap();

    assert_ne!(first.digest(), second.digest());
}

#[test]
fn type_aliases_and_union_members_are_structural() {
    let first = fingerprint_typescript("type Result = string | number;").unwrap();
    let second = fingerprint_typescript("type Result = string | boolean;").unwrap();

    assert_ne!(first.digest(), second.digest());
}

#[test]
fn string_quote_style_does_not_change_simple_string_fingerprint() {
    let single = fingerprint_typescript("const message: string = 'hello';").unwrap();
    let double = fingerprint_typescript("const message: string = \"hello\";").unwrap();

    assert_eq!(single, double);
}

#[test]
fn malformed_source_returns_diagnostics() {
    let error = fingerprint_typescript("const value: = 1;").unwrap_err();

    assert!(matches!(error, TypeScriptError::Syntax(diagnostics) if !diagnostics.is_empty()));
}

#[test]
fn fingerprint_identity_uses_typescript_language() {
    let fingerprint = fingerprint_typescript("const value: boolean = true;").unwrap();

    assert_eq!(fingerprint.identity().language(), "typescript");
    assert_eq!(fingerprint.identity().normalization_version(), 1);
}

#[test]
fn optional_semicolons_do_not_change_fingerprint() {
    let with_semicolons = fingerprint_typescript("const value: number = 1;\nvalue += 2;").unwrap();
    let without_semicolons = fingerprint_typescript("const value: number = 1\nvalue += 2").unwrap();

    assert_eq!(with_semicolons, without_semicolons);
}

#[test]
fn operator_changes_change_fingerprint() {
    let addition = fingerprint_typescript("const value: number = left + right;").unwrap();
    let subtraction = fingerprint_typescript("const value: number = left - right;").unwrap();

    assert_ne!(addition.digest(), subtraction.digest());
}
