use anchor_lang::prelude::*;

use crate::{constants::MINIMUM_LIQUIDITY, errors::AmmError, instructions::SwapParams};

/// Calculates the initial liquidity tokens to mint for a liquidity pool.
///
/// This calculation uses the geometric mean (square root of the product) of the two deposit amounts,
/// then subtracts the minimum liquidity that will be locked forever to prevent division by zero attacks.
///
/// # Formula
/// qM = √(a*b) - 1000
///
/// liquidity = sqrt(amount_a * amount_b) - MINIMUM_LIQUIDITY
///
/// # Returns
/// The amount of liquidity tokens to mint for the liquidity provider.
///
/// # Errors
/// * `AmmError::MathOverflow` - If multiplication of amounts overflows u128 or final result exceeds u64
/// * `AmmError::MathUnderflow` - If the square root is less than MINIMUM_LIQUIDITY
pub fn initial_mint_liquidity(amount_a: u64, amount_b: u64) -> Result<u64> {
    let liquidity = (amount_a as u128)
        .checked_mul(amount_b as u128)
        .ok_or(AmmError::MathOverflow)?
        .isqrt()
        .checked_sub(MINIMUM_LIQUIDITY as u128)
        .ok_or(AmmError::MathUnderflow)?
        .try_into()
        .map_err(|_| AmmError::MathOverflow)?;

    require!(liquidity > 0, AmmError::InsufficientLiquidityMinted);

    Ok(liquidity)
}

