use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;

mod fuzz_accounts;
mod types;

use crate::types::{
    amm::{
        self, CreateLiquidityPoolInstruction, CreateLiquidityPoolInstructionAccounts,
        CreateLiquidityPoolInstructionData, DepositLiquidityInstruction,
        DepositLiquidityInstructionAccounts, DepositLiquidityInstructionData,
        InitializeInstruction, InitializeInstructionAccounts, InitializeInstructionData,
        RedeemLpInstruction, RedeemLpInstructionAccounts, RedeemLpInstructionData, SwapInstruction,
        SwapInstructionAccounts, SwapInstructionData, WithdrawProtocolFeesInstruction,
        WithdrawProtocolFeesInstructionAccounts, WithdrawProtocolFeesInstructionData,
    },
    GlobalConfig, LiquidityPool, SwapParams,
};

#[derive(FuzzTestMethods)]
struct FuzzTest {
    /// Trident client for interacting with the Solana program
    trident: Trident,
    /// Storage for all account addresses used in fuzz testing
    fuzz_accounts: AccountAddresses,
}

#[flow_executor]
impl FuzzTest {
    fn new() -> Self {
        Self {
            trident: Trident::default(),
            fuzz_accounts: AccountAddresses::default(),
        }
    }

    #[init]
    fn initialize_and_create_liquidity_pool(&mut self) {
        // Perform any initialization here, this method will be executed
        // at the start of each iteration
        let protocol_fee_bps = 250;
        let fee_bps = 50;

        let authority = self.fuzz_accounts.authority.insert(&mut self.trident, None);
        self.trident.airdrop(&authority, LAMPORTS_PER_SOL);

        let global_config = self.fuzz_accounts.global_config.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[b"global_config"],
                program_id: amm::program_id(),
            }),
        );

        let ix =
            InitializeInstruction::data(InitializeInstructionData::new(protocol_fee_bps, fee_bps))
                .accounts(InitializeInstructionAccounts::new(authority, global_config))
                .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("Initialize ix"));

        assert!(tx.is_success());

        // ---- create liquidity pool ix ----

        let creator = self.fuzz_accounts.creator.insert(&mut self.trident, None);
        self.trident.airdrop(&creator, LAMPORTS_PER_SOL);

        let liquidity_pool = self.fuzz_accounts.liquidity_pool.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[b"liquidity_pool", &0u64.to_le_bytes()],
                program_id: amm::program_id(),
            }),
        );

        // -- mint creation ixs
        let mut ixs = vec![];

        let mint_a_address = self.fuzz_accounts.mint_a.insert(&mut self.trident, None);
        let mut mint_a_ix =
            self.trident
                .initialize_mint(&authority, &mint_a_address, 6, &authority, None);

        let mint_b_address = self.fuzz_accounts.mint_b.insert(&mut self.trident, None);
        let mut mint_b_ix =
            self.trident
                .initialize_mint(&authority, &mint_b_address, 6, &authority, None);

        let lp_mint_address = self.fuzz_accounts.lp_mint.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[b"lp_mint", &0u64.to_le_bytes()],
                program_id: amm::program_id(),
            }),
        );

        // -- ata vaults creation ixs
        let creator_a_ata_ix =
            self.trident
                .initialize_associated_token_account(&creator, &mint_a_address, &creator);
        let creator_b_ata_ix =
            self.trident
                .initialize_associated_token_account(&creator, &mint_b_address, &creator);

        let token_program = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        self.fuzz_accounts
            .token_program
            .insert_with_address(token_program);

        let mint_a_vault = self.trident.get_associated_token_address(
            &mint_a_address,
            &liquidity_pool,
            &token_program,
        );
        self.fuzz_accounts
            .mint_a_vault
            .insert_with_address(mint_a_vault);

        let mint_b_vault = self.trident.get_associated_token_address(
            &mint_b_address,
            &liquidity_pool,
            &token_program,
        );
        self.fuzz_accounts
            .mint_b_vault
            .insert_with_address(mint_b_vault);

        let lp_mint_vault = self.trident.get_associated_token_address(
            &lp_mint_address,
            &liquidity_pool,
            &token_program,
        );
        self.fuzz_accounts
            .lp_mint_vault
            .insert_with_address(lp_mint_vault);

        let creator_a_ata =
            self.trident
                .get_associated_token_address(&mint_a_address, &creator, &token_program);
        self.fuzz_accounts
            .creator_a_ata
            .insert_with_address(creator_a_ata);
        let creator_b_ata =
            self.trident
                .get_associated_token_address(&mint_b_address, &creator, &token_program);
        self.fuzz_accounts
            .creator_b_ata
            .insert_with_address(creator_b_ata);
        let creator_lp_ata =
            self.trident
                .get_associated_token_address(&lp_mint_address, &creator, &token_program);
        self.fuzz_accounts
            .creator_lp_ata
            .insert_with_address(creator_lp_ata);

        // -- mint a & b tokens
        let mint_a_amount = 1_000_000_000_000;
        let mint_to_a_ix =
            self.trident
                .mint_to(&creator_a_ata, &mint_a_address, &authority, mint_a_amount);

        let mint_b_amount = 2_000_000_000_000;
        let mint_to_b_ix =
            self.trident
                .mint_to(&creator_b_ata, &mint_b_address, &authority, mint_b_amount);

        let amount_a = 100_000_000;
        let amount_b = 200_000_000;

        let ix = CreateLiquidityPoolInstruction::data(CreateLiquidityPoolInstructionData::new(
            amount_a, amount_b,
        ))
        .accounts(CreateLiquidityPoolInstructionAccounts::new(
            creator,
            global_config,
            liquidity_pool,
            mint_a_address,
            mint_b_address,
            lp_mint_address,
            mint_a_vault,
            mint_b_vault,
            lp_mint_vault,
            creator_a_ata,
            creator_b_ata,
            creator_lp_ata,
            token_program,
        ))
        .instruction();

        // create a single vec ixs
        ixs.append(&mut mint_a_ix);
        ixs.append(&mut mint_b_ix);
        // ixs.append(&mut lp_mint_ix);

        // ixs.push(mint_a_vault_ix);
        // ixs.push(mint_b_vault_ix);
        // ixs.push(lp_mint_vault_ix);
        ixs.push(creator_a_ata_ix);
        ixs.push(creator_b_ata_ix);
        // ixs.push(creator_lp_ata_ix);

        ixs.push(mint_to_a_ix);
        ixs.push(mint_to_b_ix);

        ixs.push(ix);

        let tx = self
            .trident
            .process_transaction(&ixs, Some("Create Liquidity Pool ix"));

        assert!(tx.is_success());
    }

    #[flow]
    fn deposit_liquidity(&mut self) {
        // Perform logic which is meant to be fuzzed
        // This flow is selected randomly from other flows

        let provider = self
            .fuzz_accounts
            .creator
            .get(&mut self.trident)
            .expect("account not found");

        let liquidity_pool = self
            .fuzz_accounts
            .liquidity_pool
            .get(&mut self.trident)
            .expect("account not found");

        let liquidity_pool_account = self
            .trident
            .get_account_with_type::<LiquidityPool>(&liquidity_pool, 8)
            .expect("account not found");

        let token_program = self
            .fuzz_accounts
            .token_program
            .get(&mut self.trident)
            .expect("account not found");

        let mint_a_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &liquidity_pool,
            &token_program,
        );
        let mint_b_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &liquidity_pool,
            &token_program,
        );

        let provider_a_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &provider,
            &token_program,
        );
        let provider_b_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &provider,
            &token_program,
        );
        let provider_lp_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.lp_mint,
            &provider,
            &token_program,
        );

        let global_config = self
            .fuzz_accounts
            .global_config
            .get(&mut self.trident)
            .expect("account not found");

        let global_config_account = self
            .trident
            .get_account_with_type::<GlobalConfig>(&global_config, 8)
            .expect("account not found");

        let balance_mint_a = self
            .trident
            .get_token_account(provider_a_ata)
            .expect("account not found")
            .account
            .amount;
        let balance_mint_b = self
            .trident
            .get_token_account(provider_b_ata)
            .expect("account not found")
            .account
            .amount;

        let pool_id = global_config_account.pool_count - 1;
        let amount_a = self.trident.random_from_range(0..balance_mint_a);
        let amount_b = self.trident.random_from_range(0..balance_mint_b);
        let min_lp_out = 0;

        let ix = DepositLiquidityInstruction::data(DepositLiquidityInstructionData::new(
            pool_id, amount_a, amount_b, min_lp_out,
        ))
        .accounts(DepositLiquidityInstructionAccounts::new(
            provider,
            liquidity_pool,
            liquidity_pool_account.mint_a,
            liquidity_pool_account.mint_b,
            liquidity_pool_account.lp_mint,
            mint_a_vault,
            mint_b_vault,
            provider_a_ata,
            provider_b_ata,
            provider_lp_ata,
            token_program,
        ))
        .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("Deposit Liquidity ix"));

        assert!(tx.is_success());
    }

    #[flow]
    fn swap_exact_in_mint_a(&mut self) {
        // Perform logic which is meant to be fuzzed
        // This flow is selected randomly from other flows

        let signer = self.fuzz_accounts.signer.insert(&mut self.trident, None);
        self.trident.airdrop(&signer, LAMPORTS_PER_SOL);

        let liquidity_pool = self
            .fuzz_accounts
            .liquidity_pool
            .get(&mut self.trident)
            .expect("account not found");
        let liquidity_pool_account = self
            .trident
            .get_account_with_type::<LiquidityPool>(&liquidity_pool, 8)
            .expect("account not found");

        let token_program = self
            .fuzz_accounts
            .token_program
            .get(&mut self.trident)
            .expect("account not found");

        let mint_a_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &liquidity_pool,
            &token_program,
        );

        let mint_b_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &liquidity_pool,
            &token_program,
        );

        let signer_input_ata_ix = self.trident.initialize_associated_token_account(
            &signer,
            &liquidity_pool_account.mint_a,
            &signer,
        );

        let signer_input_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &signer,
            &token_program,
        );
        let signer_output_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &signer,
            &token_program,
        );

        let authority = self
            .fuzz_accounts
            .authority
            .get(&mut self.trident)
            .expect("account not found");
        let mint_to_input_ix = self.trident.mint_to(
            &signer_input_ata,
            &liquidity_pool_account.mint_a,
            &authority,
            1_000_000_000_000,
        );

        let mut ixs = vec![];
        ixs.push(signer_input_ata_ix);
        ixs.push(mint_to_input_ix);

        self.trident.process_transaction(&ixs, None);

        let global_config = self
            .fuzz_accounts
            .global_config
            .get(&mut self.trident)
            .expect("account not found");

        let global_config_account = self
            .trident
            .get_account_with_type::<GlobalConfig>(&global_config, 8)
            .expect("account not found");

        let balance_mint_a = self
            .trident
            .get_token_account(signer_input_ata)
            .expect("account not found")
            .account
            .amount;

        let pool_id = global_config_account.pool_count - 1;
        let params = SwapParams::ExactIn {
            input_amount: self
                .trident
                .random_from_range(1..liquidity_pool_account.amount_mint_a.min(balance_mint_a)),
        };
        let slippage_limit = 0; // expressed in amount, not bps

        let ix = SwapInstruction::data(SwapInstructionData::new(pool_id, params, slippage_limit))
            .accounts(SwapInstructionAccounts::new(
                signer,
                liquidity_pool,
                liquidity_pool_account.mint_a,
                liquidity_pool_account.mint_b,
                mint_a_vault,
                mint_b_vault,
                signer_input_ata,
                signer_output_ata,
                token_program,
            ))
            .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("Swap ExactIn mint A"));

        assert!(tx.is_success());
    }

    #[flow]
    fn swap_exact_out_mint_a(&mut self) {
        // Perform logic which is meant to be fuzzed
        // This flow is selected randomly from other flows

        let signer = self.fuzz_accounts.signer.insert(&mut self.trident, None);
        self.trident.airdrop(&signer, LAMPORTS_PER_SOL);

        let liquidity_pool = self
            .fuzz_accounts
            .liquidity_pool
            .get(&mut self.trident)
            .expect("account not found") ;
        let liquidity_pool_account = self
            .trident
            .get_account_with_type::<LiquidityPool>(&liquidity_pool, 8)
            .expect("account not found");

        let token_program = self
            .fuzz_accounts
            .token_program
            .get(&mut self.trident)
            .expect("account not found");

        let mint_a_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &liquidity_pool,
            &token_program,
        );

        let mint_b_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &liquidity_pool,
            &token_program,
        );

        let signer_input_ata_ix = self.trident.initialize_associated_token_account(
            &signer,
            &liquidity_pool_account.mint_a,
            &signer,
        );

        let signer_input_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &signer,
            &token_program,
        );
        let signer_output_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &signer,
            &token_program,
        );

        let authority = self
            .fuzz_accounts
            .authority
            .get(&mut self.trident)
            .expect("account not found");
        let mint_to_input_ix = self.trident.mint_to(
            &signer_input_ata,
            &liquidity_pool_account.mint_a,
            &authority,
            1_000_000_000_000,
        );

        let mut ixs = vec![];
        ixs.push(signer_input_ata_ix);
        ixs.push(mint_to_input_ix);

        self.trident.process_transaction(&ixs, None);

        let global_config = self
            .fuzz_accounts
            .global_config
            .get(&mut self.trident)
            .expect("account not found");

        let global_config_account = self
            .trident
            .get_account_with_type::<GlobalConfig>(&global_config, 8)
            .expect("account not found");

        let pool_id = global_config_account.pool_count - 1;

        let balance_mint_a = self
            .trident
            .get_token_account(signer_input_ata)
            .expect("account not found")
            .account
            .amount;

        let random_input_amount = self
            .trident
            .random_from_range(1..(liquidity_pool_account.amount_mint_a.min(balance_mint_a)));
        let max_output_amount = calculate_output_amount(
            random_input_amount,
            liquidity_pool_account.amount_mint_a,
            liquidity_pool_account.amount_mint_b,
            liquidity_pool_account.fee_bps,
        );

        let params = SwapParams::ExactOut {
            output_amount: self.trident.random_from_range(1..max_output_amount),
        };
        let slippage_limit = 0; // expressed in amount, not bps

        let ix = SwapInstruction::data(SwapInstructionData::new(pool_id, params, slippage_limit))
            .accounts(SwapInstructionAccounts::new(
                signer,
                liquidity_pool,
                liquidity_pool_account.mint_a,
                liquidity_pool_account.mint_b,
                mint_a_vault,
                mint_b_vault,
                signer_input_ata,
                signer_output_ata,
                token_program,
            ))
            .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("Swap ExactOut mint A"));

        assert!(tx.is_success());
    }

    #[flow]
    fn redeem_lp(&mut self) {
        let redeemer = self
            .fuzz_accounts
            .creator
            .get(&mut self.trident)
            .expect("account not found");

        let liquidity_pool = self
            .fuzz_accounts
            .liquidity_pool
            .get(&mut self.trident)
            .expect("account not found");

        let liquidity_pool_account = self
            .trident
            .get_account_with_type::<LiquidityPool>(&liquidity_pool, 8)
            .expect("account not found");

        let token_program = self
            .fuzz_accounts
            .token_program
            .get(&mut self.trident)
            .expect("account not found");

        let mint_a_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &liquidity_pool,
            &token_program,
        );
        let mint_b_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &liquidity_pool,
            &token_program,
        );

        let redeemer_a_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &redeemer,
            &token_program,
        );
        let redeemer_b_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &redeemer,
            &token_program,
        );
        let redeemer_lp_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.lp_mint,
            &redeemer,
            &token_program,
        );

        let global_config = self
            .fuzz_accounts
            .global_config
            .get(&mut self.trident)
            .expect("account not found");

        let global_config_account = self
            .trident
            .get_account_with_type::<GlobalConfig>(&global_config, 8)
            .expect("account not found");

        let balance_lp_mint = self
            .trident
            .get_token_account(redeemer_lp_ata)
            .expect("account not found")
            .account
            .amount;

        let pool_id = global_config_account.pool_count - 1;
        let lp_amount = self.trident.random_from_range(1..balance_lp_mint);

        let ix = RedeemLpInstruction::data(RedeemLpInstructionData::new(pool_id, lp_amount))
            .accounts(RedeemLpInstructionAccounts::new(
                redeemer,
                liquidity_pool,
                liquidity_pool_account.mint_a,
                liquidity_pool_account.mint_b,
                liquidity_pool_account.lp_mint,
                mint_a_vault,
                mint_b_vault,
                redeemer_a_ata,
                redeemer_b_ata,
                redeemer_lp_ata,
                token_program,
            ))
            .instruction();

        let tx = self.trident.process_transaction(&[ix], Some("Redeem LP"));

        assert!(tx.is_success());
    }

    #[end]
    fn end(&mut self) {
        // Perform any cleanup here, this method will be executed
        // at the end of each iteration

        let authority = self
            .fuzz_accounts
            .authority
            .get(&mut self.trident)
            .expect("account not found");

        let global_config = self
            .fuzz_accounts
            .global_config
            .get(&mut self.trident)
            .expect("account not found");

        let pool_id = {
            let global_config_account = self
                .trident
                .get_account_with_type::<GlobalConfig>(&global_config, 8)
                .expect("account not found");
            global_config_account.pool_count - 1
        };

        let liquidity_pool = self
            .fuzz_accounts
            .liquidity_pool
            .get(&mut self.trident)
            .expect("account not found");

        let liquidity_pool_account = self
            .trident
            .get_account_with_type::<LiquidityPool>(&liquidity_pool, 8)
            .expect("account not found");

        let token_program = self
            .fuzz_accounts
            .token_program
            .get(&mut self.trident)
            .expect("account not found");

        let mint_a_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &liquidity_pool,
            &token_program,
        );
        let mint_b_vault = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &liquidity_pool,
            &token_program,
        );

        let authority_a_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_a,
            &authority,
            &token_program,
        );
        let authority_b_ata = self.trident.get_associated_token_address(
            &liquidity_pool_account.mint_b,
            &authority,
            &token_program,
        );

        let ix = WithdrawProtocolFeesInstruction::data(WithdrawProtocolFeesInstructionData::new(
            pool_id,
        ))
        .accounts(WithdrawProtocolFeesInstructionAccounts::new(
            authority,
            global_config,
            liquidity_pool,
            liquidity_pool_account.mint_a,
            liquidity_pool_account.mint_b,
            mint_a_vault,
            mint_b_vault,
            authority_a_ata,
            authority_b_ata,
            token_program,
        ))
        .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("Withdraw Protocol Fees"));

        assert!(tx.is_success());
    }
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}

fn calculate_output_amount(
    input_amount: u64,
    reserve_input: u64,
    reserve_output: u64,
    fee_multiplier: u16,
) -> u64 {
    assert!(input_amount > 0, "input_amount: {}", input_amount);
    assert!(
        input_amount < reserve_input,
        "input_amount: {} < reserve_input: {}",
        input_amount,
        reserve_input
    );

    let input_amount_after_fee = (input_amount as u128)
        .checked_mul(fee_multiplier as u128)
        .expect("math overflow")
        .checked_div(10_000)
        .expect("math underflow");

    let numerator = (reserve_output as u128)
        .checked_mul(input_amount_after_fee)
        .expect("math overflow");

    let denominator = (reserve_input as u128)
        .checked_add(input_amount_after_fee)
        .expect("math overflow");

    let output_amount: u64 = numerator
        .checked_div(denominator)
        .expect("math underflow")
        .try_into()
        .expect("math overflow");

    output_amount
}
