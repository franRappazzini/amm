use trident_fuzz::fuzzing::*;

/// Storage for all account addresses used in fuzz testing.
///
/// This struct serves as a centralized repository for account addresses,
/// enabling their reuse across different instruction flows and test scenarios.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct AccountAddresses {
    pub creator: AddressStorage,

    pub global_config: AddressStorage,

    pub liquidity_pool: AddressStorage,

    pub mint_a: AddressStorage,

    pub mint_b: AddressStorage,

    pub lp_mint: AddressStorage,

    pub mint_a_vault: AddressStorage,

    pub mint_b_vault: AddressStorage,

    pub lp_mint_vault: AddressStorage,

    pub creator_a_ata: AddressStorage,

    pub creator_b_ata: AddressStorage,

    pub creator_lp_ata: AddressStorage,

    pub associated_token_program: AddressStorage,

    pub token_program: AddressStorage,

    pub system_program: AddressStorage,

    pub provider: AddressStorage,

    pub provider_a_ata: AddressStorage,

    pub provider_b_ata: AddressStorage,

    pub provider_lp_ata: AddressStorage,

    pub authority: AddressStorage,

    pub redeemer: AddressStorage,

    pub redeemer_a_ata: AddressStorage,

    pub redeemer_b_ata: AddressStorage,

    pub redeemer_lp_ata: AddressStorage,

    pub signer: AddressStorage,

    pub input_mint: AddressStorage,

    pub output_mint: AddressStorage,

    pub input_mint_vault: AddressStorage,

    pub output_mint_vault: AddressStorage,

    pub signer_input_ata: AddressStorage,

    pub signer_output_ata: AddressStorage,

    pub authority_a_ata: AddressStorage,

    pub authority_b_ata: AddressStorage,
}
