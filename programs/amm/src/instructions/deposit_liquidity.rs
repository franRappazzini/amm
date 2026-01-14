use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::LIQUIDITY_POOL_SEED, errors::AmmError, states::LiquidityPool, utils};

#[derive(Accounts)]
#[instruction(pool_id:u64)]
pub struct DepositLiquidity<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

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
        associated_token::authority = provider,
        associated_token::token_program = token_program,
    )]
    pub provider_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = provider,
        associated_token::token_program = token_program,
    )]
    pub provider_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = provider,
        associated_token::mint = lp_mint,
        associated_token::authority = provider,
        associated_token::token_program = token_program,
    )]
    pub provider_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositLiquidity<'info> {
    pub fn deposit_liquidity(&mut self, pool_id: u64, amount_a: u64, amount_b: u64) -> Result<()> {
        require!(
            amount_a > 0 && amount_b > 0,
            AmmError::InsufficientInputAmount
        );

        // calculate possible excess
        let (new_amount_a, new_amount_b) = utils::math::calculate_deposit_excess(
            amount_a,
            amount_b,
            self.liquidity_pool.amount_mint_a,
            self.liquidity_pool.amount_mint_b,
        )?;

        msg!(
            "Current pool reserves: {} of A and {} of B",
            self.liquidity_pool.amount_mint_a,
            self.liquidity_pool.amount_mint_b
        );

        msg!(
            "Depositing amounts: {} of A and {} of B",
            new_amount_a,
            new_amount_b
        );

        // transfer tokens to vaults
        utils::token::transfer_spl(
            &self.provider,
            &self.provider_a_ata,
            &self.mint_a_vault,
            &self.mint_a,
            new_amount_a,
            &self.token_program,
            None,
        )?;

        utils::token::transfer_spl(
            &self.provider,
            &self.provider_b_ata,
            &self.mint_b_vault,
            &self.mint_b,
            new_amount_b,
            &self.token_program,
            None,
        )?;

        // mint lp token
        let liquidity = utils::math::subsequent_mint_liquidity(
            amount_a,
            amount_b,
            self.liquidity_pool.lp_supply,
            self.liquidity_pool.amount_mint_a,
            self.liquidity_pool.amount_mint_b,
        )?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED,
            &pool_id.to_le_bytes(),
            &[self.liquidity_pool.bump],
        ]];

        utils::token::mint_to(
            &self.liquidity_pool.to_account_info(),
            &self.lp_mint,
            &self.provider_lp_ata,
            liquidity,
            &self.token_program,
            signer_seeds,
        )?;

        // update liquidity pool state
        self.liquidity_pool.amount_mint_a = self
            .liquidity_pool
            .amount_mint_a
            .checked_add(new_amount_a)
            .ok_or(AmmError::MathOverflow)?;

        self.liquidity_pool.amount_mint_b = self
            .liquidity_pool
            .amount_mint_b
            .checked_add(new_amount_b)
            .ok_or(AmmError::MathOverflow)?;

        self.liquidity_pool.lp_supply = self
            .liquidity_pool
            .lp_supply
            .checked_add(liquidity)
            .ok_or(AmmError::MathOverflow)?;

        Ok(())
    }
}
