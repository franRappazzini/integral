use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

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

    #[account(
        init,
        payer = authority,
        mint::decimals = mint.decimals,
        mint::authority = market,
        // mint::freeze_authority = market,
    )]
    pub receipt_mint: InterfaceAccount<'info, Mint>,

    /// The metadata account to be created
    /// CHECK: Validated by seeds constraint to be the correct PDA
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            receipt_mint.key().as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program,
    )]
    pub metadata_account: UncheckedAccount<'info>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(ctx: Context<CreateMarket>) -> Result<()> {
        let acc = ctx.accounts;

        // cpi to token metadata program
        let cpi_accounts = anchor_spl::metadata::CreateMetadataAccountsV3 {
            metadata: acc.metadata_account.to_account_info(),
            mint: acc.receipt_mint.to_account_info(),
            mint_authority: acc.market.to_account_info(),
            update_authority: acc.market.to_account_info(),
            payer: acc.authority.to_account_info(),
            system_program: acc.system_program.to_account_info(),
            rent: acc.rent.to_account_info(),
        };

        let mint_binding = acc.mint.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[MARKET_SEED, mint_binding.as_ref(), &[ctx.bumps.market]]];

        let cpi_ctx = CpiContext::new_with_signer(
            acc.token_metadata_program.key(),
            cpi_accounts,
            signer_seeds,
        );

        let name = String::from("name");
        let symbol = String::from("symbol");
        let uri = String::from("uri");

        let cpi_data = anchor_spl::metadata::mpl_token_metadata::types::DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        anchor_spl::metadata::create_metadata_accounts_v3(
            cpi_ctx, cpi_data, true, // is_mutable
            true, // update_authority_is_signer
            None, // collection_details
        )?;

        // set market account
        acc.market.set_inner(Market {
            mint: acc.mint.key(),
            receipt_mint: acc.receipt_mint.key(),
            total_deposited: 0,
            fee_bps: acc.config.fee_bps,
            is_open: true,
            bump_vault: ctx.bumps.vault,
            bump: ctx.bumps.market,
        });

        Ok(())
    }
}
