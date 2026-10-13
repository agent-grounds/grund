/// AR-rust: Rust router
///
/// ## 1. Contract
///
/// Routes requests from Rust.
pub fn route() -> &'static str {
    // Implements §FS-entry.1.
    // See §FS-gone-rust.
    let one = "data §api/FS-str-rust";
    let many = r#"first line
data §api/FS-multi-rust
"#;
    let _ = (one, many);
    "ok"
}
