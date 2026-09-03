# Lamports Vault

Per-user SOL vault on Solana. Each wallet owns one vault PDA that holds native lamports, plus a `VaultState` PDA that stores the withdrawal cap and bump seeds.

Built with [Anchor 1.1](https://www.anchor-lang.com/). No SPL tokens — deposits and withdrawals are `system_program::transfer` of lamports.

**Deployed Program in Devnet:** [`8hJ48kuf3NYwQ7L4Z92HFQLwyExxVrYLjQwALbqbH98V`](https://explorer.solana.com/address/8hJ48kuf3NYwQ7L4Z92HFQLwyExxVrYLjQwALbqbH98V?cluster=devnet)

Configured for `localnet` and `devnet` in `Anchor.toml`.

## How it works

```
user (signer)
  │
  ├── vault            PDA ["vault",       user]   — holds SOL
  └── vault_state      PDA ["vault_state", user]   — VaultState account
```

`VaultState` layout (after the 8-byte Anchor discriminator):

| Offset | Field          | Type | Notes                                      |
|--------|----------------|------|--------------------------------------------|
| 8..16  | `max_withdraw` | u64  | Per-tx withdrawal cap, set at `initialize` |
| 16     | `vault_bump`   | u8   | Bump for the vault PDA                     |
| 17     | `bump`         | u8   | Bump for the vault_state PDA               |

Ownership is baked into the PDAs: both seeds include the user's pubkey, so Alice cannot deposit into / withdraw from / close Bob's vault.

### Instructions

| Instruction  | Args           | What it does                                                                                       |
|--------------|----------------|----------------------------------------------------------------------------------------------------|
| `initialize` | `max_withdraw` | Creates `vault_state`, seeds `vault` with rent-exempt lamports. One vault per user.                |
| `deposit`    | `amount`       | Transfers `amount` lamports from the user to their vault.                                          |
| `withdraw`   | `amount`       | PDA-signed transfer from vault to user.                                                            |
| `close`      | —              | Returns remaining vault lamports to the user and closes `vault_state` (rent refunded to the user). |

### Constraints

- `amount > 0` on deposit and withdraw (`VaultError::ZeroAmount`)
- `amount <= max_withdraw` on withdraw (`VaultError::MaxWidthdraw`)
- `amount <= vault.lamports()` on withdraw (`VaultError::NotEnoughBalance`)
- Re-`initialize` of the same user fails (`init` requires a fresh account)
- Deposit / withdraw without a prior `initialize` fail (missing `vault_state`)

`max_withdraw` is immutable after init. To change it you'd close and re-initialize.

Withdraws are capped **per instruction**, not per epoch. A user with `max_withdraw = 1 SOL` and 10 SOL in the vault can withdraw 1 SOL ten times.

## Project layout

```
programs/lamports-vault/
  src/
    lib.rs                  program entrypoints
    state/vault_state.rs    VaultState account
    instructions/           initialize, deposit, withdraw, close
    error.rs                VaultError
    constants.rs            PDA seeds
  tests/                    LiteSVM unit tests
    common/mod.rs           SVM setup + ix builders
    test_initialize.rs
    test_deposit.rs
    test_withdraw.rs
scripts/interaction-test.ts live-cluster smoke test (devnet by default)
```

## Prerequisites

| Tool       | Version used here              |
|------------|--------------------------------|
| Rust       | 1.98.0 (`rust-toolchain.toml`) |
| Solana CLI | 3.0.1                          |
| Anchor CLI | 1.1.x                          |
| Node       | 18+                            |
| Yarn       | 1.22                           |

```bash
# Solana + Anchor (if you don't already have them)
sh -c "$(curl -sSfL https://release.anza.xyz/v3.0.1/install)"
cargo install --git https://github.com/coral-xyz/anchor --tag v1.1.2 anchor-cli --locked

solana --version
anchor --version
```

## Build

```bash
yarn install
anchor build
```

This produces `target/deploy/lamports_vault.so` and `target/idl/lamports_vault.json`. LiteSVM tests load the `.so` at runtime, so **build before you test**.

## Test

In-process unit tests via [LiteSVM](https://github.com/LiteSVM/litesvm) — no validator required:

```bash
anchor build
cargo test --manifest-path programs/lamports-vault/Cargo.toml -- --nocapture
```

(`anchor test` is aliased to `cargo test` in `Anchor.toml`.)

Covered:

- init creates `vault_state` + rent-seeds `vault`; second init fails; independent users get independent PDAs
- deposit increases vault balance; deposits accumulate; zero amount fails; over-balance fails; deposit without init fails
- withdraw returns lamports; zero / above-cap / above-balance fail; withdraw without init fails; wrong user cannot drain another vault

`close` currently has no LiteSVM coverage.

## Deploy

```bash
# local
solana-test-validator   # separate terminal
anchor deploy --provider.cluster localnet

# devnet
solana config set --url devnet
solana airdrop 2
anchor deploy --provider.cluster devnet
```

Program id is fixed in `programs/lamports-vault/src/lib.rs` (`declare_id!`) and `Anchor.toml`. If you generate a new keypair, update both.

## Live interaction script

`scripts/interaction-test.ts` talks to a real cluster with `@solana/web3.js` (no Anchor client). It uses `~/.config/solana/id.json`.

```bash
# defaults: PROGRAM_ID from declare_id!, RPC = https://api.devnet.solana.com
npx ts-node --esm scripts/interaction-test.ts

# override
PROGRAM_ID=<pubkey> SOLANA_RPC_URL=http://127.0.0.1:8899 npx ts-node --esm scripts/interaction-test.ts
```

What it asserts, in order:

1. `initialize` (skipped if `vault_state` already exists)
2. `deposit(0)` must fail
3. `deposit(2 SOL)` succeeds, vault delta matches
4. `withdraw(0)` must fail
5. `withdraw(1 SOL)` succeeds, vault/user balances move the right way

It does **not** call `close`. If you already initialized with a different `max_withdraw`, that value sticks until you close.

## Calling it yourself

Account order for every instruction: `user`, `vault`, `vault_state`, `system_program`.

PDA derivation:

```ts
const PROGRAM_ID = new PublicKey("8hJ48kuf3NYwQ7L4Z92HFQLwyExxVrYLjQwALbqbH98V");

const [vault] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), user.toBuffer()],
  PROGRAM_ID,
);
const [vaultState] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault_state"), user.toBuffer()],
  PROGRAM_ID,
);
```

Instruction data is the 8-byte Anchor discriminator (`sha256("global:<name>")[0..8]`) plus a little-endian `u64` for `initialize` / `deposit` / `withdraw`. See `scripts/interaction-test.ts`.

## Errors

| Code | Name               | When                                   |
|------|--------------------|----------------------------------------|
| 6000 | `ZeroAmount`       | deposit or withdraw with `amount == 0` |
| 6001 | `MaxWidthdraw`     | withdraw `amount > max_withdraw`       |
| 6002 | `NotEnoughBalance` | withdraw `amount > vault.lamports()`   |
