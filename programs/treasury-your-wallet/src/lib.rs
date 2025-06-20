use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    /// 초기화 - PDA 계정을 생성하고, 해당 계정의 owner를 이 프로그램으로 설정
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
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

    /// Vault(PDA) → 유저로 SOL 출금
    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSolFromVault>, amount: u64) -> Result<()> {
        let vault_balance = **ctx.accounts.vault_account.lamports.borrow();

        require!(vault_balance >= amount, CustomError::InsufficientFunds);

        **ctx.accounts.vault_account.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

// -----------------------
// Accounts
// -----------------------

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: PDA 계정이며, rent-exempt 이상 lamports 보유 및 owner가 프로그램으로 설정됨
    #[account(
        init,
        seeds = [b"vault", user.key().as_ref()],
        bump,
        payer = user,
        space = 8, // 최소 크기
        owner = crate::ID
    )]
    pub vault_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSolToVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: PDA Vault 계정
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: PDA Vault 계정
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault_account: AccountInfo<'info>,
}

#[error_code]
pub enum CustomError {
    #[msg("잔액이 부족합니다.")]
    InsufficientFunds,
}
