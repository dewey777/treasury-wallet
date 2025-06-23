import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SystemProgram, Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TreasuryYourWallet } from "../target/types/treasury_your_wallet";

describe("Day 7: 출금 쿨타임 테스트", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TreasuryYourWallet as Program<TreasuryYourWallet>;

  const admin = Keypair.generate();
  let vaultPda: PublicKey;

  before(async () => {
    // admin에 2 SOL 지급
    const tx = await provider.connection.requestAirdrop(admin.publicKey, LAMPORTS_PER_SOL * 2);
    await provider.connection.confirmTransaction(tx);

    // PDA 계산
    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), admin.publicKey.toBuffer()],
      program.programId
    );
  });

  it("✅ Vault 초기화", async () => {
    await program.methods
      .initializeVault()
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();
  });

  it("✅ SOL 입금", async () => {
    const amount = new anchor.BN(1 * LAMPORTS_PER_SOL);
    await program.methods
      .depositSolToVault(amount)
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const balance = await provider.connection.getBalance(vaultPda);
    console.log("📦 Vault 잔액:", balance / LAMPORTS_PER_SOL, "SOL");
  });

  it("✅ 첫 번째 출금 (성공)", async () => {
    const amount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);
    await program.methods
      .withdrawSolFromVault(amount)
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
      })
      .signers([admin])
      .rpc();
    console.log("💸 첫 번째 출금 성공");
  });

  it("❌ 두 번째 출금 시도 (쿨타임 위반 → 실패)", async () => {
    const amount = new anchor.BN(0.3 * LAMPORTS_PER_SOL);
    try {
      await program.methods
        .withdrawSolFromVault(amount)
        .accounts({
          user: admin.publicKey,
          vaultAccount: vaultPda,
        })
        .signers([admin])
        .rpc();
      throw new Error("❌ 두 번째 출금이 성공했는데, 실패했어야 합니다!");
    } catch (err) {
      console.log("⏱️ 예상대로 실패함 (쿨타임 위반):", err.error.errorMessage);
    }
  });

  // (선택) 시간 조작은 anchor test 환경에서 따로 clock mocking이 필요함
  // Advanced: use anchor.setClock() via local validator or custom test
});
