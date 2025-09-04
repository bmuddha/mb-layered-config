# Magic Block Configuration

## Configuration Layering

The configuration is loaded from four distinct sources. Each source overrides any values set by the layers that come after it in the list.

The order of precedence is:

1.  **Environment Variables** (Highest precedence)
2.  **CLI Arguments**
3.  **TOML Configuration File**
4.  **Internal Defaults** (Lowest precedence)

### Environment Variables

Environment variables are prefixed with **`MBV_`** and override all other sources. Nested fields in the TOML file (like `[validator]`) are accessed using an underscore (`_`).

* `remote` -> `MBV_REMOTE`
* `validator.basefee` -> `MBV_VALIDATOR_BASEFEE`
* `validator.keypair` -> `MBV_VALIDATOR_KEYPAIR`

## TOML Configuration Examples

You can provide an optional configuration file using the `--config <path>` argument.

### Partial Configuration Example

You only need to specify the values you want to override from the defaults. Any unspecified fields will retain their default values.

```toml
# config.partial.toml

# Override the default RPC address and storage path
listen = "0.0.0.0:9000"
storage = "/var/lib/magic-block/data"

# Override only the basefee within the validator section
[validator]
basefee = 5000
````

### Full Configuration Example

This example shows all available fields that can be set via the TOML file.

```toml
# config.full.toml

# Top-level settings
remote = "mainnet"
lifecycle = "ephemeral"
storage = "/var/lib/magic-block/full-storage"
listen = "0.0.0.0:8080"
metrics = "127.0.0.1:9100"

# Validator-specific settings
[validator]
basefee = 100
keypair = "9Vo7TbA5YfC5a3...pMc3gTgBQ"

# Settings for on-chain operations and identity
[chain-operation]
country-code = "GE"
fqdn = "[https://my-validator.ge](https://my-validator.ge)"
claim-fees-frequency = 86400 # 1 day in seconds

# Transaction commit strategy
[commit]
compute-unit-price = 50000

# Ledger database settings
[ledger]
blocks-per-partition = 2097152 # 2 * 1024 * 1024
block-time = "500ms" # Using human-readable duration
reset = false

# Accounts database settings
[accounts-db]
database-size = 10737418240 # 10 GB
block-size = "Block512"
index-size = 104857600 # 100 MB
max-snapshots = 10
snapshot-frequency = 50000

# Chainlink integration settings
[chainlink]
prepare-lookup-tables = true
auto-airdrop-lamports = 1000000000 # 1 SOL
max-monitored-accounts = 50
```

## Override Examples

These scenarios demonstrate the layering system, building from the simplest case to a full override permutation. We will use the **partial config example** as our `config.toml` for these scenarios.

-----

### Scenario 1: Defaults Only

This is the most basic case: running the binary with no arguments, environment variables, or config file. The entire configuration comes from the internal defaults.

**Command:**

```bash
cargo run
```

**Result (Partial):**

```json
{
  "remote": "https://api.devnet.solana.com",
  "lifecycle": "programs-replica",
  "storage": null,
  "listen": "127.0.0.1:8899",
  "validator": {
    "basefee": 0,
    "keypair": "9Vo7TbA5Y...QcJ4XziRFpMc3gTgBQ"
  }
}
```

-----

### Scenario 2: TOML Overrides Defaults

Here, we provide a config file. The values from the file will override the internal defaults. Note that `remote` is not in our partial `config.toml`, so it still comes from the defaults.

**Command:**

```bash
cargo run -- --config config.toml
```

**Result (Relevant Sections):**

```json
{
  "remote": "https://api.devnet.solana.com/",
  "listen": "0.0.0.0:9000",
  "storage": "/var/lib/magic-block/data",
  "validator": {
    "basefee": 5000
  }
}
```

-----

### Scenario 3: CLI Overrides TOML

Now, we add a CLI argument. This argument will override any value set in the TOML file or the defaults.

**Command:**

```bash
cargo run -- --config config.toml --listen "1.2.3.4:80"
```

**Result (Relevant Sections):**

```json
{
  "remote": "https://api.devnet.solana.com",
  "listen": "1.2.3.4:80",
  "validator": {
    "basefee": 5000
  }
}
```

*Analysis: The `--listen` flag on the command line overrode the `"0.0.0.0:9000"` value from the TOML file. The other TOML values remain.*

-----

### Scenario 4: Environment Variable Overrides CLI and TOML

This demonstrates the highest precedence. An environment variable will override a value even if it's also set via a CLI flag and a TOML file.

**Command:**

```bash
# Set an env var for basefee, but also provide a CLI flag for it.
MBV_VALIDATOR_BASEFEE="99999" \
cargo run -- --config config.toml --basefee 123
```

**Result (Relevant Sections):**

```json
{
  "validator": {
    "basefee": 99999
  }
}
```

*Analysis: The environment variable `MBV_VALIDATOR_BASEFEE` won, overriding both the CLI flag (`--basefee 123`) and the TOML value (`basefee = 5000`).*

-----

### Scenario 5: Full Layered Permutation

This final example shows all four layers working together to produce the final configuration.

**Command:**

```bash
MBV_LISTEN="10.0.0.1:443" \
MBV_LIFECYCLE="offline" \
cargo run -- \
  --config config.toml \
  --remote "testnet"
```

**Result (Full):**

```json
{
  "remote": "https://api.testnet.solana.com",
  "lifecycle": "offline",
  "storage": "/var/lib/magic-block/data",
  "listen": "10.0.0.1:443",
  "metrics": null,
  "validator": {
    "basefee": 5000,
    "keypair": "9Vo7TbA5Y...iRFpMc3gTgBQ"
  },
  "commit": { "compute-unit-price": 1000000 },
  "accounts-db": { "database-size": 104857600, "block-size": "Block256", "index-size": 1048576, "max-snapshots": 4, "snapshot-frequency": 1024 },
  "ledger": { "blocks-per-partition": 1048576, "block-time": "400ms", "reset": true },
  "chainlink": { "prepare-lookup-tables": false, "auto-airdrop-lamports": 0, "max-monitored-accounts": 0 },
  "chain-operation": null
}
```

**Analysis of Final Values:**

  * **`listen: "10.0.0.1:443"`**

      * **Source:** Environment Variable (`MBV_LISTEN`).
      * **Reason:** Highest precedence; overrides the TOML's `"0.0.0.0:9000"` and the default.

  * **`lifecycle: "offline"`**

      * **Source:** Environment Variable (`MBV_LIFECYCLE`).
      * **Reason:** Highest precedence; overrides the default of `"programs-replica"`.

  * **`remote: "https://api.testnet.solana.com/"`**

      * **Source:** CLI Argument (`--remote`).
      * **Reason:** Overrides the default of `"devnet"`.

  * **`storage: "/var/lib/magic-block/data"`**

      * **Source:** TOML File.
      * **Reason:** Overrides the default of `null`.

  * **`basefee: 5000`**

      * **Source:** TOML File.
      * **Reason:** Overrides the default of `0`.

  * **`keypair: "9Vo7..."`**

      * **Source:** Default.
      * **Reason:** Was not specified in any other layer.

