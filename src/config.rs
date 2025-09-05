use crate::consts;
use crate::types::SerdeKeypair;
use clap::Parser;
use isocountry::CountryCode;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{alloc::GlobalAlloc, time::Duration};
use url::Url;

//==============================================================================
// 2. CLI-Exposed & File-Exposed Configuration Sections
//==============================================================================

/// Configuration for the validator behavior.
#[derive(Parser, Deserialize, Serialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub struct ValidatorConfig {
    /// Base fee in lamports for transactions.
    #[arg(long, env = "MBV_BASEFEE", default_value = consts::DEFAULT_BASE_FEE_STR)]
    pub basefee: u64,

    /// The validator's identity keypair, encoded in Base58.
    #[arg(long, short, env = "MBV_KEYPAIR", default_value = consts::DEFAULT_VALIDATOR_KEYPAIR)]
    pub keypair: SerdeKeypair,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            basefee: consts::DEFAULT_BASE_FEE,
            keypair: SerdeKeypair(solana_keypair::Keypair::from_base58_string(
                consts::DEFAULT_VALIDATOR_KEYPAIR,
            )),
        }
    }
}

//==============================================================================
// 3. File-Only Configuration Sections
//==============================================================================

/// Defines the strategy for committing transactions to the ledger.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CommitStrategy {
    /// Compute unit price in micro-lamports for commit transactions.
    pub compute_unit_price: u64,
}

impl Default for CommitStrategy {
    fn default() -> Self {
        Self {
            compute_unit_price: 1_000_000,
        }
    }
}

/// Configuration for on-chain operations and validator identity.
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ChainOperationConfig {
    /// Validator's two-letter country code (e.g., "US").
    pub country_code: CountryCode,
    /// Validator's fully qualified domain name (FQDN).
    pub fqdn: Url,
    /// How often to claim fees from the chain
    #[serde(with = "humantime")]
    pub claim_fees_frequency: Duration,
}

/// Configuration for the ledger database.
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct LedgerConfig {
    pub blocks_per_partition: usize,
    /// Target time per blocks
    #[serde(with = "humantime")]
    pub block_time: Duration,
    pub reset: bool,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            blocks_per_partition: 1024 * 1024,
            block_time: Duration::from_millis(400),
            reset: true,
        }
    }
}

/// Configuration specific to ChainLink oracle integration.
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ChainLinkConfig {
    pub prepare_lookup_tables: bool,
    pub auto_airdrop_lamports: u64,
    pub max_monitored_accounts: usize,
}

/// Configuration for the accounts database.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AccountsDbConfig {
    pub database_size: usize,
    pub block_size: BlockSize,
    pub index_size: usize,
    pub max_snapshots: u16,
    pub snapshot_frequency: u64,
}

impl Default for AccountsDbConfig {
    fn default() -> Self {
        Self {
            block_size: BlockSize::Block256,
            database_size: 100 * 1024 * 1024,
            index_size: 1024 * 1024,
            max_snapshots: 4,
            snapshot_frequency: 1024,
        }
    }
}

/// Block size for the accounts DB.
#[derive(Deserialize, Serialize, Debug, Default, Clone, Copy)]
pub enum BlockSize {
    Block128 = 128,
    #[default]
    Block256 = 256,
    Block512 = 512,
}
