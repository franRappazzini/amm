use anchor_lang::constant;

pub const DISCRIMINATOR_SIZE: usize = 8;

#[constant]
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global_config";

#[constant]
pub const LIQUIDITY_POOL_SEED: &[u8] = b"liquidity_pool";

#[constant]
pub const LP_MINT_SEED: &[u8] = b"lp_mint";

#[constant]
pub const MINIMUM_LIQUIDITY: u64 = 1_000;
