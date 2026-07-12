use trident_fuzz::fuzzing::*;

/// Storage for all account addresses used in fuzz testing.
///
/// This struct serves as a centralized repository for account addresses,
/// enabling their reuse across different instruction flows and test scenarios.
///
/// Docs: https://ackee.xyz/trident/docs/latest/trident-api-macro/trident-types/fuzz-accounts/
#[derive(Default)]
pub struct AccountAddresses {
    pub authority: AddressStorage,

    pub config: AddressStorage,

    pub market: AddressStorage,

    pub mint: AddressStorage,

    pub signer: AddressStorage,

    pub reward_mint: AddressStorage,

    pub signer_ata: AddressStorage,

    pub reward_vault: AddressStorage,

    pub token_program: AddressStorage,

    pub vault: AddressStorage,

    pub authority_ata: AddressStorage,

    pub associated_token_program: AddressStorage,

    pub system_program: AddressStorage,

    pub farmer: AddressStorage,

    pub receipt_mint: AddressStorage,

    pub farmer_ata: AddressStorage,

    pub farmer_receipt_ata: AddressStorage,

    pub farmer_reward_ata: AddressStorage,
}
