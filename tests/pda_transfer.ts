import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TreasuryYourWallet } from "../target/types/treasury_your_wallet";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";

describe("🔐 Day 5: PDA Vault - SOL 입출금 (Devnet)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TreasuryYourWallet as Program<TreasuryYourWallet>;
  const user = provider.wallet;

  let vaultPda: PublicKey;
  let bump: number;
  const seed = Buffer.from("vault");

  it("✅ Vault PDA 생성 및 입출금 테스트", async () => {
    [vaultPda, bump] = PublicKey.findProgramAddressSync(
      [seed, user.publicKey.toBuffer()],
      program.programId
    );
    console.log("🧱 PDA 주소:", vaultPda.toBase58());

    // Vault 생성
    try {
      const tx = await program.methods
        .initializeVault()
        .accounts({
          vaultAccount: vaultPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("✅ Vault PDA 생성 완료");
      console.log("✔ Tx:", tx);
    } catch (e) {
      console.log("Vault PDA 생성 에러 (이미 있을 수 있음):", e);
    }

    const balanceBefore = await provider.connection.getBalance(vaultPda);
    console.log("💰 PDA 초기 잔고:", balanceBefore / LAMPORTS_PER_SOL, "SOL");

    // 입금: 0.1 SOL
    const depositAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL);
    const depositTx = await program.methods
      .depositSolToVault(depositAmount)
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("💰 입금 완료 Tx:", depositTx);
    const balanceAfterDeposit = await provider.connection.getBalance(vaultPda);
    console.log("💰 PDA 잔고 (입금 후):", balanceAfterDeposit / LAMPORTS_PER_SOL, "SOL");

    // 출금: 0.05 SOL
    const withdrawAmount = new anchor.BN(0.05 * LAMPORTS_PER_SOL);
    const withdrawTx = await program.methods
      .withdrawSolFromVault(withdrawAmount)
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
      })
      .rpc();

    console.log("📤 출금 완료 Tx:", withdrawTx);
    const balanceAfterWithdraw = await provider.connection.getBalance(vaultPda);
    console.log("📉 PDA 잔고 (출금 후):", balanceAfterWithdraw / LAMPORTS_PER_SOL, "SOL");
  });
});
