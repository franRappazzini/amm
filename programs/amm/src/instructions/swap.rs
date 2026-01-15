use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::LIQUIDITY_POOL_SEED, errors::AmmError, states::LiquidityPool, utils};

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [LIQUIDITY_POOL_SEED, &pool_id.to_le_bytes()],
        bump = liquidity_pool.bump,
        constraint = liquidity_pool.mint_a == input_mint.key() || liquidity_pool.mint_a == output_mint.key() @ AmmError::InvalidMint,
        constraint = liquidity_pool.mint_b == input_mint.key() || liquidity_pool.mint_b == output_mint.key() @ AmmError::InvalidMint,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(constraint = input_mint.key() != output_mint.key() @ AmmError::IdenticalMints)]
    pub input_mint: Box<InterfaceAccount<'info, Mint>>,
    pub output_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = input_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub input_mint_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = output_mint,
        associated_token::authority = liquidity_pool,
        associated_token::token_program = token_program,
    )]
    pub output_mint_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = input_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub signer_input_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = output_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub signer_output_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(&mut self, pool_id: u64, params: SwapParams) -> Result<()> {
        // calculate and transfer input mint from signer to vault
        let input_is_mint_a = self.liquidity_pool.mint_a == self.input_mint.key();

        let (input_amount, output_amount, protocol_fee_amount) =
            self.calculate_amounts(&params, input_is_mint_a)?;

        msg!(
            "Swapping {} of input mint for {} of output mint",
            input_amount,
            output_amount
        );

        msg!("Applying protocol fee of {}", protocol_fee_amount);

        utils::token::transfer_spl(
            &self.signer,
            &self.signer_input_ata,
            &self.input_mint_vault,
            &self.input_mint,
            input_amount,
            &self.token_program,
            None,
        )?;

        // transfer output mint from vault to signer
        let signer_seeds: &[&[&[u8]]] = &[&[
            LIQUIDITY_POOL_SEED,
            &pool_id.to_le_bytes(),
            &[self.liquidity_pool.bump],
        ]];

        utils::token::transfer_spl(
            &self.liquidity_pool.to_account_info(),
            &self.output_mint_vault,
            &self.signer_output_ata,
            &self.output_mint,
            output_amount,
            &self.token_program,
            Some(signer_seeds),
        )?;

        // update liquidity pool state
        self.update_liquidity_pool(
            input_amount,
            output_amount,
            protocol_fee_amount,
            input_is_mint_a,
        )
    }

    fn calculate_amounts(
        &self,
        params: &SwapParams,
        input_is_mint_a: bool,
    ) -> Result<(u64, u64, u64)> {
        let (reserve_input, reserve_output) = if input_is_mint_a {
            (
                self.liquidity_pool.amount_mint_a,
                self.liquidity_pool.amount_mint_b,
            )
        } else {
            (
                self.liquidity_pool.amount_mint_b,
                self.liquidity_pool.amount_mint_a,
            )
        };

        utils::math::calculate_swap_amounts(
            &params,
            reserve_input,
            reserve_output,
            self.liquidity_pool.fee_bps,
            self.liquidity_pool.protocol_fee_bps,
        )
    }

    fn update_liquidity_pool(
        &mut self,
        input_amount: u64,
        output_amount: u64,
        protocol_fee_amount: u64,
        input_is_mint_a: bool,
    ) -> Result<()> {
        let pool_input = input_amount
            .checked_sub(protocol_fee_amount)
            .ok_or(AmmError::MathUnderflow)?;

        if input_is_mint_a {
            self.liquidity_pool.add_liquidity_a(pool_input)?;
            self.liquidity_pool.remove_liquidity_b(output_amount)?;
            self.liquidity_pool
                .accumulate_protocol_fee_a(protocol_fee_amount)
        } else {
            self.liquidity_pool.add_liquidity_b(pool_input)?;
            self.liquidity_pool.remove_liquidity_a(output_amount)?;
            self.liquidity_pool
                .accumulate_protocol_fee_b(protocol_fee_amount)
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum SwapParams {
    ExactIn { input_amount: u64 },
    ExactOut { output_amount: u64 },
}
