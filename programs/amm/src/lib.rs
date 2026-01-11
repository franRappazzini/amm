mod constants;
mod instructions;
mod states;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("BiHu64Uy1VhgeehWxymVCvbLM4bqMMRYkkXn3ExUBfu7");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
        ctx.accounts.initialize(fee_bps, ctx.bumps.global_config)
    }
}
