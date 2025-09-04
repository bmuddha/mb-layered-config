//! A magical blockchain application with a robust, layered configuration system.
//!
//! This application demonstrates how to use `figment`, `serde`, and `clap` together
//! to load configuration from defaults, a TOML file, and command-line arguments.
//!
//! Run with `--help` to see all available command-line options.

use clap::{Parser, ValueEnum};
use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use isocountry::CountryCode;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, DisplayFromStr, DurationSeconds, SerializeDisplay, serde_as};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use std::{
    convert::Infallible,
    fmt::{self, Debug, Display},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
    time::Duration,
};
use url::Url;

//==============================================================================
// 1. Core Configuration Struct (`MagicBlockParams`)
//==============================================================================

/// The top-level configuration for the Magic Block application.
///
/// This struct orchestrates the entire configuration, pulling values from a TOML file
/// and overriding them with command-line arguments where specified.
#[derive(Parser, Deserialize, Serialize, Debug, Default)]
#[serde(default, rename_all = "kebab-case")]
#[command(author, version, about, long_about = None)]
pub struct MagicBlockParams {
    /// Path to the TOML configuration file.
    #[arg(long, short, global = true)]
    pub config: Option<PathBuf>,

    /// The remote Solana cluster to connect to.
    ///
    /// Can be a URL or a predefined alias (e.g., "mainnet", "devnet").
    /// In the config file, this can be a complex object with separate http/ws URLs.
    #[arg(long, short, default_value = "devnet")]
    pub remote: RemoteCluster,

    /// The operational mode for the application's lifecycle.
    #[arg(long, default_value = "programs-replica")]
    pub lifecycle: LifecycleMode,

    /// The root directory for application storage (e.g., accounts, ledger).
    #[arg(long)]
    pub storage: Option<PathBuf>,

    /// The primary listen address for the application's main RPC service.
    #[arg(long, short, default_value = "127.0.0.1:8899")]
    pub listen: BindAddress,

    /// The listen address for the metrics endpoint. If not provided, metrics are disabled.
    #[arg(long, short)]
    pub metrics: Option<BindAddress>,

    /// Exposes validator-specific configurations as top-level CLI arguments.
    #[clap(flatten)]
    pub validator: ValidatorConfig,

    // --- File-Only Configuration ---
    // The following fields use `#[clap(skip)]` to hide them from the CLI.
    // They can only be configured via the TOML file.
    #[clap(skip)]
    pub commit: CommitStrategy,

    #[clap(skip)]
    pub accounts_db: AccountsDbConfig,

    #[clap(skip)]
    pub ledger: LedgerConfig,

    #[clap(skip)]
    pub chainlink: ChainLinkConfig,

    #[clap(skip)]
    pub solana_interaction: Option<SolanaInteraction>,
}

//==============================================================================
// 2. CLI-Exposed & File-Exposed Configuration Sections
//==============================================================================

/// Defines the operational mode of the application.
#[derive(ValueEnum, Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleMode {
    /// Runs in production ready Ephemeral Rollup mode
    Ephemeral,
    /// Runs in dev mode, where everything is cloned from base chain.
    Replica,
    /// Runs in offline mode without any base chain access
    Offline,
    /// Runs in a mode that only clones programs from base chain.
    #[default]
    ProgramsReplica,
}

/// Configuration for the validator behavior.
///
/// Using `#[clap(flatten)]` on the parent struct makes these arguments
/// appear at the top level of the CLI (e.g., `--base-fee` instead of `--validator.base-fee`).
#[derive(Parser, Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub struct ValidatorConfig {
    /// The base fee in lamports for transactions processed by this validator.
    #[arg(long)]
    pub base_fee: Option<u64>,
    #[arg(long, short)]
    pub keypair: Option<SerdeKeypair>,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        const DEFAULT_KEYPAIR: &str = "9Vo7TbA5YfC5a33JhAi9Fb41usA6JwecHNRw3f9MzzHAM8hFnXTzL5DcEHwsAFjuUZ8vNQcJ4XziRFpMc3gTgBQ";
        Self {
            base_fee: Some(0),
            keypair: Some(SerdeKeypair(Keypair::from_base58_string(DEFAULT_KEYPAIR))),
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
    /// The compute unit price in micro-lamports offered for commit transactions.
    pub compute_unit_price: u64,
}

impl Default for CommitStrategy {
    fn default() -> Self {
        Self {
            compute_unit_price: 1_000_000,
        }
    }
}

/// Configuration related to on-chain interactions and validator identity.
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SolanaInteraction {
    /// The validator's two-letter country code for location-based services (e.g., "US").
    pub country_code: CountryCode,
    /// The validator's fully qualified domain name (FQDN).
    pub fqdn: Url,
    /// How often to claim fees from the chain, specified in seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub claim_fees_frequency: Duration,
}

