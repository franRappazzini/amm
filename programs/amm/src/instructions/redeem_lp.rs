use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{constants::LIQUIDITY_POOL_SEED, errors::AmmError, states::LiquidityPool, utils};

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct RedeemLp<'info> {
    #[account(mut)]
    pub redeemer: Signer<'info>,

    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED, &pool_id.to_le_bytes()],
        bump = liquidity_pool.bump,
        has_one = mint_a,
        has_one = mint_b,
        has_one = lp_mint,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub mint_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub mint_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = redeemer,
        associated_token::token_program = token_program,
    )]
    pub redeemer_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = redeemer,
        associated_token::token_program = token_program,
    )]
    pub redeemer_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = redeemer,
        associated_token::token_program = token_program,
    )]
    pub redeemer_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> RedeemLp<'info> {
    pub fn redeem_lp(&mut self, pool_id: u64, lp_amount: u64) -> Result<()> {
        require!(lp_amount > 0, AmmError::InsufficientInputAmount);

        // burn lp tokens
        utils::token::burn_spl(
            &self.redeemer,
            &self.redeemer_lp_ata,
            &self.lp_mint,
            lp_amount,
            &self.token_program,
        )?;

        // close ata if balance == 0 ?

        // calculate amounts and transfer mint A and B to redeemer
        let claimable_amount_a = utils::math::calculate_claimable_amount(
            lp_amount,
            self.liquidity_pool.lp_supply,
            self.liquidity_pool.amount_mint_a,
        )?;
        let claimable_amount_b = utils::math::calculate_claimable_amount(
            lp_amount,
            self.liquidity_pool.lp_supply,
            self.liquidity_pool.amount_mint_b,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED,
            &pool_id.to_le_bytes(),
            &[self.liquidity_pool.bump],
        ]];

        if claimable_amount_a > 0 {
            utils::token::transfer_spl(
                &self.liquidity_pool.to_account_info(),
                &self.mint_a_vault,
                &self.redeemer_a_ata,
                &self.mint_a,
                claimable_amount_a,
                &self.token_program,
                Some(signer_seeds),
            )?;

            self.liquidity_pool.amount_mint_a = self
                .liquidity_pool
                .amount_mint_a
                .checked_sub(claimable_amount_a)
                .ok_or(AmmError::MathUnderflow)?;
        }

        if claimable_amount_b > 0 {
            utils::token::transfer_spl(
                &self.liquidity_pool.to_account_info(),
                &self.mint_b_vault,
                &self.redeemer_b_ata,
                &self.mint_b,
                claimable_amount_b,
                &self.token_program,
                Some(signer_seeds),
            )?;

            self.liquidity_pool.amount_mint_b = self
                .liquidity_pool
                .amount_mint_b
                .checked_sub(claimable_amount_b)
                .ok_or(AmmError::MathUnderflow)?;
        }

        // update liquidity pool state
        self.liquidity_pool.lp_supply = self
            .liquidity_pool
            .lp_supply
            .checked_sub(lp_amount)
            .ok_or(AmmError::MathUnderflow)?;

        Ok(())
    }
}
