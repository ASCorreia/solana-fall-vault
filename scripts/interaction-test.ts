import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const PROGRAM_ID = new PublicKey(
  process.env.PROGRAM_ID ?? "8hJ48kuf3NYwQ7L4Z92HFQLwyExxVrYLjQwALbqbH98V",
);
const VAULT_SEED = Buffer.from("vault");
const VAULT_STATE_SEED = Buffer.from("vault_state");

const CLUSTER_URL =
  process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";

const WALLET_PATH = path.join(process.env.HOME ?? "", ".config/solana/id.json");

function u64le(x: bigint): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(x);
  return b;
}

function anchorDiscriminator(ixName: string): Buffer {
  const preimage = `global:${ixName}`;
  const hash = crypto.createHash("sha256").update(preimage).digest();
  return hash.subarray(0, 8);
}

function pda(seeds: Buffer[], programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

async function expectTxFail<T>(fn: () => Promise<T>, label: string) {
  try {
    await fn();
    throw new Error(`${label}: expected failure but tx succeeded`);
  } catch (e: any) {
    const msg = String(e?.message ?? e);
    console.log(`${label}: failed as expected`);
    console.log(msg.split("\n").slice(0, 8).join("\n"));
  }
}

async function main() {
  const connection = new Connection(CLUSTER_URL, "confirmed");

  const secret = JSON.parse(fs.readFileSync(WALLET_PATH, "utf8"));
  const wallet = Keypair.fromSecretKey(Uint8Array.from(secret));
  const user = wallet.publicKey;

  // Sanity: program exists on this cluster
  const progInfo = await connection.getAccountInfo(PROGRAM_ID);
  if (!progInfo) {
    throw new Error(
      `Program ${PROGRAM_ID.toBase58()} not found on ${CLUSTER_URL}`,
    );
  }

  const [vault] = pda([VAULT_SEED, user.toBuffer()], PROGRAM_ID);
  const [vaultState] = pda([VAULT_STATE_SEED, user.toBuffer()], PROGRAM_ID);

  const bal = async (pk: PublicKey) => BigInt(await connection.getBalance(pk));

  console.log("user:", user.toBase58());
  console.log("vault:", vault.toBase58());
  console.log("vault_state:", vaultState.toBase58());

  // Airdrop for whichever cluster you're using
  const bal0 = await connection.getBalance(user);
  if (bal0 < 5_000_000_000) {
    console.log(`Airdropping SOL to wallet on ${CLUSTER_URL}...`);
    const sig = await connection.requestAirdrop(user, 5_000_000_000);
    await connection.confirmTransaction(sig, "confirmed");
  }

  console.log("vault balance before:", (await bal(vault)).toString());

  const maxWithdraw = 5_000_000_000n; // 5 SOL
  const sendIx = async (ix: TransactionInstruction) => {
    const tx = new Transaction().add(ix);
    tx.feePayer = user;
    tx.recentBlockhash = (
      await connection.getLatestBlockhash("confirmed")
    ).blockhash;
    return sendAndConfirmTransaction(connection, tx, [wallet], {
      commitment: "confirmed",
    });
  };

  // ---------------- initialize ----------------
  // Anchor account order for Initialize<'info'>: user, vault, vault_state, system_program
  const initData = Buffer.concat([
    anchorDiscriminator("initialize"),
    u64le(maxWithdraw),
  ]);
  const initIx = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: initData,
  });

  console.log("Initializing vault...");
  const vaultStateInfo = await connection.getAccountInfo(vaultState);
  if (!vaultStateInfo) {
    console.log("Initializing vault...");
    await sendIx(initIx);
  } else {
    console.log("vault_state already exists; skipping initialize.");
  }

  // ---------------- deposit(0) must fail ----------------
  const deposit0Data = Buffer.concat([
    anchorDiscriminator("deposit"),
    u64le(0n),
  ]);
  const deposit0Ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: deposit0Data,
  });

  const vaultBeforeZeroDeposit = await bal(vault);
  await expectTxFail(() => sendIx(deposit0Ix), "deposit(0)");
  const vaultAfterZeroDeposit = await bal(vault);
  if (vaultAfterZeroDeposit !== vaultBeforeZeroDeposit) {
    throw new Error("deposit(0) changed vault balance unexpectedly");
  }
  console.log("deposit(0) failure confirmed, vault unchanged.");

  // ---------------- deposit(x) success ----------------
  const depositAmount = 2_000_000_000n; // 2 SOL
  const depositData = Buffer.concat([
    anchorDiscriminator("deposit"),
    u64le(depositAmount),
  ]);
  const depositIx = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: depositData,
  });

  const vaultBeforeDeposit = await bal(vault);
  await sendIx(depositIx);
  const vaultAfterDeposit = await bal(vault);

  if (vaultAfterDeposit - vaultBeforeDeposit !== depositAmount) {
    throw new Error(
      `deposit delta mismatch: expected ${depositAmount} got ${
        vaultAfterDeposit - vaultBeforeDeposit
      }`,
    );
  }
  console.log(
    "deposit(x) success, vault balance delta matches deposit amount.",
  );

  // ---------------- withdraw(0) must fail ----------------
  const withdraw0Data = Buffer.concat([
    anchorDiscriminator("withdraw"),
    u64le(0n),
  ]);
  const withdraw0Ix = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: withdraw0Data,
  });

  const vaultBeforeZeroWithdraw = await bal(vault);
  await expectTxFail(() => sendIx(withdraw0Ix), "withdraw(0)");
  const vaultAfterZeroWithdraw = await bal(vault);
  if (vaultAfterZeroWithdraw !== vaultBeforeZeroWithdraw) {
    throw new Error("withdraw(0) changed vault balance unexpectedly");
  }
  console.log("withdraw(0) failure confirmed, vault unchanged.");

  // ---------------- withdraw(y) success ----------------
  const withdrawAmount = 1_000_000_000n; // 1 SOL
  const withdrawData = Buffer.concat([
    anchorDiscriminator("withdraw"),
    u64le(withdrawAmount),
  ]);
  const withdrawIx = new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: withdrawData,
  });

  const vaultBeforeWithdraw = await bal(vault);
  const userBeforeWithdraw = await bal(user);

  await sendIx(withdrawIx);

  const vaultAfterWithdraw = await bal(vault);
  const userAfterWithdraw = await bal(user);

  if (vaultBeforeWithdraw - vaultAfterWithdraw !== withdrawAmount) {
    throw new Error(
      `withdraw delta mismatch: expected ${withdrawAmount} got ${
        vaultBeforeWithdraw - vaultAfterWithdraw
      }`,
    );
  }
  if (userAfterWithdraw <= userBeforeWithdraw) {
    throw new Error("user did not increase after withdraw (unexpected)");
  }
  if (userAfterWithdraw - userBeforeWithdraw > withdrawAmount) {
    throw new Error(
      "user net gain exceeded withdrawn amount (unexpected fees behavior)",
    );
  }

  console.log("withdraw(y) success, vault/user assertions passed.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
