import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TreasuryYourWallet } from "../target/types/treasury_your_wallet";

describe("treasury_your_wallet", () => {
  // Provider 및 프로그램 로딩
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TreasuryYourWallet as Program<TreasuryYourWallet>;

  // 관리자 키쌍
  const admin = Keypair.generate();
  let vaultPda: PublicKey;
  let vaultBump: number;

  before(async () => {
    // admin에게 SOL 에어드롭
    const sig = await provider.connection.requestAirdrop(admin.publicKey, LAMPORTS_PER_SOL * 2);
    await provider.connection.confirmTransaction(sig);

    // Vault PDA 계산
    [vaultPda, vaultBump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), admin.publicKey.toBuffer()],
      program.programId
    );
  });

  it("🛠️ 초기화 - Vault 생성", async () => {
    await program.methods
      .initializeVault()
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const vault = await program.account.vaultAccount.fetch(vaultPda);
    console.log("Vault Admin:", vault.admin.toBase58());
    if (!vault.admin.equals(admin.publicKey)) throw new Error("Admin 설정 실패");
  });

  it("💰 입금 - 관리자 → Vault", async () => {
    const amount = 0.5 * LAMPORTS_PER_SOL;

    await program.methods
      .depositSolToVault(new anchor.BN(amount))
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    const vaultBalance = await provider.connection.getBalance(vaultPda);
    console.log("Vault 잔액:", vaultBalance / LAMPORTS_PER_SOL, "SOL");
  });

  it("💸 출금 - 관리자만 가능", async () => {
    const amount = 0.2 * LAMPORTS_PER_SOL;

    const beforeBalance = await provider.connection.getBalance(admin.publicKey);

    await program.methods
      .withdrawSolFromVault(new anchor.BN(amount))
      .accounts({
        user: admin.publicKey,
        vaultAccount: vaultPda,
      })
      .signers([admin])
      .rpc();

    const afterBalance = await provider.connection.getBalance(admin.publicKey);
    console.log("출금 전/후 관리자 지갑:", beforeBalance, "→", afterBalance);
  });

  it("⛔️ 실패 - 관리자 아닌 유저가 출금 시도", async () => {
    const attacker = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(attacker.publicKey, LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);

    try {
      await program.methods
        .withdrawSolFromVault(new anchor.BN(0.1 * LAMPORTS_PER_SOL))
        .accounts({
          user: attacker.publicKey,
          vaultAccount: vaultPda,
        })
        .signers([attacker])
        .rpc();

      throw new Error("❌ 비관리자가 출금에 성공했습니다. 이는 버그입니다!");
    } catch (err) {
      console.log("✅ 예상된 에러 발생:", err.error.errorMessage);
    }
  });
});
