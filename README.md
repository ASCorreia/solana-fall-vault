# Solana Fall Vault

An [Anchor](https://www.anchor-lang.com/) program for Solana that implements a simple lamports (SOL) vault with a **per-transaction withdrawal limit** (`max_withdraw`).

The vault stores SOL on behalf of a user and is protected by a configurable per-transaction withdrawal cap. This cap bounds the damage a compromised key, a buggy client, or a malicious UI could do in a single withdrawal - a defense-in-depth measure inspired by "withdrawal limits" found in traditional and DeFi custody systems.

## What it does

- **Initialize** a vault on-chain with a chosen `max_withdraw` cap (in lamports).
- **Deposit** SOL into the vault.
- **Withdraw** SOL from the vault, subject to the per-transaction `max_withdraw` cap. Any single withdrawal larger than the cap is rejected.
- **Close** the vault and reclaim rent (included in the program).

## Repository layout

```
Anchor.toml                         Cluster + wallet + program key configuration
programs/lamports-vault/src/
  lib.rs                            Program entrypoint and instruction dispatch
  state/vault_state.rs              VaultState account layout (incl. max_withdraw)
  state.rs                          re-exports
  instructions/initialize.rs        Initialize handler
  instructions/deposit.rs           Deposit handler
  instructions/withdraw.rs          Withdraw handler (enforces max_withdraw)
  instructions/close.rs             Close handler
  error.rs                          Custom error codes
  constants.rs                      Seed constants
programs/lamports-vault/tests/
  common/mod.rs                     Shared test helpers (LiteSVM setup)
  test_initialize.rs                Initialize tests
  test_deposit.rs                   Deposit tests
  test_withdraw.rs                  Withdraw + max_withdraw boundary tests
scripts/deploy-test/
  test_deploy.js                    Live devnet smoke test (devnet)
  withdraw_all.js                   Drains the vault on devnet
```

## Features added

This fork adds the `max_withdraw` cap to the original upstream vault:

- **`state/vault_state.rs`** - `VaultState` gains a `max_withdraw: u64` field.
- **`instructions/initialize.rs`** - `initialize` now accepts and persists a `max_withdraw` argument.
- **`instructions/withdraw.rs`** - `withdraw` rejects any amount `> vault_state.max_withdraw` with the custom error `WithdrawalExceedsMaxLimit`.
- **`error.rs`** — adds the `WithdrawalExceedsMaxLimit` error variant.

## State layout

The `VaultState` account data layout (little-endian):

| Offset | Size | Field                         |
|--------|------|-------------------------------|
| 0      | 8    | Anchor account discriminator  |
| 8      | 1    | `bump`                         |
| 9      | 1    | `vault_bump`                   |
| 10     | 8    | **`max_withdraw`** (u64)        |

## Build & test

Requires `anchor-cli` (1.1.x), `solana-cli`, and Rust.

```bash
# Build the program (generates target/deploy/lamports_vault.so)
anchor build

# Run the full unit + integration test suite (LiteSVM, no chain needed)
cargo test

# Formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

The test suite includes boundary tests for the new feature:

- withdraw **under** the cap → succeeds
- withdraw **at** the cap → succeeds
- withdraw **one lamport over** the cap → fails with `WithdrawalExceedsMaxLimit`

## Deploy to devnet

```bash
# Generate a fresh program key (or reuse an existing one)
anchor keys generate lamports_vault

# Sync the program ID into lib.rs + Anchor.toml
anchor keys sync

# Deploy to the cluster configured in Anchor.toml (devnet)
anchor program deploy target/deploy/lamports_vault.so
```

The IDL is intentionally not deployed (keeps the address space minimal for this assignment).

## Live on-chain smoke test

`scripts/deploy-test/test_deploy.js` sends real transactions to the deployed devnet
program to verify the feature end-to-end. See [`docs/DEVNET_TESTING.md`](docs/DEVNET_TESTING.md)
for the full run details, addresses, and transaction listing.

```bash
cd scripts/deploy-test
npm install @solana/web3.js@1 bs58
node test_deploy.js /path/to/wallet-keypair.json   # default: ~/.config/solana/id.json
```

## Program address (devnet)

- **Program ID:** [`FhhGq5patKiMzg5b7AMF51zWDoND2d48YisBgYkbifz1`](https://explorer.solana.com/address/FhhGq5patKiMzg5b7AMF51zWDoND2d48YisBgYkbifz1?cluster=devnet)
- **Cluster:** `https://api.devnet.solana.com`
