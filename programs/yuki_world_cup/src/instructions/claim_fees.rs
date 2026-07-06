use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{utils, Config, Market, CONFIG_SEED, MARKET_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        constraint = market.is_winner(),
        constraint = !market.fees_claimed
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = market,
        token::token_program = token_program,
        seeds = [VAULT_SEED, mint.key().as_ref()],
        bump = market.bump_vault
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program
    )]
    pub authority_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimFees<'info> {
    pub fn handler(ctx: Context<ClaimFees>) -> Result<()> {
        let acc = ctx.accounts;

        // update market account and transfer fees to authority_ata
        acc.market.fees_claimed = true;

        let mint_binding = acc.mint.key();
        let seeds = &[MARKET_SEED, mint_binding.as_ref(), &[acc.market.bump]];

        utils::token::transfer_with_signer(
            acc.market.to_account_info(),
            &acc.vault,
            &acc.authority_ata,
            &acc.mint,
            acc.market.collected_fees,
            acc.token_program.key(),
            seeds,
        )
    }
}
