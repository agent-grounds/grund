//! Enforce the approved interpreter/ABI boundary (§FS-distribution.3.3.7).

fn main() {
    let config = pyo3_build_config::get();
    assert!(
        matches!(
            config.implementation,
            pyo3_build_config::PythonImplementation::CPython
        ),
        "grund supports CPython 3.10+ only"
    );
    assert!(
        !config.is_free_threaded(),
        "grund requires CPython with the GIL"
    );
}
