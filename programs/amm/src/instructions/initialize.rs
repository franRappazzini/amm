use anchor_lang::prelude::*;

use crate::{constants::GLOBAL_CONFIG_SEED, errors::AmmError, states::GlobalConfig};

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
    pub fn initialize(&mut self, protocol_fee_bps: u16, fee_bps: u16, bump: u8) -> Result<()> {
        let total_fee_bps = protocol_fee_bps
            .checked_add(fee_bps)
            .ok_or(AmmError::MathOverflow)?;

        require!(total_fee_bps <= 10_000, AmmError::FeeExceedsMaximum);

        self.global_config.set_inner(GlobalConfig {
            authority: self.authority.key(),
            bump,
            protocol_fee_bps,
            fee_bps,
            pool_count: 0,
        });

        Ok(())
    }
}