/// Configuration for the ledger database.
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct LedgerConfig {
    pub blocks_per_partition: usize,
    /// Target time per block, in seconds (can be fractional).
    #[serde_as(as = "DurationSeconds<f64>")]
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

/// Defines the block size for the accounts DB.
#[derive(Deserialize, Serialize, Debug, Default, Clone, Copy)]
pub enum BlockSize {
    Block128 = 128,
    #[default]
    Block256 = 256,
    Block512 = 512,
}

//==============================================================================
// 4. Remote Cluster Types & Parsers
//==============================================================================

/// Represents a connection to one or more remote clusters.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteCluster {
    Single(Remote),
    Multiple(Vec<Remote>),
}

/// The CLI parser for `--remote`. It only supports the simplest case:
/// a single, unified URL. This keeps the CLI user experience clean.
impl FromStr for RemoteCluster {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AliasedUrl::from_str(s).map(|url| Self::Single(Remote::Unified(url)))
    }
}

impl Default for RemoteCluster {
    fn default() -> Self {
        Self::Single(Remote::Unified(
            "devnet".parse().expect("Default URL should be valid"),
        ))
    }
}

/// Represents a connection to a single remote node.
#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Remote {
    /// A single URL for both HTTP and WebSocket connections.
    Unified(#[serde_as(as = "DisplayFromStr")] AliasedUrl),
    /// Separate URLs for HTTP and WebSocket connections.
    Disjointed {
        #[serde_as(as = "DisplayFromStr")]
        http: AliasedUrl,
        #[serde_as(as = "DisplayFromStr")]
        ws: AliasedUrl,
    },
}

/// A URL that can be aliased with shortcuts like "mainnet" or "devnet".
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct AliasedUrl(pub Url);

impl Display for AliasedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AliasedUrl {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_str = match s {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "localhost" | "dev" => "http://127.0.0.1:8899",
            custom => custom,
        };
        Url::parse(url_str).map(Self)
    }
}

impl TryFrom<String> for AliasedUrl {
    type Error = url::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

//==============================================================================
// 5. Reusable Utility Types
//==============================================================================

/// A network bind address that can be parsed from a string (e.g., "0.0.0.0:8080").
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BindAddress(pub SocketAddr);

impl FromStr for BindAddress {
    type Err = std::net::AddrParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl Default for BindAddress {
    fn default() -> Self {
        "0.0.0.0:8899".parse().unwrap()
    }
}

/// A wrapper for `solana_pubkey::Pubkey` to enable deserializing from Base58.
#[derive(Clone, Debug, DeserializeFromStr, SerializeDisplay)]
pub struct SerdePubkey(pub Pubkey);

impl FromStr for SerdePubkey {
    type Err = solana_pubkey::ParsePubkeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl Display for SerdePubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A wrapper for `solana_signer::keypair::Keypair` to enable Serde.
///
/// Serializes to and from a Base58 encoded string of the keypair's secret key.
#[derive(DeserializeFromStr, SerializeDisplay)]
pub struct SerdeKeypair(pub Keypair);

impl Clone for SerdeKeypair {
    fn clone(&self) -> Self {
        Self(self.0.insecure_clone())
    }
}
impl FromStr for SerdeKeypair {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Keypair::from_base58_string(s)))
    }
}

impl Display for SerdeKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_base58_string())
    }
}
impl Debug for SerdeKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_base58_string())
    }
}

//==============================================================================
// 6. Main Application Logic
//==============================================================================

fn main() -> Result<(), figment::Error> {
    // Start by parsing arguments provided on the command line.
    let cli_args = MagicBlockParams::parse();

    // Create a Figment instance to begin layering configuration sources.
    let mut figment = Figment::new()
        // Layer 1: Start with the struct's `Default` implementation.
        .merge(Serialized::defaults(&cli_args));

    // Layer 2: If a config file path is provided via `--config`, load and merge it.
    if let Some(config_path) = &cli_args.config {
        // In a real app, you might want to handle file-not-found errors gracefully here.
        if config_path.exists() {
            println!(
                "--- Loading from config file: {} ---",
                config_path.to_string_lossy()
            );
            figment = figment.join(Toml::file(config_path));
        } else {
            eprintln!(
                "Warning: Config file not found at '{}', skipping.",
                config_path.to_string_lossy()
            );
        }
    } else {
        println!("--- No config file specified, using defaults and CLI args only ---");
    }

    // Layer 3: Merge the command-line arguments. Any args provided will
    // override values from the config file and the defaults.
    figment = figment.join(Serialized::from(MagicBlockParams::default(), "defaults"));

    // Extract the final, layered configuration into our struct.
    let config: MagicBlockParams = figment.extract()?;

    println!("\n--- Final Configuration ---");
    println!("{:?}", config);

    Ok(())
}
