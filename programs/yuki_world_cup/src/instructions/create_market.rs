use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::ErrorCode, Config, Market, CONFIG_SEED, MARKET_SEED};

#[derive(Accounts)]
pub struct CreateMarket<'info> {
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
        init,
        payer = authority,
        space = Market::SIZE,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        constraint = mint.key() != reward_mint.key() @ ErrorCode::InvalidTokenMint
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = authority,
    )]
    pub authority_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = market,
        seeds = [MARKET_SEED, mint.key().as_ref()], // same seeeds as market, different program owner
        bump
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = market,
        token::token_program = token_program,
        seeds = [MARKET_SEED, reward_mint.key().as_ref()],
        bump
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(ctx: Context<CreateMarket>, reward_amount: u64) -> Result<()> {
        let acc = ctx.accounts;

        // transfer rewards to vault
        let cpi_accounts = token::TransferChecked {
            authority: acc.authority.to_account_info(),
            from: acc.authority_ata.to_account_info(),
            to: acc.reward_vault.to_account_info(),
            mint: acc.reward_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(*acc.token_program.key, cpi_accounts);

        token::transfer_checked(cpi_ctx, reward_amount, acc.reward_mint.decimals)?;

        // set market
        acc.market.set_inner(Market {
            mint: acc.mint.key(),
            reward_mint: acc.reward_mint.key(),
            reward_amount,
            bump_vault: ctx.bumps.vault,
            bump_reward_vault: ctx.bumps.reward_vault,
            bump: ctx.bumps.market,
        });

        Ok(())
    }
}
