use anchor_lang::error_code;

#[error_code]
pub enum AmmError {
    #[msg("The two mint addresses must be different")]
    IdenticalMints,
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Math operation underflowed")]
    MathUnderflow,
    #[msg("Insufficient liquidity minted")]
    InsufficientLiquidityMinted,
    #[msg("Insufficient input amount")]
    InsufficientInputAmount,
}
