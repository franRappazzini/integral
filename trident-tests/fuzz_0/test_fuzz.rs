mod constants;
mod fuzz_accounts;
mod types;

use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;

use crate::{
    constants::{MARKET_SEED, VAULT_SEED},
    types::integral::{
        program_id, AddRewardsInstruction, AddRewardsInstructionAccounts,
        AddRewardsInstructionData, ClaimFeesInstruction, ClaimFeesInstructionAccounts,
        ClaimFeesInstructionData, ClaimRewardsInstruction, ClaimRewardsInstructionAccounts,
        ClaimRewardsInstructionData, CreateMarketInstruction, CreateMarketInstructionAccounts,
        CreateMarketInstructionData, DepositInstruction, DepositInstructionAccounts,
        DepositInstructionData, InitializeInstruction, InitializeInstructionAccounts,
        InitializeInstructionData, SettleMarketInstruction, SettleMarketInstructionAccounts,
        SettleMarketInstructionData, WithdrawInstruction, WithdrawInstructionAccounts,
        WithdrawInstructionData,
    },
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
    fn start(&mut self) {
        // Perform any initialization here, this method will be executed
        // at the start of each iteration

        let authority = self.fuzz_accounts.authority.insert(&mut self.trident, None);
        self.trident.airdrop(&authority, LAMPORTS_PER_SOL * 10);

        let config = self.fuzz_accounts.config.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[constants::CONFIG_SEED],
                program_id: program_id(),
            }),
        );

        // reward mint creation
        let reward_mint = self
            .fuzz_accounts
            .reward_mint
            .insert(&mut self.trident, None);
        let mut reward_mint_ixs =
            self.trident
                .initialize_mint_2022(&authority, &reward_mint, 6, &authority, None, &[]);

        // authority ata creation
        let mut authority_ata_ixs = self.trident.initialize_associated_token_account_2022(
            &authority,
            &reward_mint,
            &authority,
            &[],
        );

        let reward_vault = self.fuzz_accounts.reward_vault.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[VAULT_SEED, reward_mint.as_ref()],
                program_id: program_id(),
            }),
        );

        let authority_ata = self.trident.get_associated_token_address(
            &reward_mint,
            &authority,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        // mint reward token to authority
        let reward_amount = self.trident.random_from_range(LAMPORTS_PER_SOL..u64::MAX);
        let mint_to_ix =
            self.trident
                .mint_to_2022(&authority_ata, &reward_mint, &authority, reward_amount);

        let fee_bps = 100;

        let ix = InitializeInstruction::data(InitializeInstructionData {
            reward_amount: reward_amount / 2,
            fee_bps,
        })
        .accounts(InitializeInstructionAccounts {
            authority,
            config,
            reward_mint,
            authority_ata,
            reward_vault,
        })
        .instruction();

        let mut ixs = vec![];
        ixs.append(&mut reward_mint_ixs);
        ixs.append(&mut authority_ata_ixs);
        ixs.push(mint_to_ix);
        ixs.push(ix);

        let tx = self.trident.process_transaction(&ixs, Some("Initialize"));

        assert!(tx.is_success());

        // outcome token 1 creation
        let mint = self.fuzz_accounts.mint.insert(&mut self.trident, None);
        let mut mint_ix = self.trident.initialize_mint_2022(
            &authority,
            &mint,
            6,
            &authority,
            None,
            &[
                MintExtension::TokenMetadata {
                    mint: mint,
                    name: "FRA wins WC26".to_string(),
                    symbol: "FRA-WC26-Y".to_string(),
                    uri: "".to_string(),
                    additional_metadata: vec![],
                    update_authority: Some(authority),
                    metadata: mint,
                },
                MintExtension::MetadataPointer {
                    authority: Some(authority),
                    metadata_address: Some(mint),
                },
            ],
        );

        let market = self.fuzz_accounts.market.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[MARKET_SEED, mint.as_ref()],
                program_id: program_id(),
            }),
        );

        let vault = self.fuzz_accounts.vault.insert(
            &mut self.trident,
            Some(PdaSeeds {
                seeds: &[VAULT_SEED, mint.as_ref()],
                program_id: program_id(),
            }),
        );

        let receipt_mint = self
            .fuzz_accounts
            .receipt_mint
            .insert(&mut self.trident, None);

        let ix = CreateMarketInstruction::data(CreateMarketInstructionData {})
            .accounts(CreateMarketInstructionAccounts {
                authority,
                config,
                market,
                mint,
                vault,
                receipt_mint,
            })
            .instruction();

        let mut ixs = vec![];
        ixs.append(&mut mint_ix);
        ixs.push(ix);

        let tx = self.trident.process_transaction(&ixs, Some("CreateMarket"));

        assert!(tx.is_success());

        // initial accounts
        let farmer = self.fuzz_accounts.farmer.insert(&mut self.trident, None);
        self.trident.airdrop(&farmer, LAMPORTS_PER_SOL);

        let mut farmer_ata_ixs =
            self.trident
                .initialize_associated_token_account_2022(&farmer, &mint, &farmer, &[]);

        let farmer_ata = self.trident.get_associated_token_address(
            &mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        let amount = self.trident.random_from_range(LAMPORTS_PER_SOL..u64::MAX);

        let mint_ix = self
            .trident
            .mint_to_2022(&farmer_ata, &mint, &authority, amount);

        let mut ixs = vec![];
        ixs.append(&mut farmer_ata_ixs);
        ixs.push(mint_ix);

        let tx = self.trident.process_transaction(&ixs, None);

        assert!(tx.is_success());
    }

    #[flow]
    fn add_rewards(&mut self) {
        // Perform logic which is meant to be fuzzed
        // This flow is selected randomly from other flows
        let signer = self.fuzz_accounts.authority.get(&mut self.trident).unwrap();
        let config = self.fuzz_accounts.config.get(&mut self.trident).unwrap();
        let reward_mint = self
            .fuzz_accounts
            .reward_mint
            .get(&mut self.trident)
            .unwrap();
        let signer_ata = self.trident.get_associated_token_address(
            &reward_mint,
            &signer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );
        let reward_vault = self
            .fuzz_accounts
            .reward_vault
            .get(&mut self.trident)
            .unwrap();

        let amount = self.trident.random_from_range(0..LAMPORTS_PER_SOL);

        let ix = AddRewardsInstruction::data(AddRewardsInstructionData { amount })
            .accounts(AddRewardsInstructionAccounts {
                signer,
                config,
                reward_mint,
                signer_ata,
                reward_vault,
            })
            .instruction();

        let tx = self.trident.process_transaction(&[ix], Some("AddRewards"));

        assert!(tx.is_success());
    }

    #[flow]
    fn deposit(&mut self) {
        // Perform logic which is meant to be fuzzed
        // This flow is selected randomly from other flows

        let farmer = self.fuzz_accounts.farmer.get(&mut self.trident).unwrap();
        let market = self.fuzz_accounts.market.get(&mut self.trident).unwrap();
        let mint = self.fuzz_accounts.mint.get(&mut self.trident).unwrap();
        let vault = self.fuzz_accounts.vault.get(&mut self.trident).unwrap();
        let receipt_mint = self
            .fuzz_accounts
            .receipt_mint
            .get(&mut self.trident)
            .unwrap();

        let farmer_ata = self.trident.get_associated_token_address(
            &mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );
        let farmer_receipt_ata = self.trident.get_associated_token_address(
            &receipt_mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        let balance = self
            .trident
            .get_token_account(farmer_ata)
            .unwrap()
            .account
            .amount;
        let amount = self.trident.random_from_range(0..(balance + 1));

        let ix = DepositInstruction::data(DepositInstructionData { amount })
            .accounts(DepositInstructionAccounts {
                farmer,
                market,
                mint,
                farmer_ata,
                vault,
                receipt_mint,
                farmer_receipt_ata,
            })
            .instruction();

        let mut ixs = vec![];
        ixs.push(ix);

        let tx = self.trident.process_transaction(&ixs, Some("Deposit"));

        assert!(tx.is_success());
    }

    #[flow]
    fn withdraw(&mut self) {
        let farmer = self.fuzz_accounts.farmer.get(&mut self.trident).unwrap();
        let market = self.fuzz_accounts.market.get(&mut self.trident).unwrap();
        let mint = self.fuzz_accounts.mint.get(&mut self.trident).unwrap();
        let receipt_mint = self
            .fuzz_accounts
            .receipt_mint
            .get(&mut self.trident)
            .unwrap();
        let vault = self.fuzz_accounts.vault.get(&mut self.trident).unwrap();
        let farmer_ata = self.trident.get_associated_token_address(
            &mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );
        let farmer_receipt_ata = self.trident.get_associated_token_address(
            &receipt_mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        let balance = self
            .trident
            .get_token_account(farmer_receipt_ata)
            .unwrap()
            .account
            .amount;
        let amount = self.trident.random_from_range(0..(balance + 1));

        let ix = WithdrawInstruction::data(WithdrawInstructionData { amount })
            .accounts(WithdrawInstructionAccounts {
                farmer,
                market,
                mint,
                receipt_mint,
                vault,
                farmer_ata,
                farmer_receipt_ata,
            })
            .instruction();

        let tx = self.trident.process_transaction(&[ix], Some("Withdraw"));

        assert!(tx.is_success());
    }

    #[flow]
    fn settle_market(&mut self) {
        let authority = self.fuzz_accounts.authority.get(&mut self.trident).unwrap();
        let config = self.fuzz_accounts.config.get(&mut self.trident).unwrap();
        let market = self.fuzz_accounts.market.get(&mut self.trident).unwrap();
        let mint = self.fuzz_accounts.mint.get(&mut self.trident).unwrap();

        let status = types::MarketStatus::Winner;

        let ix = SettleMarketInstruction::data(SettleMarketInstructionData { status })
            .accounts(SettleMarketInstructionAccounts {
                authority,
                config,
                market,
                mint,
            })
            .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("SettleMarket"));

        assert!(tx.is_success());
    }

    #[flow]
    fn claim_rewards(&mut self) {
        let farmer = self.fuzz_accounts.farmer.get(&mut self.trident).unwrap();
        let config = self.fuzz_accounts.config.get(&mut self.trident).unwrap();
        let market = self.fuzz_accounts.market.get(&mut self.trident).unwrap();
        let vault = self.fuzz_accounts.vault.get(&mut self.trident).unwrap();
        let reward_mint = self
            .fuzz_accounts
            .reward_mint
            .get(&mut self.trident)
            .unwrap();
        let mint = self.fuzz_accounts.mint.get(&mut self.trident).unwrap();
        let receipt_mint = self
            .fuzz_accounts
            .receipt_mint
            .get(&mut self.trident)
            .unwrap();
        let reward_vault = self
            .fuzz_accounts
            .reward_vault
            .get(&mut self.trident)
            .unwrap();

        let farmer_ata = self.trident.get_associated_token_address(
            &mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );
        let farmer_receipt_ata = self.trident.get_associated_token_address(
            &receipt_mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );
        let farmer_reward_ata = self.trident.get_associated_token_address(
            &reward_mint,
            &farmer,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        let ix = ClaimRewardsInstruction::data(ClaimRewardsInstructionData {})
            .accounts(ClaimRewardsInstructionAccounts {
                farmer,
                config,
                market,
                reward_mint,
                mint,
                receipt_mint,
                reward_vault,
                vault,
                farmer_ata,
                farmer_receipt_ata,
                farmer_reward_ata,
            })
            .instruction();

        let tx = self
            .trident
            .process_transaction(&[ix], Some("ClaimRewards"));

        assert!(tx.is_success());
    }

    #[flow] // or end
    fn claim_fees(&mut self) {
        let authority = self.fuzz_accounts.authority.get(&mut self.trident).unwrap();
        let config = self.fuzz_accounts.config.get(&mut self.trident).unwrap();
        let market = self.fuzz_accounts.market.get(&mut self.trident).unwrap();
        let mint = self.fuzz_accounts.mint.get(&mut self.trident).unwrap();
        let vault = self.fuzz_accounts.vault.get(&mut self.trident).unwrap();

        let authority_ata = self.trident.get_associated_token_address(
            &mint,
            &authority,
            &constants::TOKEN_2022_PROGRAM_ID,
        );

        let ix = ClaimFeesInstruction::data(ClaimFeesInstructionData {})
            .accounts(ClaimFeesInstructionAccounts {
                authority,
                config,
                market,
                mint,
                vault,
                authority_ata,
            })
            .instruction();

        let tx = self.trident.process_transaction(&[ix], Some("ClaimFees"));

        assert!(tx.is_success());
    }

    #[end]
    fn end(&mut self) {
        // Perform any cleanup here, this method will be executed
        // at the end of each iteration
        // self.claim_fees();
    }
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}
