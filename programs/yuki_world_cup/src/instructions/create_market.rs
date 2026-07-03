use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{error::ErrorCode, Config, Market, CONFIG_SEED, MARKET_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = authority,
        space = Market::SIZE,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        constraint = mint.key() != config.reward_mint.key() @ ErrorCode::InvalidTokenMint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = market,
        seeds = [VAULT_SEED, mint.key().as_ref()],
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(ctx: Context<CreateMarket>) -> Result<()> {
        // set market
        ctx.accounts.market.set_inner(Market {
            mint: ctx.accounts.mint.key(),
            fee_bps: ctx.accounts.config.fee_bps,
            bump_vault: ctx.bumps.vault,
            bump: ctx.bumps.market,
        });

        Ok(())
    }
}
