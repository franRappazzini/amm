use anchor_lang::prelude::*;

use crate::{constants::DISCRIMINATOR_SIZE, errors::AmmError};

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

    pub fn add_liquidity_a(&mut self, amount: u64) -> Result<()> {
        self.amount_mint_a = self
            .amount_mint_a
            .checked_add(amount)
            .ok_or(AmmError::MathOverflow)?;
        Ok(())
    }

    pub fn add_liquidity_b(&mut self, amount: u64) -> Result<()> {
        self.amount_mint_b = self
            .amount_mint_b
            .checked_add(amount)
            .ok_or(AmmError::MathOverflow)?;
        Ok(())
    }

    pub fn add_liquidity_lp(&mut self, amount: u64) -> Result<()> {
        self.lp_supply = self
            .lp_supply
            .checked_add(amount)
            .ok_or(AmmError::MathOverflow)?;
        Ok(())
    }

    pub fn remove_liquidity_a(&mut self, amount: u64) -> Result<()> {
        self.amount_mint_a = self
            .amount_mint_a
            .checked_sub(amount)
            .ok_or(AmmError::MathUnderflow)?;
        Ok(())
    }

    pub fn remove_liquidity_b(&mut self, amount: u64) -> Result<()> {
        self.amount_mint_b = self
            .amount_mint_b
            .checked_sub(amount)
            .ok_or(AmmError::MathUnderflow)?;
        Ok(())
    }

    pub fn remove_liquidity_lp(&mut self, amount: u64) -> Result<()> {
        self.lp_supply = self
            .lp_supply
            .checked_sub(amount)
            .ok_or(AmmError::MathUnderflow)?;
        Ok(())
    }
}
