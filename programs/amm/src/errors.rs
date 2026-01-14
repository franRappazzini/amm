use anchor_lang::error_code;

#[error_code]
pub enum AmmError {
    #[msg("The two mint addresses must be different")]
    IdenticalMints,
    #[msg("The provided mint is invalid")]
    InvalidMint,
    #[msg("Math operation overflowed")]
    MathOverflow,
    #[msg("Math operation underflowed")]
    MathUnderflow,
    #[msg("Insufficient liquidity minted")]
    InsufficientLiquidityMinted,
    #[msg("Insufficient input amount")]
    InsufficientInputAmount,
    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,
    #[msg("Excessive input amount")]
    ExcessiveInputAmount,
    #[msg("Excessive output amount")]
    ExcessiveOutputAmount,
}
