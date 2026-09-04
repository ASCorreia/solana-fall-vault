# Deploy-test scripts

Live smoke tests for the deployed devnet program. They send **real transactions** on the
Solana devnet cluster and require a funded devnet wallet.

## Prerequisites

```bash
npm install @solana/web3.js@1 bs58
```

## Scripts

### `test_deploy.js` - end-to-end smoke test

Runs the full scenario against devnet:

1. `initialize(max_withdraw)` - creates the PDAs.
2. Reads back `max_withdraw` to confirm it persisted.
3. `deposit(3 SOL)`.
4. `withdraw(1 SOL)` - **under** the cap (succeeds).
5. `withdraw(over cap)` - **must be rejected** (proves the cap works; funds untouched).

```bash
node test_deploy.js /path/to/wallet-keypair.json
# default wallet: ~/.config/solana/id.json
```

`max_withdraw`, the deposit amount, and the over-cap attempt are constants near the top of
the file.

### `withdraw_all.js` - drain the vault

Withdraws every lamport from the vault, automatically splitting into multiple transactions
when the remaining balance exceeds the cap (i.e. it demonstrates the per-transaction limit
by draining in ≤ `max_withdraw` chunks).

```bash
node withdraw_all.js /path/to/wallet-keypair.json
```

## Note

Because the over-cap withdrawal fails during preflight simulation, its transactions are not
recorded on-chain. All successful transactions, signatures, and slot numbers are listed in
`docs/DEVNET_TESTING.md`.
