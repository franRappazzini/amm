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

// qM = min { (Cx / A) * M , (Cy / B) * M  }
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
