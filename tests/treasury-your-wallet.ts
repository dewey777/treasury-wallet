import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TreasuryYourWallet } from "../target/types/treasury_your_wallet";
import { SystemProgram } from "@solana/web3.js";

describe("vault PDA test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TreasuryYourWallet as Program<TreasuryYourWallet>;

  it("Initializes a vault account", async () => {
    const [vaultPDA, _bump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods.createVault()
      .accounts({
        vault: vaultPDA,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vaultAccount = await program.account.vaultState.fetch(vaultPDA);
    console.log("Vault Owner:", vaultAccount.owner.toBase58());
  });
});
