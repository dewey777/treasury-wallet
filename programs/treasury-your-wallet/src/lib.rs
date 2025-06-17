use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWxTWqKbq3Yjz1G8c9W5bQxtATNQ"); // 배포 후 자동 교체 필요

#[program]
pub mod treasury_your_wallet {
    use super::*;

    // lamports 단위로 전송 (1 SOL = 1_000_000_000 lamports)
    pub fn transfer_sol(ctx: Context<TransferSol>, amount: u64) -> Result<()> {
        let from = &ctx.accounts.from;
        let to = &ctx.accounts.to;

        **from.try_borrow_mut_lamports()? -= amount;
        **to.try_borrow_mut_lamports()? += amount;

        msg!("Transferred {} lamports!", amount);
        Ok(())
    }

    pub fn log_balance(ctx: Context<LogBalance>) -> Result<()> {
        let user = &ctx.accounts.user;
        let balance = user.lamports();
        msg!("Current balance: {} lamports", balance);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferSol<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    /// CHECK: Safe because we're transferring lamports
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LogBalance<'info> {
    /// CHECK: We're just logging balance, no mutation
    pub user: AccountInfo<'info>,
}
