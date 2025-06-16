use anchor_lang::prelude::*;

declare_id!("AqejyYXgr792tntPRJajpBLMvhYCTFXQWRoBzUKp6TkN");

#[program]
pub mod treasury_your_wallet {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
