use std::fmt;

use js_sys::Error as JsError;
use structaura_core::{
    Fingerprint, FingerprintIdentity, SyntaxNodeRecord, canonicalize_syntax_nodes,
    compare_fingerprints as compare_core_fingerprints, fingerprint as fingerprint_core,
};
use wasm_bindgen::prelude::*;

const NORMALIZATION_VERSION: u32 = 1;

/// Versioned fingerprint value exposed to JavaScript.
#[wasm_bindgen]
pub struct WasmFingerprint {
    inner: Fingerprint,
}

impl WasmFingerprint {
    fn new(inner: Fingerprint) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl WasmFingerprint {
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn digest(&self) -> String {
        self.inner.digest().iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn language(&self) -> String {
        self.inner.identity().language().to_owned()
    }

    #[wasm_bindgen(getter = formatVersion)]
    pub fn format_version(&self) -> u32 {
        u32::from(self.inner.format_version())
    }

    #[wasm_bindgen(getter = normalizationVersion)]
    pub fn normalization_version(&self) -> u32 {
        self.inner.identity().normalization_version()
    }
}

/// Accepts flat preorder parser records so the parser runtime remains in JavaScript
/// while canonicalization, Merkle hashing, and identity construction remain in Rust.
#[wasm_bindgen(js_name = fingerprintSyntaxNodes)]
pub fn fingerprint_syntax_nodes(
    records: JsValue,
    language: &str,
) -> Result<WasmFingerprint, JsValue> {
    if !matches!(language, "javascript" | "typescript") {
        return Err(js_error(format!(
            "unsupported language '{language}'; expected 'javascript' or 'typescript'"
        )));
    }

    let records: Vec<SyntaxNodeRecord> = serde_wasm_bindgen::from_value(records)
        .map_err(|error| js_error(format!("invalid syntax records: {error}")))?;
    let canonical = canonicalize_syntax_nodes(&records).map_err(js_error)?;
    let identity = FingerprintIdentity::new(language, NORMALIZATION_VERSION).map_err(js_error)?;

    Ok(WasmFingerprint::new(fingerprint_core(&canonical, identity)))
}

#[wasm_bindgen(js_name = compareFingerprints)]
pub fn compare_fingerprints(
    left: &WasmFingerprint,
    right: &WasmFingerprint,
) -> Result<bool, JsValue> {
    compare_core_fingerprints(&left.inner, &right.inner).map_err(js_error)
}

fn js_error(error: impl fmt::Display) -> JsValue {
    JsError::new(&error.to_string()).into()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::fingerprint_syntax_nodes;
    use js_sys::{Array, Object, Reflect};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    fn record(kind: &str, text: &str, is_named: bool, children: u32) -> JsValue {
        let object = Object::new();
        Reflect::set(&object, &"syntax_kind".into(), &kind.into()).unwrap();
        Reflect::set(&object, &"text".into(), &text.into()).unwrap();
        Reflect::set(&object, &"is_named".into(), &is_named.into()).unwrap();
        Reflect::set(&object, &"child_count".into(), &JsValue::from_f64(f64::from(children)))
            .unwrap();
        object.into()
    }

    #[wasm_bindgen_test]
    fn canonicalizes_flat_records_and_rejects_bad_shapes() {
        let records = Array::new();
        records.push(&record("program", "", true, 1));
        records.push(&record("identifier", "answer", true, 0));
        let fingerprint = fingerprint_syntax_nodes(records.into(), "javascript").unwrap();
        assert!(fingerprint.value().starts_with("structaura:v1:javascript:n1:"));
        assert!(fingerprint_syntax_nodes(Array::new().into(), "javascript").is_err());
        assert!(fingerprint_syntax_nodes(Array::new().into(), "rust").is_err());
    }
}
