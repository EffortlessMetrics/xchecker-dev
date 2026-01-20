//! Tests for create_default_config helper

use crate::cli::commands;
use crate::{CliArgs, Config};

#[test]
fn test_create_default_config() {
    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();
    let config_map = commands::create_default_config(true, &config, &cli_args);

    assert_eq!(config_map.get("verbose"), Some(&"true".to_string()));
    assert_eq!(
        config_map.get("packet_max_bytes"),
        Some(&"65536".to_string())
    );
    assert_eq!(
        config_map.get("packet_max_lines"),
        Some(&"1200".to_string())
    );
}

#[test]
fn test_create_default_config_no_verbose() {
    let cli_args = CliArgs::default();
    let config = Config::discover(&cli_args).unwrap();
    let config_map = commands::create_default_config(false, &config, &cli_args);

    assert!(!config_map.contains_key("verbose"));
    assert_eq!(
        config_map.get("packet_max_bytes"),
        Some(&"65536".to_string())
    );
    assert_eq!(
        config_map.get("packet_max_lines"),
        Some(&"1200".to_string())
    );
}
