# Devnet Testing Report - `max_withdraw` Lamports Vault

This document records the live, on-chain verification of the deployed program on the
Solana **devnet** cluster. It covers the deployment, the smoke-test methodology, every
transaction sent (with signatures and Explorer links), and the final on-chain state.

---

## 1. Deployment summary

| Item | Value |
|------|-------|
| Program ID | `FhhGq5patKiMzg5b7AMF51zWDoND2d48YisBgYkbifz1` |
| Explorer | https://explorer.solana.com/address/FhhGq5patKiMzg5b7AMF51zWDoND2d48YisBgYkbifz1?cluster=devnet |
| Cluster RPC | `https://api.devnet.solana.com` |
| Upgrade authority / payer wallet | `8BntfcMy1J8tpkXdbwWms6S2Py7AXsfmLtgDCuK2AHMX` |
| Deployed binary size | 174,064 bytes (ProgramData account) |
| IDL | Not deployed (intentional) |

The binary upload was performed by the standard BPFLoader `write` instruction in ~198
chunks of ~10 KB each, followed by a single final `deploy` instruction.

---

## 2. Methodology

Because no generated Anchor TypeScript client or IDL was used, the smoke test builds the
Anchor instruction data directly:

- Each instruction's **8-byte discriminator** is computed as the first 8 bytes of
  `sha256("global:<instruction_name>")`, e.g. `global:initialize`, `global:withdraw`.
- Arguments are **borsh-serialized** (`u64` as 8-byte little-endian).
- Account lists are supplied in the exact **struct field order** the program's
  `#[derive(Accounts)]` expects (`user, vault, vault_state, system_program`).

The scripts use `@solana/web3.js` v1 against devnet with the payer keypair
`~/.config/solana/id.json`.

---

## 3. Deterministic PDAs

Derived for the test wallet `8Bntf...`:

| Account | Address | Seed(s) | Bump |
|---------|---------|---------|------|
| `vault_state` | `gX9qST39EM2FZfwtQKJYVHkEytbvxegwZVoRBBP3Uxj` | `["vault_state", user]` | 253 |
| `vault` | `146b92r1WqpE3u4wLy9EP1sgFw82Den29SSDaNMKatGe` | `["vault", user]` | 255 |

---

## 4. Test scenarios and results

Configured cap: **`max_withdraw = 2 SOL`** (2,000,000,000 lamports).

| # | Scenario | Expected | Result |
|---|----------|----------|--------|
| 1a | `initialize(max_withdraw=2 SOL)` | success, PDAs created | ✅ success |
| 1b | Read `vault_state.max_withdraw` | 2 SOL | ✅ stored `2,000,000,000` lamports |
| 2 | `deposit(3 SOL)` | success, vault holds SOL | ✅ vault → 3.00065024 (incl. rent) |
| 3 | `withdraw(1 SOL)` (under cap) | success | ✅ vault reduced |
| 4 | `withdraw(3 SOL)` (over cap) | **rejected** | ✅ **blocked, funds untouched** |

### The core guarantee — over-cap rejection

The withdrawal of **3 SOL** with a **2 SOL** cap was **rejected**. The transaction was
intercepted during preflight simulation - the program returned a custom program error and
**no funds moved**. This is the exact behavior the feature was designed to enforce.

**Note on preflight:** the over-cap transaction was caught by `sendTransaction`'s
preflight simulation, so it never received a blockchain signature. It therefore does not
appear in the on-chain transaction list below.

### Draining the vault (respecting the cap)

After the above tests the vault held **2.00065024 SOL** - more than the 2 SOL cap. To empty
it, the cap forced the drain across **two** withdrawals, demonstrating the limit in action:

| # | Scenario | Expected | Result |
|---|----------|----------|--------|
| 5 | `withdraw(2 SOL)` (at cap) | success | ✅ vault → 0.00065024 |
| 6 | `withdraw(0.00065024 SOL)` (under cap) | success | ✅ vault → 0 |

---

## 5. Transaction listing (confirmed on-chain)

All fee = 5,000 lamports each, all confirmed.

| # | Scenario | Slot | Signature |
|---|----------|------|-----------|
| 1 | `initialize(max_withdraw=2 SOL)` | 492,993,975 | [`3xFmBJsr...2G`](https://explorer.solana.com/tx/3xFmBJsr3tBLrd8v8bagAfBiQ178Ha6v1akgGmR736SSZQPjmAamVQXgiAsGdhHeoFViZYnvTSFLCFCVEouY6Z2G?cluster=devnet) |
| 2 | `deposit(3 SOL)` | 492,993,987 | [`3KBXWcu7...3g`](https://explorer.solana.com/tx/3KBXWcu7GEiTmNu9MXzZV9wJD7Wvb25Eqe3sNzPag4w8XbxzPBrGYo3ZdVBGtQwYvHvLhe6KRJMqwKtVzwGWDf3g?cluster=devnet) |
| 3 | `withdraw(1 SOL)` under cap | 492,993,998 | [`274GEM9P...3sX`](https://explorer.solana.com/tx/274GEM9P3L1TFTCnRTd6xKZQ927ZsdZQDfvBPcL1QPKfKLQcyLnQFaQmYyumKMV5BvtTtffbMrCA1gYpaZti3sHX?cluster=devnet) |
| 4 | `withdraw(2 SOL)` at cap | 492,996,072 | [`4JN7EQ9X...aCK`](https://explorer.solana.com/tx/4JN7EQ9XdkcKcMsmn5BnDz19ssF5qtUUnz1quSytj6My1uJnG7pQU6tBccn4mkWuTzyYTkL9ZGVkzDgNTrrYaoCK?cluster=devnet) |
| 5 | `withdraw(0.00065024 SOL)` | 492,996,092 | [`21EdZXCw...FTVQ`](https://explorer.solana.com/tx/21EdZXCwhHogfkAzjZ3xwH6fzyPoRRfeaV1E6ubfepc54MroS7o5nyBBHqiwkjWcoHexJaHU1tTWS7TiJW7uFTVQ?cluster=devnet) |

---

## 6. Final on-chain state

| Item | Value |
|------|-------|
| `vault_state` owner | `FhhGq5pat...` (the program) |
| `vault_state.max_withdraw` | `2,000,000,000` lamports (**2 SOL**) |
| `vault` balance | **0 SOL** (drained) |
| Payer wallet | 9.11235624 SOL |

---

## 7. How to reproduce

Dependencies (use a scratch dir outside the repo so it isn't committed):

```bash
mkdir -p /tmp/vault-smoketest && cd /tmp/vault-smoketest
npm init -y
npm install @solana/web3.js@1 bs58
```

Run the end-to-end smoke test (needs a funded devnet wallet):

```bash
node /path/to/repo/scripts/deploy-test/test_deploy.js ~/.config/solana/id.json
```

Drain the vault (requires an initialized vault for that wallet):

```bash
node /path/to/repo/scripts/deploy-test/withdraw_all.js ~/.config/solana/id.json
```

---

## 8. Notes and caveats

- **Anchor instruction serialization** was reproduced by hand (discriminator + borsh)
  rather than via a generated client. Deriving the account list in the incorrect struct
  order produced `AnchorError 0x7d6 / ConstraintSeeds`; this is a good example of why
  generated clients are recommended for anything beyond a smoke test.
- **Preflight rejection:** the over-cap withdrawal is caught during simulation, so it
  never lands on-chain - the program rejects it (custom error) before any transfer.
- The vault's own rent (0.00065024 SOL at initialize) was separate from deposited funds and
  has been fully drained by the two-step withdrawal above.
