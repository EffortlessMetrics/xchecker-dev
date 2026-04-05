use xchecker_utils::secure_read::secure_read_to_string;

#[test]
fn test_secure_read_integration() {
    let result = secure_read_to_string("Cargo.toml");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("[workspace]"));
}
