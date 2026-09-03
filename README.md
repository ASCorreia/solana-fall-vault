# lamports-vault

An Anchor program implementing a per-user lamports vault with deposit,
withdraw, and close instructions.

## What

Adds a per-transaction withdrawal limit to the lamports vault.

## Why

The vault currently allows draining the full balance in a single
withdraw. A per-transaction cap bounds the damage from a leaked key
or a buggy client.

## How

- `max_withdraw: u64` appended to `VaultState` (appended, not
  prepended, so the byte offsets `test_initialize` asserts on stay valid)
- set from a new `initialize` argument
- `withdraw` rejects `amount > max_withdraw` with `WithdrawalExceedsLimit`
  (the check runs before any lamports move, and `amount == max_withdraw`
  is allowed)

## Testing

`anchor build && cargo test` — all existing tests pass, plus three new
ones covering under, exactly at, and one lamport over the cap.