/// Calculates the liquidity tokens to mint for subsequent deposits to an existing liquidity pool.
///
/// This function handles three cases of liquidity deposits based on the proportion of tokens deposited:
///
/// # Cases
///
/// Let `C` = amount of token to deposit, `M` = current LP token supply, `A` = reserve of token X, `B` = reserve of token Y
///
/// ## 1. Equal Proportions
/// When `Cy / Cx = B / A`, the deposit matches the pool's ratio perfectly.
///
/// The liquidity provider receives:
/// ```text
/// qM = (Cx / A) * M = (Cy / B) * M
/// ```
///
/// ## 2. Excess Token
/// When `Cy / Cx > B / A`, there's an excess of token Y.
/// ```text
/// Cy > (B * Cx) / A
/// Δy = Cy - (B * Cx) / A  (excess that will be returned)
/// ```
///
/// The liquidity provider receives `Δy` back and gets LP tokens:
/// ```text
/// qM = (Cx / A) * M = ((Cy - Δy) / B) * M < (Cy / B) * M
/// ```
///
/// ## 3. Insufficient Token
/// When `Cy / Cx < B / A`, there's insufficient token Y relative to token X.
/// ```text
/// (A * Cy) / B < Cx
/// Δx = Cx - (A * Cy) / B  (excess that will be returned)
/// ```
///
/// The liquidity provider receives `Δx` back and gets LP tokens:
/// ```text
/// qM = (Cy / B) * M = ((Cx - Δx) / A) * M < (Cx / A) * M
/// ```
///
/// # Summary Formula
/// In all cases, the liquidity provider receives:
/// ```text
/// qM = min { (Cx / A) * M , (Cy / B) * M }
/// ```
///
/// # Parameters
/// * `amount_a` - Amount of token A to deposit (Cx)
/// * `amount_b` - Amount of token B to deposit (Cy)
/// * `supply` - Current total supply of LP tokens (M)
/// * `reserve_a` - Current reserve of token A in the pool (A)
/// * `reserve_b` - Current reserve of token B in the pool (B)
///
/// # Returns
/// The amount of liquidity tokens to mint for the liquidity provider.
///
/// # Errors
/// * `AmmError::MathOverflow` - If multiplication overflows u128 or result exceeds u64
/// * `AmmError::MathUnderflow` - If division would result in underflow
/// * `AmmError::InsufficientLiquidityMinted` - If calculated liquidity is zero
pub fn subsequent_mint_liquidity(
    amount_a: u64,
    amount_b: u64,
    supply: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<u64> {
    let liquidity_a = (amount_a as u128)
        .checked_mul(supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::MathUnderflow)?;

    let liquidity_b = (amount_b as u128)
        .checked_mul(supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_b as u128)
        .ok_or(AmmError::MathUnderflow)?;

    let liquidity: u64 = liquidity_a
        .min(liquidity_b)
        .try_into()
        .map_err(|_| AmmError::MathOverflow)?;

    require!(liquidity > 0, AmmError::InsufficientLiquidityMinted);

    Ok(liquidity)
}

/// Calculates the adjusted deposit amounts after accounting for excess tokens.
///
/// When liquidity is added to a pool, the deposit must maintain the pool's ratio to be fair.
/// This function determines if there's an excess of either token and calculates the actual
/// amounts that will be deposited, with any excess being returned to the depositor.
///
/// # Cases
///
/// Let `Cx` = amount_a (deposit), `Cy` = amount_b (deposit), `A` = reserve_a, `B` = reserve_b
///
/// ## 1. Equal Proportions
/// When `Cy / Cx = B / A`, the deposit matches the pool's ratio perfectly.
///
/// No adjustments needed:
/// ```text
/// Returns: (Cx, Cy)
/// ```
///
/// ## 2. Excess Token B
/// When `Cy / Cx > B / A`, there's an excess of token B.
/// ```text
/// Cy > (B * Cx) / A
/// Δy = Cy - (B * Cx) / A  (excess to be returned)
/// ```
///
/// Returns adjusted amounts with excess removed:
/// ```text
/// Returns: (Cx, Cy - Δy)
/// ```
/// The user receives `Δy` of token B back.
///
/// ## 3. Excess Token A
/// When `Cy / Cx < B / A`, there's an excess of token A.
/// ```text
/// (A * Cy) / B < Cx
/// Δx = Cx - (A * Cy) / B  (excess to be returned)
/// ```
///
/// Returns adjusted amounts with excess removed:
/// ```text
/// Returns: (Cx - Δx, Cy)
/// ```
/// The user receives `Δx` of token A back.
///
/// # Parameters
/// * `amount_a` - Amount of token A user wants to deposit (Cx)
/// * `amount_b` - Amount of token B user wants to deposit (Cy)
/// * `reserve_a` - Current reserve of token A in the pool (A)
/// * `reserve_b` - Current reserve of token B in the pool (B)
///
/// # Returns
/// A tuple `(actual_amount_a, actual_amount_b)` representing the adjusted amounts that will
/// actually be deposited into the pool. Any difference from the input amounts will be returned
/// to the depositor.
///
/// # Errors
/// * `AmmError::MathOverflow` - If multiplication overflows u128 or result exceeds u64
/// * `AmmError::MathUnderflow` - If division or subtraction would result in underflow
pub fn calculate_deposit_excess(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<(u64, u64)> {
    let deposited = amount_b
        .checked_div(amount_a)
        .ok_or(AmmError::MathUnderflow)?;

    let reserves = reserve_b
        .checked_div(reserve_a)
        .ok_or(AmmError::MathUnderflow)?;

    if deposited > reserves {
        // Δy = Cy - (B * Cx) / A
        let res = (reserve_b as u128)
            .checked_mul(amount_a as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(AmmError::MathUnderflow)?;

        let excess_b = (amount_b as u128)
            .checked_sub(res)
            .ok_or(AmmError::MathUnderflow)?;

        let amount_b = (amount_b as u128)
            .checked_sub(excess_b)
            .ok_or(AmmError::MathUnderflow)?
            .try_into()
            .map_err(|_| AmmError::MathOverflow)?;

        Ok((amount_a, amount_b))
    } else if deposited < reserves {
        // Δx = Cx - (A * Cy) / B
        let res = (reserve_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(AmmError::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(AmmError::MathUnderflow)?;

        let excess_a = (amount_a as u128)
            .checked_sub(res)
            .ok_or(AmmError::MathUnderflow)?;

        let amount_a = (amount_a as u128)
            .checked_sub(excess_a)
            .ok_or(AmmError::MathUnderflow)?
            .try_into()
            .map_err(|_| AmmError::MathOverflow)?;

        Ok((amount_a, amount_b))
    } else {
        Ok((amount_a, amount_b))
    }
}

/// Calculates the claimable amount of a token when burning LP tokens.
///
/// When a liquidity provider wants to withdraw liquidity from the pool, they burn their LP tokens
/// and receive a proportional share of the pool's reserves based on their ownership percentage.
///
/// # Formula
/// ```text
/// claimable = (m / M) * T
/// ```
///
/// Where:
/// - `m` = amount of LP tokens being burned (lp_amount)
/// - `M` = total supply of LP tokens (lp_supply)
/// - `T` = current reserve of the token in the pool (reserve)
///
/// # Implementation Note
/// The formula is reordered to `(m * T) / M` to avoid integer division resulting in zero.
/// If we compute `m / M` first with integer arithmetic, the result would be truncated to 0
/// when `m < M`, which is almost always the case.
///
/// # Example
/// If the pool has:
/// - Total LP supply: 1000 tokens
/// - Token reserve: 5000 tokens
/// - User burns: 100 LP tokens
///
/// The user receives: `(100 * 5000) / 1000 = 500` tokens
///
/// This represents 10% of the pool (100/1000), so they get 10% of the reserves (500/5000).
///
/// # Parameters
/// * `lp_amount` - Amount of LP tokens being burned (m)
/// * `lp_supply` - Total supply of LP tokens in circulation (M)
/// * `reserve` - Current reserve of the token in the pool (T)
///
/// # Returns
/// The amount of tokens the liquidity provider can claim from the pool.
///
/// # Errors
/// * `AmmError::MathOverflow` - If multiplication overflows u128 or result exceeds u64
/// * `AmmError::MathUnderflow` - If division would result in underflow
pub fn calculate_claimable_amount(lp_amount: u64, lp_supply: u64, reserve: u64) -> Result<u64> {
    let res = (lp_amount as u128)
        .checked_mul(reserve as u128)
        .ok_or(AmmError::MathOverflow)?;

    Ok(res
        .checked_div(lp_supply as u128)
        .ok_or(AmmError::MathUnderflow)?
        .try_into()
        .map_err(|_| AmmError::MathUnderflow)?)
}

pub fn calculate_swap_amounts(
    swap_params: &SwapParams,
    reserve_input: u64,
    reserve_output: u64,
    pool_fee_bps: u16,
    protocol_fee_bps: u16,
) -> Result<(u64, u64, u64)> {
    require!(
        reserve_input > 0 && reserve_output > 0,
        AmmError::InsufficientReserve
    );

    let total_fee_bps = (pool_fee_bps)
        .checked_add(protocol_fee_bps)
        .ok_or(AmmError::MathOverflow)?;

    let fee_multiplier = 10_000u128
        .checked_sub(total_fee_bps as u128)
        .ok_or(AmmError::MathUnderflow)?;

    match swap_params {
        SwapParams::ExactIn { input_amount } => {
            require!(*input_amount > 0, AmmError::InsufficientInputAmount);
            require!(
                *input_amount < reserve_input,
                AmmError::ExcessiveInputAmount
            );

            // isolating a - if the trader deposits an amount `b` of token Y, they will receive an amount `a` of token X
            // a = A * (1 - ϕ)*b / (B + (1 - ϕ)*b)
            let input_amount_after_fee = (*input_amount as u128)
                .checked_mul(fee_multiplier)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(10_000)
                .ok_or(AmmError::MathUnderflow)?;

            msg!("input_amount_after_fee: {}", input_amount_after_fee);

            let numerator = (reserve_output as u128)
                .checked_mul(input_amount_after_fee)
                .ok_or(AmmError::MathOverflow)?;

            let denominator = (reserve_input as u128)
                .checked_add(input_amount_after_fee)
                .ok_or(AmmError::MathOverflow)?;

            let output_amount: u64 = numerator
                .checked_div(denominator)
                .ok_or(AmmError::MathUnderflow)?
                .try_into()
                .map_err(|_| AmmError::MathOverflow)?;

            // Calcular el fee del protocolo del input_amount
            let protocol_fee_amount = (*input_amount as u128)
                .checked_mul(protocol_fee_bps as u128)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(10_000)
                .ok_or(AmmError::MathUnderflow)?
                .try_into()
                .map_err(|_| AmmError::MathOverflow)?;

            Ok((*input_amount, output_amount, protocol_fee_amount))
        }
        SwapParams::ExactOut { output_amount } => {
            require!(*output_amount > 0, AmmError::InsufficientOutputAmount);
            require!(
                *output_amount < reserve_output,
                AmmError::ExcessiveOutputAmount
            );

            // isolating b - in order to receive an amount `a` of token X, the trader must deposit an amount `b` of token Y
            // b = aB / (A - a) * (1 - ϕ)
            let numerator = (*output_amount as u128)
                .checked_mul(reserve_input as u128)
                .ok_or(AmmError::MathOverflow)?;

            let denominator = (reserve_output as u128)
                .checked_sub(*output_amount as u128)
                .ok_or(AmmError::ExcessiveOutputAmount)?
                .checked_mul(fee_multiplier)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(10_000)
                .ok_or(AmmError::MathUnderflow)?;

            let input_amount: u64 = numerator
                .checked_div(denominator)
                .ok_or(AmmError::MathUnderflow)?
                .try_into()
                .map_err(|_| AmmError::MathOverflow)?;

            // Calcular el fee del protocolo del input_amount
            let protocol_fee_amount = (input_amount as u128)
                .checked_mul(protocol_fee_bps as u128)
                .ok_or(AmmError::MathOverflow)?
                .checked_div(10_000)
                .ok_or(AmmError::MathUnderflow)?
                .try_into()
                .map_err(|_| AmmError::MathOverflow)?;

            Ok((input_amount, *output_amount, protocol_fee_amount))
        }
    }
}
