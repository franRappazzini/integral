use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{FarmerPosition, Market, FARMER_POSITION_SEED, MARKET_SEED};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = farmer,
        space = FarmerPosition::SIZE,
        seeds = [FARMER_POSITION_SEED, farmer.key().as_ref(), market.key().as_ref()],
        bump
    )]
    pub farmer_position: Account<'info, FarmerPosition>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        // constraint = market.is_open
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = farmer,
    )]
    pub farmer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = market,
        seeds = [MARKET_SEED, mint.key().as_ref()], // same seeeds as market, different program owner
        bump = market.bump_vault
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        Ok(())
    }
}
