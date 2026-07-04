pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("6NVUFsjC6oK9TxYinznWjLgvY2WUS3p8THapPBt5Nxak");

#[program]
pub mod yuki_world_cup {
    use super::*;

    /// MANUAL settle market
    pub fn _settle_market(ctx: Context<SettleMarket>, status: MarketStatus) -> Result<()> {
        SettleMarket::handler(ctx, status)
    }

    pub fn initialize(ctx: Context<Initialize>, reward_amount: u64, fee_bps: u16) -> Result<()> {
        Initialize::handler(ctx, reward_amount, fee_bps)
    }

    pub fn add_rewards(ctx: Context<AddRewards>, amount: u64) -> Result<()> {
        AddRewards::handler(ctx, amount)
    }
    pub fn create_market(ctx: Context<CreateMarket>) -> Result<()> {
        CreateMarket::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        Deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        Withdraw::handler(ctx, amount)
    }
}
