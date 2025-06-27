use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    /// Vault(금고) 생성 함수
    /// - 유저가 자신의 금고(PDA)를 만들 때 호출합니다.
    /// - 출금 한도(max_withdraw)와 쿨타임(cooldown)을 유저가 직접 지정할 수 있습니다.
    /// - 금고의 관리자, 마지막 출금 시각, 한도, 쿨타임을 저장합니다.
    pub fn initialize_vault(ctx: Context<InitializeVault>, max_withdraw: u64, cooldown: i64) -> Result<()> {
        let vault = &mut ctx.accounts.vault_account;
        vault.admin = ctx.accounts.user.key(); // 금고의 관리자(유저) 주소 저장
        vault.last_withdraw_ts = 0; // 아직 출금한 적 없으므로 0으로 초기화
        vault.max_withdraw = max_withdraw; // 유저가 지정한 출금 한도
        vault.cooldown = cooldown; // 유저가 지정한 쿨타임(초)
        Ok(())
    }

    /// SOL 입금 함수 (user → vault)
    /// - 유저가 자신의 금고(PDA)로 SOL을 입금할 때 호출합니다.
    /// - 실제로는 Solana 시스템 트랜스퍼 명령을 사용해 유저 지갑에서 금고(PDA)로 SOL을 전송합니다.
    pub fn deposit_sol_to_vault(ctx: Context<DepositSolToVault>, amount: u64) -> Result<()> {
        // Solana 시스템 트랜스퍼 명령 생성 (user → vault)
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.vault_account.key(),
            amount,
        );
        // 트랜스퍼 명령을 실제로 실행
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

    /// SOL 출금 함수 (vault → user)
    /// - 유저가 자신의 금고(PDA)에서 SOL을 출금할 때 호출합니다.
    /// - 출금 한도, 쿨타임, 관리자 권한 등 다양한 조건을 체크합니다.
    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSolFromVault>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault_account;
        let clock = Clock::get()?; // 현재 블록체인 시각(유닉스 타임스탬프)

        // ✅ 관리자(본인)만 출금 가능
        require_keys_eq!(ctx.accounts.user.key(), vault.admin, CustomError::Unauthorized);

        // ✅ 출금 한도 체크 (유저별로 다름)
        require!(amount <= vault.max_withdraw, CustomError::ExceedsMaxWithdrawal);

        // ✅ 쿨타임 체크 (유저별로 다름)
        let now = clock.unix_timestamp;
        let elapsed = now - vault.last_withdraw_ts;
        require!(elapsed >= vault.cooldown, CustomError::TooEarlyToWithdraw);

        // ✅ 금고 잔액이 충분한지 확인
        let vault_balance = **vault.to_account_info().lamports.borrow();
        require!(vault_balance >= amount, CustomError::InsufficientFunds);

        // ✅ 실제 출금 (금고에서 유저로 SOL 이동)
        **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

        // ✅ 마지막 출금 시각 갱신
        vault.last_withdraw_ts = now;

        Ok(())
    }
}

// ------------------------------
// Account 구조체들 (블록체인에 저장되는 데이터 구조)
// ------------------------------

// VaultAccount: 각 유저별 금고(PDA)에 저장되는 데이터 구조입니다.
#[account]
pub struct VaultAccount {
    pub admin: Pubkey,            // 금고의 관리자(유저) 지갑 주소
    pub last_withdraw_ts: i64,    // 마지막 출금 시각(유닉스 타임스탬프)
    pub max_withdraw: u64,        // 출금 한도 (lamports)
    pub cooldown: i64,            // 쿨타임 (초)
}

// InitializeVault: 금고 생성 시 필요한 계정 정보와 제약조건을 정의합니다.
#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // 금고를 만드는 유저(관리자)

    #[account(
        init, // 새로 계정 생성
        seeds = [b"vault", user.key().as_ref()], // PDA 주소 생성 규칙
        bump, // PDA bump 값 자동 계산
        payer = user, // 계정 생성 수수료를 유저가 부담
        space = 8 + 32 + 8 + 8 + 8, // 계정 크기(Anchor용)
    )]
    pub vault_account: Account<'info, VaultAccount>, // 유저별 금고(PDA)

    pub system_program: Program<'info, System>, // 시스템 프로그램
}

// DepositSolToVault: 입금 시 필요한 계정 정보와 제약조건을 정의합니다.
#[derive(Accounts)]
pub struct DepositSolToVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // 입금하는 유저

    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault_account: Account<'info, VaultAccount>, // 유저별 금고(PDA)

    pub system_program: Program<'info, System>, // 시스템 프로그램
}

// WithdrawSolFromVault: 출금 시 필요한 계정 정보와 제약조건을 정의합니다.
#[derive(Accounts)]
pub struct WithdrawSolFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // 출금하는 유저

    #[account(mut, seeds = [b"vault", vault_account.admin.as_ref()], bump)]
    pub vault_account: Account<'info, VaultAccount>, // 유저별 금고(PDA)
}

// ------------------------------
// 커스텀 에러 정의 (출금 실패 등 상황별 에러 메시지)
// ------------------------------

#[error_code]
pub enum CustomError {
    #[msg("잔액이 부족합니다.")]
    InsufficientFunds, // 금고에 잔액이 부족할 때

    #[msg("관리자만 출금할 수 있습니다.")]
    Unauthorized, // 금고의 주인이 아닐 때

    #[msg("출금 한도를 초과했습니다. (최대 1 SOL)")]
    ExceedsMaxWithdrawal, // 출금 한도를 초과할 때

    #[msg("출금 간 쿨타임이 아직 지나지 않았습니다. (1시간 제한)")]
    TooEarlyToWithdraw, // 쿨타임이 지나지 않았을 때
}

// ------------------------------
// 전체 동작 흐름 요약
// ------------------------------
// 1. 유저가 initialize_vault로 자신의 금고(PDA)를 생성합니다.
//    - 이때 출금 한도와 쿨타임을 직접 지정할 수 있습니다.
// 2. deposit_sol_to_vault로 금고(PDA)에 SOL을 입금합니다.
// 3. withdraw_sol_from_vault로 금고에서 SOL을 출금할 수 있습니다.
//    - 출금 한도, 쿨타임, 관리자 권한 등 다양한 조건이 적용됩니다.
// 4. 모든 데이터는 VaultAccount(PDA)에 저장되며, 출금/입금은 반드시 이 프로그램의 규칙을 따라야만 가능합니다.
