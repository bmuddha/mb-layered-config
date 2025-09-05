//! Integration tests for the Magic Block configuration layering.
//!
//! To run these tests:
//! 1. Create a `tests` directory next to your `src` directory.
//! 2. Save this file as `tests/config_layering.rs`.
//! 3. Run `cargo test`.

use magicblock_config::LifecycleMode;
use magicblock_config::{consts, remote::RemoteCluster, MagicBlockParams};
use std::env;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

/// Helper function to build a TOML config file in a temporary directory.
fn create_toml_config(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("config.toml");
    let mut file = File::create(&path).expect("Failed to create temp config file");
    writeln!(file, "{}", content).expect("Failed to write to temp config file");
    (dir, path)
}

/// Simulates the configuration assembly process for testing.
fn assemble_config_from_simulated_sources(cli_args: Vec<&str>) -> MagicBlockParams {
    MagicBlockParams::try_new(cli_args.into_iter().map(Into::into))
        .expect("Failed to assemble config for test")
}

#[test]
fn test_defaults_only() {
    let argv = vec!["magic-block"];
    let config = assemble_config_from_simulated_sources(argv);

    assert_eq!(config.remote, consts::DEFAULT_REMOTE.parse().unwrap());
    assert_eq!(config.listen.0.to_string(), consts::DEFAULT_RPC_ADDR);
    assert_eq!(config.validator.basefee, consts::DEFAULT_BASE_FEE);
    assert_eq!(
        config.validator.keypair,
        consts::DEFAULT_VALIDATOR_KEYPAIR.parse().unwrap()
    );
}

#[test]
fn test_toml_overrides_cli_defaults() {
    let toml_content = r#"
        listen = "0.0.0.0:9999"
        remote = "mainnet"
        [validator]
        basefee = 5000
    "#;
    let (_dir, config_path) = create_toml_config(toml_content);
    let argv = vec!["magic-block", "--config", config_path.to_str().unwrap()];

    let config = assemble_config_from_simulated_sources(argv);

    // Values from TOML
    assert_eq!(config.listen.0.to_string(), "0.0.0.0:9999");
    assert_eq!(config.remote, "mainnet".parse().unwrap());
    assert_eq!(config.validator.basefee, 5000);
    // Value from Default (not in TOML)
    assert_eq!(config.lifecycle, LifecycleMode::ProgramsReplica);
}

#[test]
fn test_cli_overrides_defaults() {
    // No TOML file is used in this test.
    let argv = vec!["magicblock", "--remote", "localhost", "--basefee", "123"];

    let config = assemble_config_from_simulated_sources(argv);

    // Value from CLI
    assert_eq!(config.remote, "localhost".parse().unwrap());
    assert_eq!(config.validator.basefee, 123);
    // Value from Default
    assert_eq!(config.lifecycle, LifecycleMode::ProgramsReplica);
}

#[test]
fn test_env_overrides_toml_and_cli() {
    // Set environment variables that should win.
    env::set_var("MBV_REMOTE", "testnet");
    env::set_var("MBV_VALIDATOR_BASEFEE", "99999");

    let toml_content = r#"
        remote = "mainnet"
        [validator]
        basefee = 5000
    "#;
    let (_dir, config_path) = create_toml_config(toml_content);
    let argv = vec![
        "magic-block",
        "--config",
        config_path.to_str().unwrap(),
        "--remote", // This CLI arg should be overridden by TOML, which is then overridden by ENV.
        "localhost",
    ];

    let config = assemble_config_from_simulated_sources(argv);

    // Clean up environment variables immediately
    env::remove_var("MBV_REMOTE");
    env::remove_var("MBV_VALIDATOR_BASEFEE");

    // Values from ENV (highest precedence)
    assert_eq!(config.remote, "testnet".parse::<RemoteCluster>().unwrap());
    assert_eq!(config.validator.basefee, 99999);
}

#[test]
fn test_full_permutation_scenario() {
    // Layer 1: Environment (Highest precedence)
    env::set_var("MBV_LISTEN", "10.0.0.1:443");
    env::set_var("MBV_LIFECYCLE", "offline");

    // Layer 2: TOML File
    let toml_content = r#"
        # This listen value will be overridden by the ENV var.
        listen = "0.0.0.0:9000"
        # This basefee value will win as it's not set in ENV.
        [validator]
        basefee = 5000
    "#;
    let (_dir, config_path) = create_toml_config(toml_content);

    // Layer 3: CLI Arguments
    let argv = vec![
        "magic-block",
        "--config",
        config_path.to_str().unwrap(),
        // This remote value will win as it's not set in ENV or TOML.
        "--remote",
        "mainnet",
    ];

    let config = assemble_config_from_simulated_sources(argv);

    env::remove_var("MBV_LISTEN");
    env::remove_var("MBV_LIFECYCLE");

    // Assert values based on the precedence: TOML > Env > CLI > Defaults
    // Highest precedence: TOML file
    assert_eq!(config.listen.0.to_string(), "10.0.0.1:443");
    assert_eq!(config.validator.basefee, 5000);
    // Second highest precedence: Environment variables
    assert_eq!(config.lifecycle, LifecycleMode::Offline);
    // Third highest precedence: CLI arguments
    assert_eq!(config.remote, "mainnet".parse().unwrap());
    // Lowest precedence: Default (keypair was never set anywhere else)
    assert_eq!(
        config.validator.keypair,
        consts::DEFAULT_VALIDATOR_KEYPAIR.parse().unwrap()
    );
}
