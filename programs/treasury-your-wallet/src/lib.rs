use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    /// 초기화 - Vault 생성 + admin 저장
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.vault_account.admin = ctx.accounts.user.key();
        Ok(())
    }

    /// 유저 → Vault 입금
    pub fn deposit_sol_to_vault(ctx: Context<DepositSolToVault>, amount: u64) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault_account.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.vault_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        Ok(())
    }

    /// Vault → 유저 출금 (admin만 가능 + 1 SOL 이하만 가능)
    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSolFromVault>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault_account;

        // ✅ 조건 1: 관리자만 출금 가능
        require_keys_eq!(
            ctx.accounts.user.key(),
            vault.admin,
            CustomError::Unauthorized
        );

        // ✅ 조건 2: 출금 상한선 1 SOL
        const MAX_WITHDRAW_AMOUNT: u64 = 1_000_000_000; // 1 SOL in lamports
        require!(amount <= MAX_WITHDRAW_AMOUNT, CustomError::ExceedsMaxWithdrawal);

        // ✅ 조건 3: Vault에 충분한 잔고
        let vault_balance = **vault.to_account_info().lamports.borrow();
        require!(vault_balance >= amount, CustomError::InsufficientFunds);

        // ✅ 출금 수행
        **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

// -----------------------
// Accounts
// -----------------------

#[account]
pub struct VaultAccount {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        seeds = [b"vault", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32, // 8 byte discriminator + 32 byte pubkey
    )]
    pub vault_account: Account<'info, VaultAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSolToVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault_account: Account<'info, VaultAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [b"vault", vault_account.admin.as_ref()], bump)]
    pub vault_account: Account<'info, VaultAccount>,
}

#[error_code]
pub enum CustomError {
    #[msg("잔액이 부족합니다.")]
    InsufficientFunds,

    #[msg("관리자만 출금할 수 있습니다.")]
    Unauthorized,

    #[msg("출금 한도를 초과했습니다. (최대 1 SOL)")]
    ExceedsMaxWithdrawal,
}
