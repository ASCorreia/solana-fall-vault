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
const VAULT_STATE_SEED = Buffer.from("vault_state");
const VAULT_SEED = Buffer.from("vault");

function discriminator(name) {
  return crypto
    .createHash("sha256")
    .update("global:" + name)
    .digest()
    .subarray(0, 8);
}
function u64(v) {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(v));
  return b;
}
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

  const [vaultState] = vaultStatePda(payer.publicKey);
  const [vault] = vaultPda(payer.publicKey);

  // read max_withdraw cap from state (bytes 10..18 little-endian)
  const stateInfo = await conn.getAccountInfo(vaultState);
  if (!stateInfo) throw new Error("vault_state not found - not initialized");
  const maxWithdraw = Number(stateInfo.data.readBigUInt64LE(10));
  console.log("max_withdraw cap:", maxWithdraw / LAMPORTS_PER_SOL, "SOL");

  let guard = 0;
  while (true) {
    const vaultBal = await conn.getBalance(vault);
    if (vaultBal <= 5000) {
      console.log("vault fully drained. remaining:", vaultBal, "lamports (fee dust)");
      break;
    }
    if (++guard > 20) throw new Error("guard: too many iterations");

    const amount = Math.min(vaultBal, maxWithdraw);
    console.log("withdrawing", (amount / LAMPORTS_PER_SOL).toFixed(9), "SOL (vault at", (vaultBal / LAMPORTS_PER_SOL).toFixed(9) + ")");

    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true }, // user
        { pubkey: vault, isSigner: false, isWritable: true }, // vault
        { pubkey: vaultState, isSigner: false, isWritable: false }, // vault_state
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: Buffer.concat([discriminator("withdraw"), u64(amount)]),
    });
    const tx = new Transaction().add(ix);
    const sig = await conn.sendTransaction(tx, [payer], {
      preflightCommitment: "confirmed",
      maxRetries: 3,
    });
    await conn.confirmTransaction(sig, "confirmed");
    console.log("  done", sig);
  }
}

main().catch((e) => {
  console.error("ERROR:", e && e.message ? e.message : e);
  process.exit(1);
});
