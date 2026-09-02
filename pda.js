const { PublicKey } = require("@coral-xyz/anchor").web3;

const programId = new PublicKey(process.argv[2]);
const seed = process.argv[3] ?? "counter";

const [pda, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(seed)],
  programId
);

console.log("program", programId.toBase58());
console.log("seed   ", seed);
console.log("pda    ", pda.toBase58());
console.log("bump   ", bump);
