# Magic Block Configuration

## Configuration Layering

The configuration is loaded from four distinct sources. Each source overrides any values set by the layers that come before it in the list.

The order of precedence is:

1.  **Internal Defaults** (Lowest precedence)
2.  **CLI Arguments**
3.  **TOML Configuration File**
4.  **Environment Variables** (Highest precedence)

## Command-Line Arguments & Help

All available command-line arguments, their environment variable fallbacks, and default values are listed below.

```text
Top-level configuration, assembled from multiple sources

Usage: magicblock-config [OPTIONS]

Options:
  -c, --config <CONFIG>
          Path to the TOML configuration file
          [env: MBV_CONFIG=]

  -r, --remote <REMOTE>
          Remote Solana cluster URL or a predefined alias (e.g., "mainnet")
          [env: MBV_REMOTE=]
          [default: devnet]

      --lifecycle <LIFECYCLE>
          The application's operational mode
          
          Possible values:
          - ephemeral:        Ephemeral Rollup mode for production
          - replica:          Dev mode, cloning all state from a base chain
          - offline:          Offline mode without any base chain access
          - programs-replica: Clones only programs from a base chain
          
          [env: MBV_LIFECYCLE=]
          [default: programs-replica]

      --storage <STORAGE>
          Root directory for application storage (e.g., accounts, ledger)
          [env: MBV_STORAGE=]

  -l, --listen <LISTEN>
          Primary listen address for the main RPC service
          [env: MBV_LISTEN=]
          [default: 127.0.0.1:8899]

  -m, --metrics <METRICS>
          Listen address for the metrics endpoint. If disabled, this is not set
          [env: MBV_METRICS=]

      --basefee <BASEFEE>
          Base fee in lamports for transactions
          [env: MBV_BASEFEE=]

  -k, --keypair <KEYPAIR>
          The validator's identity keypair, encoded in Base58
          [env: MBV_KEYPAIR=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
````

## Override Examples

These scenarios demonstrate the layering system, building from the simplest case to a full override permutation. We will use the following `config.toml` as our baseline file.

**Baseline `config.toml`:**

```toml
# config.toml
listen = "0.0.0.0:9000"
remote = "mainnet"

[validator]
basefee = 5000
```

-----

### Scenario 1: Defaults Only

Running the binary with no arguments, environment variables, or config file.

**Command:**

```bash
cargo run
```

**Result (Partial):**

```json
{
  "remote": "https://api.devnet.solana.com/",
  "lifecycle": "programs-replica",
  "listen": "127.0.0.1:8899",
  "validator": { "basefee": null }
}
```

*Analysis: The entire configuration comes from the internal defaults set by `clap/serde`.*

-----

### Scenario 2: CLI Overrides Defaults

CLI arguments are the second layer, overriding the defaults.

**Command:**

```bash
cargo run -- --remote localhost --lifecycle ephemeral
```

**Result (Relevant Sections):**

```json
{
  "remote": "http://127.0.0.1:8899/",
  "lifecycle": "ephemeral",
  "listen": "127.0.0.1:8899"
}
```

*Analysis: `remote` and `lifecycle` are taken from the CLI, overriding their defaults. `listen` still comes from the default.*

-----

### Scenario 3: TOML Overrides CLI

The TOML file is the third layer, overriding any values set by the CLI arguments or defaults.

**Command:**

```bash
# The `--remote` flag is set, but the TOML file also sets `remote`.
cargo run -- --config config.toml --remote localhost
```

**Result (Relevant Sections):**

```json
{
  "remote": "https://api.mainnet-beta.solana.com",
  "listen": "0.0.0.0:9000",
  "validator": { "basefee": 5000 }
}
```

*Analysis: `remote` and `listen` are taken from `config.toml`, overriding both the CLI flag (`--remote localhost`) and their respective defaults.*

-----

### Scenario 4: Environment Variable Overrides All

Environment variables are the final layer and have the highest precedence. They will override values from the TOML file, CLI, and defaults.

**Command:**

```bash
# Set env vars for remote and basefee.
MBV_REMOTE="testnet" \
MBV_VALIDATOR_BASEFEE="99999" \
cargo run -- --config config.toml --remote localhost
```

**Result (Relevant Sections):**

```json
{
  "remote": "https://api.testnet.solana.com/",
  "listen": "0.0.0.0:9000",
  "validator": { "basefee": 99999 }
}
```

**Analysis of Final Values:**

  * **`remote: "testnet"`**

      * **Source:** Environment Variable (`MBV_REMOTE`).
      * **Reason:** Highest precedence; overrides the CLI flag (`localhost`) and the TOML value (`mainnet`).

  * **`basefee: 99999`**

      * **Source:** Environment Variable (`MBV_VALIDATOR_BASEFEE`).
      * **Reason:** Highest precedence; overrides the TOML value (`5000`).

  * **`listen: "0.0.0.0:9000"`**

      * **Source:** TOML File.
      * **Reason:** No environment variable was set for `listen`, so the TOML value wins over the CLI default.

