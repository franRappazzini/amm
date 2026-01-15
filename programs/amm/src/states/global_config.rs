use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR_SIZE;

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub bump: u8,
    pub protocol_fee_bps: u16,
    pub fee_bps: u16,
    pub pool_count: u64,
}

impl GlobalConfig {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + GlobalConfig::INIT_SPACE;
}
