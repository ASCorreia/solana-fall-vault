# Add per-transaction withdrawal limit

## What

Adds a `max_withdraw` cap to each vault. Any single withdrawal that
exceeds the cap is rejected before lamports move.

## Why

Without a limit, a vault can be fully drained in one transaction.
A per-transaction ceiling bounds the blast radius of a leaked keypair
or a misbehaving client — the attacker (or the bug) can only move
`max_withdraw` lamports per instruction, buying time to rotate keys
or freeze the vault.

The cap is per-vault, not a global constant, because different users
have different risk profiles. A treasury vault and a tip jar should
not share the same ceiling.

## How

### State — `vault_state.rs`

`max_withdraw: u64` added to `VaultState`, **after** the two bump
fields. Field order matters here: `test_initialize.rs` reads raw
account bytes and asserts `data[8] == vault_bump`, `data[9] == bump`
(Anchor's 8-byte discriminator occupies bytes 0–7, and Borsh
serialises fields in declaration order with no padding). Prepending
the new field would shift those offsets and silently break the
existing test. Appending keeps the layout stable.

`InitSpace` is derived, so the account space recalculates
automatically — no manual `space =` edit needed.

### Initialise — `initialize.rs` + `lib.rs`

`initialize_vault` takes a new `max_withdraw: u64` argument and
writes it into `set_inner`. The routing wrapper in `lib.rs` forwards
it. Making this an instruction argument rather than a constant means
the cap is a property of the vault, chosen at creation time.

### Error — `error.rs`

New variant `ExceedsMaxWithdraw` added **after** the existing
`CustomError` placeholder. Anchor assigns error codes by variant
index; deleting or reordering the placeholder would shift every code
downstream. Appending is the only safe option in a shared codebase.

### Withdraw — `withdraw.rs`

A single `require!` guard before the CPI transfer:

```rust
require!(
    amount <= ctx.accounts.vault_state.max_withdraw,
    ErrorCode::ExceedsMaxWithdraw
);
```

`<=` not `<` — the cap is a limit, not a strict bound. Withdrawing
exactly `max_withdraw` is allowed. The boundary test
(`withdraw_exactly_at_max_succeeds`) pins this down so a future
refactor can't accidentally flip the operator without a test failure.

## Testing

Three new tests in `test_withdraw.rs`, all using `max_withdraw = 2 SOL`:

| Test | Amount | Expected |
|------|--------|----------|
| `withdraw_under_max_succeeds` | 1 SOL | success |
| `withdraw_exactly_at_max_succeeds` | 2 SOL | success |
| `withdraw_over_max_fails` | 2 SOL + 1 lamport | rejected |

The middle test is the important one — it is the only input that
distinguishes `<=` from `<`. The third test uses `max_withdraw + 1`
(one lamport over), not some large round number, so it cannot pass
for an unrelated reason like insufficient vault funds.

All existing tests pass unchanged. Call sites in `test_initialize.rs`,
`test_deposit.rs`, and `test_withdraw.rs` now pass `100 * ONE_SOL`
as the cap — generous enough that the limit never interferes with
what those tests were written to check.

```
anchor build && cargo test
# 16 tests, 0 failures
```

## Files changed

| File | Touch |
|------|-------|
| `src/state/vault_state.rs` | +1 field |
| `src/error.rs` | +1 variant |
| `src/instructions/initialize.rs` | +1 param, updated `set_inner` |
| `src/lib.rs` | forwarded param |
| `src/instructions/withdraw.rs` | +1 `require!` guard, +1 import |
| `tests/common/mod.rs` | updated builders |
| `tests/test_initialize.rs` | updated call sites |
| `tests/test_deposit.rs` | updated call sites |
| `tests/test_withdraw.rs` | updated call sites + 3 new tests |
| `.gitignore` | added `node_modules/`, `.anchor/` |
