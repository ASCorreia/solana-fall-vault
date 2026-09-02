import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HelloSolana } from "../target/types/hello_solana";
import { assert } from "chai";

describe("hello_solana", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.helloSolana as Program<HelloSolana>;

  it("Is initialized and respects max_withdraw!", async () => {
    const vault = anchor.web3.Keypair.generate();
    const maxWithdraw = new anchor.BN(100);
    
    // 1. Initialize with max_withdraw = 100
    const tx = await program.methods
      .initialize(maxWithdraw)
      .accounts({
        vault: vault.publicKey,
        user: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([vault])
      .rpc();
      
    console.log("Initialization signature:", tx);

    // 2. Test successful withdrawal within the limit (e.g., 50)
    // (Assuming your withdraw method is set up, adjust accounts/args as per your withdraw struct)
    try {
      const withdrawTx = await program.methods
        .withdraw(new anchor.BN(50))
        .accounts({
          vault: vault.publicKey,
          user: program.provider.wallet.publicKey,
        })
        .rpc();
      console.log("Withdrawal success signature:", withdrawTx);
    } catch (e) {
      console.log("Withdrawal failed unexpectedly:", e);
      assert.fail("Should have allowed withdrawal within limit");
    }

    // 3. Test failed withdrawal over the limit (e.g., trying to withdraw 60 more when limit/remaining allows or checking error)
    // Note: Adjust the amount depending on whether your limit is absolute or cumulative per tx.
    try {
      await program.methods
        .withdraw(new anchor.BN(200)) // exceeds max_withdraw (100)
        .accounts({
          vault: vault.publicKey,
          user: program.provider.wallet.publicKey,
        })
        .rpc();
      assert.fail("The withdrawal should have failed due to max_withdraw limit");
    } catch (error) {
      assert.include(error.message, "MaxWithdrawExceeded");
    }
  });
});
