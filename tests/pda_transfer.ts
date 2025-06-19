import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TreasuryYourWallet } from "../target/types/treasury_your_wallet";

describe("🔐 Day 5: PDA Vault - SOL 입출금", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TreasuryYourWallet as Program<TreasuryYourWallet>;
  const user = provider.wallet;

  let vaultPda: PublicKey;
  let bump: number;

  const seed = Buffer.from("vault");

  it("📌 1. Vault(PDA) 계정 생성", async () => {
    [vaultPda, bump] = await PublicKey.findProgramAddressSync(
      [seed, user.publicKey.toBuffer()],
      program.programId
    );
    console.log("🧱 PDA 주소:", vaultPda.toBase58());

    // 스마트 컨트랙트 호출
    await program.methods
      .initializeVault()
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Vault PDA 생성 완료");
  });

  it("💰 2. 유저 → Vault(PDA)로 0.1 SOL 입금", async () => {
    const depositAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL);
    await program.methods
      .depositSolToVault(depositAmount)
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
      })
      .rpc();

    const balance = await provider.connection.getBalance(vaultPda);
    console.log("💰 PDA 잔고 (입금 후):", balance / LAMPORTS_PER_SOL, "SOL");
  });

  it("📤 3. PDA → 유저로 0.05 SOL 출금", async () => {
    const withdrawAmount = new anchor.BN(0.05 * LAMPORTS_PER_SOL);
    await program.methods
      .withdrawSolFromVault(withdrawAmount)
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
      })
      .rpc();

    const balance = await provider.connection.getBalance(vaultPda);
    console.log("📉 PDA 잔고 (출금 후):", balance / LAMPORTS_PER_SOL, "SOL");
  });

  it("🔎 4. PDA 잔액 로그 출력", async () => {
    await program.methods
      .logVaultBalance()
      .accounts({
        vaultAccount: vaultPda,
        user: user.publicKey,
      })
      .rpc();

    console.log("✅ 잔액 확인 로그 호출 완료");
  });
});
