use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{error::IntegralError, utils, Market, MARKET_SEED, VAULT_SEED};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        constraint = !market.is_winner(),
        has_one = receipt_mint,
        constraint = market.total_deposited >= amount @ IntegralError::InvalidAmount
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub receipt_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = market,
        seeds = [VAULT_SEED, mint.key().as_ref()],
        bump = market.bump_vault
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = farmer,
        associated_token::token_program = token_program
    )]
    pub farmer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = receipt_mint,
        associated_token::authority = farmer,
        associated_token::token_program = token_program
    )]
    pub farmer_receipt_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> Withdraw<'info> {
    pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, IntegralError::InvalidAmount);

        let acc = ctx.accounts;

        // update market account
        acc.market.withdraw(amount)?;

        // burn receipt tokens
        utils::token::burn(
            &acc.receipt_mint,
            &acc.farmer_receipt_ata,
            &acc.farmer,
            amount,
            acc.token_program.key(),
        )?;

        // transfer tokens from vault to farmer_ata
        let mint_binding = acc.mint.key();
        let seeds = &[MARKET_SEED, mint_binding.as_ref(), &[acc.market.bump]];

        utils::token::transfer_with_signer(
            acc.market.to_account_info(),
            &acc.vault,
            &acc.farmer_ata,
            &acc.mint,
            amount,
            acc.token_program.key(),
            seeds,
        )
    }
}
