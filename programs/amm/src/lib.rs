mod constants;
mod errors;
mod instructions;
mod states;
mod utils;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("92NnZLZ8TS5Ay1UwAnQmtbYWbAFcEWtZcn7MwVkLhhMZ");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
        ctx.accounts.initialize(fee_bps, ctx.bumps.global_config)
    }

    pub fn create_liquidity_pool(
        ctx: Context<CreateLiquidityPool>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        ctx.accounts
            .create_liquidity_pool(amount_a, amount_b, ctx.bumps.liquidity_pool)
    }

    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        pool_id: u64,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_liquidity(pool_id, amount_a, amount_b)
    }

    pub fn redeem_lp(ctx: Context<RedeemLp>, pool_id: u64, lp_amount: u64) -> Result<()> {
        ctx.accounts.redeem_lp(pool_id, lp_amount)
    }

    pub fn swap(ctx: Context<Swap>, pool_id: u64, params: SwapParams) -> Result<()> {
        ctx.accounts.swap(pool_id, params)
    }
}
