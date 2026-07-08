pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("9Ay66qfjtXeHJwNDZkoko11zcnojQBsuqatwA4J8FpgJ");

#[program]
pub mod integral {
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

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ClaimRewards::handler(ctx)
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        ClaimFees::handler(ctx)
    }
}
