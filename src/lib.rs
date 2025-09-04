//! A robust, layered configuration library for the Magic Block application.
//!
//! This library uses `figment`, `serde`, and `clap` to assemble a configuration
//! from multiple sources with a clear order of precedence.

pub mod config;
pub mod consts;
pub mod remote;
pub mod types;

use clap::{Parser, ValueEnum};
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    config::{
        AccountsDbConfig, ChainLinkConfig, ChainOperationConfig, CommitStrategy, LedgerConfig,
        ValidatorConfig,
    },
    remote::RemoteCluster,
    types::BindAddress,
};

//==============================================================================
// 1. Core Configuration Struct (`MagicBlockParams`)
//==============================================================================

/// Top-level configuration, assembled from multiple sources.
#[derive(Parser, Deserialize, Serialize, Debug, Default)]
#[serde(default, rename_all = "kebab-case")]
#[command(author, version, about, long_about = None)]
pub struct MagicBlockParams {
    /// Path to the TOML configuration file.
    #[arg(long, short, global = true)]
    pub config: Option<PathBuf>,

    /// Remote Solana cluster URL or a predefined alias (e.g., "mainnet").
    #[arg(long, short, default_value = consts::DEFAULT_REMOTE)]
    pub remote: RemoteCluster,

    /// The application's operational mode.
    #[arg(long, value_enum, default_value = consts::DEFAULT_LIFECYCLE)]
    pub lifecycle: LifecycleMode,

    /// Root directory for application storage (e.g., accounts, ledger).
    #[arg(long)]
    pub storage: Option<PathBuf>,

    /// Primary listen address for the main RPC service.
    #[arg(long, short, default_value = consts::DEFAULT_RPC_ADDR)]
    pub listen: BindAddress,

    /// Listen address for the metrics endpoint. If disabled, this is not set.
    #[arg(long, short)]
    pub metrics: Option<BindAddress>,

    /// Validator-specific arguments, flattened to the top level.
    #[clap(flatten)]
    pub validator: ValidatorConfig,

    // --- File-Only Configuration ---
    #[clap(skip)]
    pub commit: CommitStrategy,
    #[clap(skip)]
    pub accounts_db: AccountsDbConfig,
    #[clap(skip)]
    pub ledger: LedgerConfig,
    #[clap(skip)]
    pub chainlink: ChainLinkConfig,
    #[clap(skip)]
    pub chain_operation: Option<ChainOperationConfig>,
}

impl MagicBlockParams {
    /// Assembles the final configuration from all sources.
    ///
    /// The precedence is: Environment > CLI > TOML File > Defaults.
    pub fn try_new() -> figment::Result<Self> {
        let cli = Self::parse();

        let mut figment = Figment::new().join(Serialized::from(Self::default(), "default"));

        if let Some(path) = &cli.config {
            figment = figment.merge(Toml::file(path));
        }
        figment
            .merge(Serialized::from(&cli, "cli"))
            .merge(Env::prefixed(consts::ENV_VAR_PREFIX).split("_"))
            .extract()
    }
}

/// Defines the operational mode of the application.
#[derive(ValueEnum, Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleMode {
    /// Ephemeral Rollup mode for production.
    Ephemeral,
    /// Dev mode, cloning all state from a base chain.
    Replica,
    /// Offline mode without any base chain access.
    Offline,
    /// Clones only programs from a base chain.
    #[default]
    ProgramsReplica,
}
