use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    /// Vault 생성 및 관리자, 출금 한도, 쿨타임 저장
    pub fn initialize_vault(ctx: Context<InitializeVault>, max_withdraw: u64, cooldown: i64) -> Result<()> {
        let vault = &mut ctx.accounts.vault_account;
        vault.admin = ctx.accounts.user.key();
        vault.last_withdraw_ts = 0; // 최초는 0으로 설정
        vault.max_withdraw = max_withdraw;
        vault.cooldown = cooldown;
        Ok(())
    }

    /// SOL 입금 (user → vault)
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

    /// SOL 출금 (vault → user)  
    /// 관리자만 가능, 출금 상한선, 쿨타임 적용 (유저별)
    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSolFromVault>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault_account;
        let clock = Clock::get()?;

        // ✅ 관리자만 출금 가능
        require_keys_eq!(ctx.accounts.user.key(), vault.admin, CustomError::Unauthorized);

        // ✅ 출금 상한선: 유저별
        require!(amount <= vault.max_withdraw, CustomError::ExceedsMaxWithdrawal);

        // ✅ 쿨타임: 유저별
        let now = clock.unix_timestamp;
        let elapsed = now - vault.last_withdraw_ts;
        require!(elapsed >= vault.cooldown, CustomError::TooEarlyToWithdraw);

        // ✅ 잔액 확인
        let vault_balance = **vault.to_account_info().lamports.borrow();
        require!(vault_balance >= amount, CustomError::InsufficientFunds);

        // ✅ 출금 수행
        **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

        // ✅ 마지막 출금 시간 기록
        vault.last_withdraw_ts = now;

        Ok(())
    }
}

// ------------------------------
// Account 구조체들
// ------------------------------

#[account]
pub struct VaultAccount {
    pub admin: Pubkey,            // 관리자 지갑 주소
    pub last_withdraw_ts: i64,    // 마지막 출금 유닉스 시간
    pub max_withdraw: u64,        // 출금 한도 (lamports)
    pub cooldown: i64,            // 쿨타임 (초)
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
        space = 8 + 32 + 8 + 8 + 8, // Anchor 계정 디스크 크기
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

// ------------------------------
// 커스텀 에러
// ------------------------------

#[error_code]
pub enum CustomError {
    #[msg("잔액이 부족합니다.")]
    InsufficientFunds,

    #[msg("관리자만 출금할 수 있습니다.")]
    Unauthorized,

    #[msg("출금 한도를 초과했습니다. (최대 1 SOL)")]
    ExceedsMaxWithdrawal,

    #[msg("출금 간 쿨타임이 아직 지나지 않았습니다. (1시간 제한)")]
    TooEarlyToWithdraw,
}
