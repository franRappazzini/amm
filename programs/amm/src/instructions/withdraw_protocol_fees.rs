use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    constants::{GLOBAL_CONFIG_SEED, LIQUIDITY_POOL_SEED},
    states::{GlobalConfig, LiquidityPool},
    utils,
};

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct WithdrawProtocolFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
        has_one = authority,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED, &pool_id.to_le_bytes()],
        bump = liquidity_pool.bump,
        has_one = mint_a,
        has_one = mint_b,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

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
        init_if_needed,
        payer = authority,
        associated_token::mint = mint_a,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
    )]
    pub authority_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint_b,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
    )]
    pub authority_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawProtocolFees<'info> {
    pub fn withdraw_protocol_fees(&mut self, pool_id: u64) -> Result<()> {
        // transfer fees from vaults to authority ATAs if there are any fees accumulated
        let accumulated_fee_a = self.liquidity_pool.accumulated_protocol_fee_a;
        let accumulated_fee_b = self.liquidity_pool.accumulated_protocol_fee_b;

        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED,
            &pool_id.to_le_bytes(),
            &[self.liquidity_pool.bump],
        ]];

        if accumulated_fee_a > 0 {
            self.liquidity_pool.accumulated_protocol_fee_a = 0;

            utils::token::transfer_spl(
                &self.liquidity_pool.to_account_info(),
                &self.mint_a_vault,
                &self.authority_a_ata,
                &self.mint_a,
                accumulated_fee_a,
                &self.token_program,
                Some(signer_seeds),
            )?;
        };
        if accumulated_fee_b > 0 {
            self.liquidity_pool.accumulated_protocol_fee_b = 0;

            utils::token::transfer_spl(
                &self.liquidity_pool.to_account_info(),
                &self.mint_b_vault,
                &self.authority_b_ata,
                &self.mint_b,
                accumulated_fee_b,
                &self.token_program,
                Some(signer_seeds),
            )?;
        };

        Ok(())
    }
}
