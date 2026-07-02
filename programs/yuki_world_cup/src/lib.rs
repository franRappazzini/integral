pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("6NVUFsjC6oK9TxYinznWjLgvY2WUS3p8THapPBt5Nxak");

#[program]
pub mod yuki_world_cup {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Initialize::handler(ctx)
    }

    pub fn create_market(ctx: Context<CreateMarket>, reward_amount: u64) -> Result<()> {
        CreateMarket::handler(ctx, reward_amount)
    }
}
