# Lamports Vault

A minimal SOL vault built with Anchor for the Solana Fall School curriculum. Each user gets their own vault, derived as a PDA from their public key, and moves lamports in and out through four instructions.

## Instructions

`initialize` creates a vault for the signer. It opens two PDAs: a `SystemAccount` that holds the deposited lamports, seeded with `[b"vault", user]`, and a `VaultState` account that records both PDA bumps plus the withdrawal cap, seeded with `[b"vault_state", user]`. The instruction funds the vault with enough lamports to stay rent-exempt.

`deposit` transfers lamports from the signer into their vault. Anyone who knows the seeds can compute the PDA, but only the account owner who initialized it can later withdraw from it, since `withdraw` and `close` derive the vault bump from the caller's own `vault_state`.

`withdraw` moves lamports back out of the vault to the signer, capped by `max_withdraw` (see below).

`close` returns every lamport left in the vault to the signer and closes both PDAs.

## Withdrawal cap

`initialize` now takes a `max_withdraw: u64` argument, stored on `VaultState` right after the two PDA bumps. `withdraw` checks the requested amount against it before moving any funds and rejects anything over the cap with `ExceedsMaxWithdraw`. The cap is set once, at initialization, and applies per transaction rather than to a running total. A user who wants a higher ceiling has to close the vault and open a new one.

`tests/test_withdraw.rs` covers the boundary directly: a withdrawal under the cap succeeds, one exactly at the cap succeeds, and one lamport over it fails with `ExceedsMaxWithdraw`.

## Building and testing

```bash
anchor build
cargo test
```

Run `anchor build` first every time you touch the program. The test harness embeds the compiled `.so` file at compile time, so a stale build makes `cargo test` run old code instead of failing outright.
