use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{GLOBAL_CONFIG_SEED, LIQUIDITY_POOL_SEED, LP_MINT_SEED, MINIMUM_LIQUIDITY},
    errors::AmmError,
    states::{GlobalConfig, LiquidityPool},
    utils,
};

#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        init,
        payer = creator,
        space = LiquidityPool::SIZE,
        seeds = [LIQUIDITY_POOL_SEED, &global_config.pool_count.to_le_bytes()],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(constraint = mint_a.key() != mint_b.key() @ AmmError::IdenticalMints)]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 6,
        mint::authority = liquidity_pool,
        seeds = [LP_MINT_SEED, &global_config.pool_count.to_le_bytes()],
        bump,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_a,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub mint_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint_b,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub mint_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub lp_mint_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLiquidityPool<'info> {
    pub fn create_liquidity_pool(&mut self, amount_a: u64, amount_b: u64, bump: u8) -> Result<()> {
        require!(
            amount_a > 0 && amount_b > 0,
            AmmError::InsufficientInputAmount
        );

        // transfer tokens to vaults
        utils::token::transfer_spl(
            &self.creator,
            &self.creator_a_ata,
            &self.mint_a_vault,
            &self.mint_a,
            amount_a,
            &self.token_program,
            None,
        )?;

        utils::token::transfer_spl(
            &self.creator,
            &self.creator_b_ata,
            &self.mint_b_vault,
            &self.mint_b,
            amount_b,
            &self.token_program,
            None,
        )?;

        // mint lp token
        let liquidity = utils::math::initial_mint_liquidity(amount_a, amount_b)?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED,
            &self.global_config.pool_count.to_le_bytes(),
            &[bump],
        ]];

        utils::token::mint_to(
            &self.liquidity_pool.to_account_info(),
            &self.lp_mint,
            &self.lp_mint_vault,
            MINIMUM_LIQUIDITY,
            &self.token_program,
            signer_seeds,
        )?;

        utils::token::mint_to(
            &self.liquidity_pool.to_account_info(),
            &self.lp_mint,
            &self.creator_lp_ata,
            liquidity,
            &self.token_program,
            signer_seeds,
        )?;

        // create liquidity pool state
        self.liquidity_pool.set_inner(LiquidityPool {
            creator: self.creator.key(),
            bump,
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            lp_mint: self.lp_mint.key(),
            protocol_fee_bps: self.global_config.protocol_fee_bps,
            fee_bps: self.global_config.fee_bps,
            lp_supply: liquidity
                .checked_add(MINIMUM_LIQUIDITY)
                .ok_or(AmmError::MathOverflow)?,
            amount_mint_a: amount_a,
            amount_mint_b: amount_b,
            accumulated_protocol_fee_a: 0,
            accumulated_protocol_fee_b: 0,
        });

        // update global config counter
        self.global_config.pool_count = self
            .global_config
            .pool_count
            .checked_add(1)
            .ok_or(AmmError::MathOverflow)?;

        Ok(())
    }
}
