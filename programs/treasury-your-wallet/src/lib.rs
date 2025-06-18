use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN"); // 배포 후 자동 교체 필요


#[program]
pub mod treasury_your_wallet {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.user.key();
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct CreateVault<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32,
        seeds = [b"vault", user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, VaultState>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub owner: Pubkey,
}