#[test]
fn accesskit_feature_is_declared_when_backend_is_compiled() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    assert!(manifest.contains("accesskit = []") || manifest.contains("accesskit = ["));
}
