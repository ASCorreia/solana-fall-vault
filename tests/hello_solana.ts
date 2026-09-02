import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HelloSolana } from "../target/types/hello_solana";

describe("hello_solana", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.helloSolana as Program<HelloSolana>;

  it("Is initialized!", async () => {
    // Add your test here.
    const vault = anchor.web3.Keypair.generate();
    const maxWithdraw = new anchor.BN(100);
    
    const tx = await program.methods
      .initialize(maxWithdraw)
      .accounts({
        vault: vault.publicKey,
        user: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vault])
      .rpc();
      
    console.log("Your transaction signature", tx);
  });
});
