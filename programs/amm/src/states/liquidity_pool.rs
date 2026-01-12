use anchor_lang::prelude::*;

use crate::constants::DISCRIMINATOR_SIZE;

#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub creator: Pubkey,
    pub bump: u8,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub lp_mint: Pubkey,
    pub fee_bps: u16,
    pub lp_supply: u64,
    pub amount_mint_a: u64,
    pub amount_mint_b: u64,
}

impl LiquidityPool {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + LiquidityPool::INIT_SPACE;
}
