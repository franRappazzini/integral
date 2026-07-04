use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{utils, FarmerPosition, Market, FARMER_POSITION_SEED, MARKET_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = farmer,
        space = FarmerPosition::SIZE,
        seeds = [FARMER_POSITION_SEED, market.key().as_ref(), farmer.key().as_ref()],
        bump
    )]
    pub farmer_position: Account<'info, FarmerPosition>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        constraint = market.is_open,
        has_one = receipt_mint
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = farmer,
        associated_token::token_program = token_program,
    )]
    pub farmer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = market,
        token::token_program = token_program,
        seeds = [VAULT_SEED, mint.key().as_ref()],
        bump = market.bump_vault,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub receipt_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = farmer,
        associated_token::mint = receipt_mint,
        associated_token::authority = farmer,
        associated_token::token_program = token_program,
    )]
    pub farmer_receipt_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let acc = ctx.accounts;

        // transfer outcome token to market vault
        utils::token::transfer(
            &acc.farmer,
            &acc.farmer_ata,
            &acc.vault,
            &acc.mint,
            amount,
            acc.token_program.key(),
        )?;

        // mint receipt token to farmer ata
        let mint_binding = acc.mint.key();
        let seeds = &[MARKET_SEED, mint_binding.as_ref(), &[acc.market.bump]];

        utils::token::mint_to_with_signer(
            &acc.receipt_mint,
            &acc.farmer_receipt_ata,
            acc.market.to_account_info(),
            amount,
            acc.token_program.key(),
            seeds,
        )?;

        // set farmer_position account if needed and update amount
        if !acc.farmer_position.is_initialized {
            acc.farmer_position.set_inner(FarmerPosition {
                amount: 0,
                is_initialized: true,
                bump: ctx.bumps.farmer_position,
            });
        }

        acc.farmer_position.deposit(amount)?;

        // update market account
        acc.market.deposit(amount)
    }
}
