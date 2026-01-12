use anchor_lang::prelude::*;

use crate::{constants::GLOBAL_CONFIG_SEED, states::GlobalConfig};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = GlobalConfig::SIZE,
        seeds = [GLOBAL_CONFIG_SEED],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, fee_bps: u16, bump: u8) -> Result<()> {
        self.global_config.set_inner(GlobalConfig {
            authority: self.authority.key(),
            bump,
            fee_bps,
            pool_count: 0,
        });

        Ok(())
    }
}
