use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{Config, Market, MarketStatus, CONFIG_SEED, MARKET_SEED};

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        constraint = market.is_open()
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,
}

impl<'info> SettleMarket<'info> {
    pub fn handler(ctx: Context<SettleMarket>, status: MarketStatus) -> Result<()> {
        ctx.accounts.config.winner_settled = true;
        ctx.accounts.market.status = status;

        Ok(())
    }
}
