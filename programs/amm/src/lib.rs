use anchor_lang::prelude::*;

declare_id!("BiHu64Uy1VhgeehWxymVCvbLM4bqMMRYkkXn3ExUBfu7");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
