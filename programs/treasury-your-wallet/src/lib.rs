use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    /// 초기화 - PDA 계정 생성 및 관리자(admin) 저장
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.vault_account.admin = ctx.accounts.user.key();
        Ok(())
    }

    /// 유저 → Vault(PDA)로 SOL 입금
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

    /// Vault(PDA) → 유저로 SOL 출금 (admin만 가능)
    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSolFromVault>, amount: u64) -> Result<()> {
        // 관리자만 출금 가능
        require_keys_eq!(
            ctx.accounts.user.key(),
            ctx.accounts.vault_account.admin,
            CustomError::Unauthorized
        );

        let vault_balance = **ctx.accounts.vault_account.to_account_info().lamports.borrow();
        require!(vault_balance >= amount, CustomError::InsufficientFunds);

        **ctx.accounts.vault_account.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

// -----------------------
// Accounts
// -----------------------

#[account]
pub struct VaultAccount {
    pub admin: Pubkey, // 관리자 키
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
}
