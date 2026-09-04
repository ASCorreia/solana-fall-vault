const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
} = require("@solana/web3.js");
const fs = require("fs");
const crypto = require("crypto");

const PROGRAM_ID = new PublicKey("FhhGq5patKiMzg5b7AMF51zWDoND2d48YisBgYkbifz1");
const CLUSTER = "https://api.devnet.solana.com";

// Anchor instruction discriminators = first 8 bytes of sha256("global:<name>")
function discriminator(name) {
  return crypto
    .createHash("sha256")
    .update("global:" + name)
    .digest()
    .subarray(0, 8);
}

// borsh-serialize a u64 (little-endian)
function u64(v) {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(v));
  return b;
}

const VAULT_STATE_SEED = Buffer.from("vault_state");
const VAULT_SEED = Buffer.from("vault");

function vaultStatePda(user) {
  return PublicKey.findProgramAddressSync([VAULT_STATE_SEED, user.toBuffer()], PROGRAM_ID);
}
function vaultPda(user) {
  return PublicKey.findProgramAddressSync([VAULT_SEED, user.toBuffer()], PROGRAM_ID);
}

async function main() {
  const walletPath = process.argv[2] || "~/.config/solana/id.json";
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  const payer = Keypair.fromSecretKey(Uint8Array.from(secret));

  const conn = new Connection(CLUSTER, "confirmed");
  const lamports = await conn.getBalance(payer.publicKey);
  console.log("payer:", payer.publicKey.toBase58(), "balance:", lamports / LAMPORTS_PER_SOL, "SOL");

  if (lamports < 0.1 * LAMPORTS_PER_SOL) {
    throw new Error("Insufficient devnet SOL in wallet - need at least 0.1 SOL");
  }

  const [vaultState, bump] = vaultStatePda(payer.publicKey);
  const [vault, vaultBump] = vaultPda(payer.publicKey);
  console.log("vault_state:", vaultState.toBase58(), "(bump", bump + ")");
  console.log("vault      :", vault.toBase58(), "(bump", vaultBump + ")");

  const ONE_SOL = LAMPORTS_PER_SOL;
  const maxWithdraw = 2 * ONE_SOL; // per-transaction cap = 2 SOL

  async function sendIx(name, ix) {
    const tx = new Transaction().add(ix);
    const sig = await conn.sendTransaction(tx, [payer], {
      preflightCommitment: "confirmed",
      maxRetries: 3,
    });
    const conf = await conn.confirmTransaction(sig, "confirmed");
    if (conf.value.err) throw new Error(name + " failed: " + JSON.stringify(conf.value.err));
    console.log("  OK  ", name, sig);
    return sig;
  }

  function ixAccounts(meta) {
    return meta.map(([pubkey, isSigner, isWritable]) => ({
      pubkey: new PublicKey(pubkey),
      isSigner,
      isWritable,
    }));
  }

  console.log("\n--- 1. initialize(max_withdraw=2 SOL) ---");
  // attempt only if vault_state does not exist yet
  const existing = await conn.getAccountInfo(vaultState);
  if (existing && existing.owner.equals(PROGRAM_ID)) {
    console.log("  vault already initialized - skipping create (data length", existing.data.length + ")");
  } else {
    const initIx = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: ixAccounts([
        [payer.publicKey, true, true], // user (signer)
        [vault, false, true], // vault (order: struct field order)
        [vaultState, false, true], // vault_state (init)
        [SystemProgram.programId, false, false], // system_program
      ]),
      data: Buffer.concat([discriminator("initialize"), u64(maxWithdraw)]),
    });
    await sendIx("initialize", initIx);
  }

  // read back the account to verify max_withdraw persisted
  const state = await conn.getAccountInfo(vaultState);
  if (state) {
    // bytes: 0..8 discriminator, 8 bump, 9 vault_bump, 10..18 max_withdraw (u64)
    const storedMax = state.data.readBigUInt64LE(10);
    console.log("  stored max_withdraw:", Number(storedMax / BigInt(ONE_SOL)), "SOL (raw:", storedMax.toString() + ")");
    console.log("  expected           :", Number(maxWithdraw / ONE_SOL), "SOL");
  }

  console.log("\n--- 2. deposit(3 SOL) ---");
  const depositAmt = 3 * ONE_SOL;
  const depIx = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: ixAccounts([
      [payer.publicKey, true, true], // user
      [vault, false, true], // vault
      [vaultState, false, false], // vault_state
      [SystemProgram.programId, false, false],
    ]),
    data: Buffer.concat([discriminator("deposit"), u64(depositAmt)]),
  });
  await sendIx("deposit(3 SOL)", depIx);
  console.log("  vault balance:", (await conn.getBalance(vault)) / ONE_SOL, "SOL");

  console.log("\n--- 3. withdraw(1 SOL) within cap ---");
  const w1 = 1 * ONE_SOL;
  const wIx1 = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: ixAccounts([
      [payer.publicKey, true, true], // user
      [vault, false, true], // vault
      [vaultState, false, false], // vault_state
      [SystemProgram.programId, false, false],
    ]),
    data: Buffer.concat([discriminator("withdraw"), u64(w1)]),
  });
  await sendIx("withdraw(1 SOL)", wIx1);
  console.log("  vault balance:", (await conn.getBalance(vault)) / ONE_SOL, "SOL");

  console.log("\n--- 4. withdraw(3 SOL) OVER cap -> MUST FAIL ---");
  const w2 = 3 * ONE_SOL;
  const wIx2 = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: ixAccounts([
      [payer.publicKey, true, true],
      [vault, false, true],
      [vaultState, false, false],
      [SystemProgram.programId, false, false],
    ]),
    data: Buffer.concat([discriminator("withdraw"), u64(w2)]),
  });
  const tx2 = new Transaction().add(wIx2);
  try {
    const sig2 = await conn.sendTransaction(tx2, [payer], {
      preflightCommitment: "confirmed",
      maxRetries: 2,
      skipPreflight: false,
    });
    const conf2 = await conn.confirmTransaction(sig2, "confirmed");
    if (conf2.value.err) {
      console.log("  EXPECTED FAILURE:", JSON.stringify(conf2.value.err));
    } else {
      console.log("  *UNEXPECTED* withdraw over cap succeeded!!");
    }
  } catch (e) {
    const msg = (e && e.message) || "";
    const m = msg.match(/WithdrawalAmountExceeds|exceeds the maximum|0x[\da-f]+|custom program error/i);
    console.log("  EXPECTED FAILURE caught:", (m && m[0]) || msg.slice(0, 200));
  }
  console.log("  vault balance after:", (await conn.getBalance(vault)) / ONE_SOL, "SOL");

  console.log("\nDone. Program:", PROGRAM_ID.toBase58());
}

main().catch((e) => {
  console.error("ERROR:", e && e.message ? e.message : e);
  process.exit(1);
});
