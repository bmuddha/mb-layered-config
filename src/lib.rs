use std::{fmt, net::SocketAddr, path::PathBuf, str::FromStr, time::Duration};

use clap::Parser;
use derive_more::FromStr;
use isocountry::CountryCode;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use serde_with::{DisplayFromStr, serde_as};
use solana_pubkey::Pubkey;
use url::Url;

#[derive(Parser, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
struct MagicBlockParams {
    #[serde(skip)]
    config: PathBuf,
    #[arg(long, short)]
    remote: RemoteCluster,
    lifecycle: LifecycleMode,
    #[clap(skip)]
    commit: CommitStrategy,
    storage: PathBuf,
    #[clap(skip)]
    accounts_db: AccountsDbConfig,
    #[clap(skip)]
    ledger: LedgerConfig,
    #[clap(skip)]
    chainlink: ChainLinkConfig,
    #[arg(long, short)]
    listen: BindAddress,
    #[arg(long, short)]
    metrics: Option<BindAddress>,
    #[clap(skip)]
    solana_interaction: Option<SolanaInteraction>,
    #[clap(flatten)]
    validator: ValidatorConfig,
}

#[derive(Parser, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub struct ValidatorConfig {
    #[arg(long)]
    pub base_fee: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SolanaInteraction {
    pub country_code: CountryCode,
    pub fqdn: Url,
    #[serde(with = "humantime")]
    pub claim_fees_frequency: Duration,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LedgerConfig {
    pub blocks_per_partition: usize,
    #[serde(with = "humantime")]
    pub block_time: Duration,
    pub reset: bool,
}
impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            blocks_per_partition: 1024 * 1024,
            block_time: Duration::from_millis(50),
            reset: true,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct BindAddress(pub SocketAddr);

impl FromStr for BindAddress {
    type Err = <SocketAddr as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl Default for BindAddress {
    fn default() -> Self {
        "0.0.0.0:8899".parse().unwrap()
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ChainLinkConfig {
    pub prepare_lookup_tables: bool,
    pub auto_airdrop_lamports: u64,
    pub max_monitored_accounts: usize,
}

#[derive(Deserialize)]
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

#[derive(Deserialize, Default)]
pub enum BlockSize {
    Block128 = 128,
    #[default]
    Block256 = 256,
    Block512 = 512,
}

#[derive(Deserialize, Clone)]
pub struct CommitStrategy {
    /// The compute unit price offered when we send the commit account transaction
    /// This is in micro lamports and defaults to `1_000_000` (1 Lamport)
    pub compute_unit_price: u64,
}

impl Default for CommitStrategy {
    fn default() -> Self {
        Self {
            compute_unit_price: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, FromStr)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleMode {
    Ephemeral,
    Replica,
    #[default]
    ProgramsReplica,
    Offline,
}

#[derive(Deserialize, Clone)]
pub enum RemoteCluster {
    Single(Remote),
    Multiple(Vec<Remote>),
}

#[serde_as]
#[derive(Deserialize, Clone)]
pub enum Remote {
    Unified(#[serde_as(as = "DisplayFromStr")] AliasedUrl),
    Disjointed {
        #[serde_as(as = "DisplayFromStr")]
        http: AliasedUrl,
        #[serde_as(as = "DisplayFromStr")]
        ws: AliasedUrl,
    },
}

impl FromStr for RemoteCluster {
    type Err = <AliasedUrl as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AliasedUrl::from_str(s).map(|s| RemoteCluster::Single(Remote::Unified(s)))
    }
}

impl Default for RemoteCluster {
    fn default() -> Self {
        Self::Single(Remote::Unified("devnet".parse().unwrap()))
    }
}
#[derive(Clone)]
pub struct AliasedUrl(pub Url);

impl FromStr for AliasedUrl {
    type Err = <Url as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Url::parse("https://api.mainnet-beta.solana.com"),
            "devnet" => Url::parse("https://api.devnet.solana.com"),
            "testnet" => Url::parse("https://api.testnet.solana.com"),
            "localhost" | "dev" => Url::parse("http://127.0.0.1:8899"),
            custom => Url::parse(custom),
        }
        .map(Self)
    }
}

pub struct SerdePubkey(pub Pubkey);

impl<'de> Deserialize<'de> for SerdePubkey {
    /// Deserializes a Base58 encoded string into a 32-byte array.
    /// It returns an error if the decoded data is not exactly 32 bytes.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Serde32BytesVisitor;

        impl Visitor<'_> for Serde32BytesVisitor {
            type Value = SerdePubkey;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Base58 string representing a 32-byte array")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let mut buffer = [0u8; 32];
                let decoded_len = bs58::decode(value)
                    .onto(&mut buffer)
                    .map_err(de::Error::custom)?;
                if decoded_len != 32 {
                    return Err(de::Error::custom(format!(
                        "expected 32 bytes, got {}",
                        decoded_len
                    )));
                }
                Ok(SerdePubkey(Pubkey::new_from_array(buffer)))
            }
        }
        deserializer.deserialize_str(Serde32BytesVisitor)
    }
}
