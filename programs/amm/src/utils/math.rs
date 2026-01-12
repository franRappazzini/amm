use anchor_lang::prelude::*;

use crate::{constants::MINIMUM_LIQUIDITY, errors::AmmError};

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
