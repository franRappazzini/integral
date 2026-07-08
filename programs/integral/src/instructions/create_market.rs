use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    metadata::{Metadata as MetadataProgram, MetadataAccount},
    token_interface::{
        spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        TokenAccount, TokenInterface, TokenMetadataInitialize,
    },
};
use spl_type_length_value::variable_len_pack::VariableLenPack;

use crate::{
    error::IntegralError, Config, Market, MarketStatus, CONFIG_SEED, MARKET_SEED, VAULT_SEED,
};

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
        constraint = mint.key() != config.reward_mint.key() @ IntegralError::InvalidTokenMint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = market,
        token::token_program = token_program,
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
        mint::token_program = token_program,
        extensions::metadata_pointer::authority = authority,
        extensions::metadata_pointer::metadata_address = receipt_mint,
    )]
    pub receipt_mint: InterfaceAccount<'info, Mint>,

    /// The metadata account to be created
    /// CHECK: Validated by seeds constraint to be the correct PDA
    // #[account(
    //     mut,
    //     seeds = [
    //         b"metadata",
    //         token_metadata_program.key().as_ref(),
    //         receipt_mint.key().as_ref(),
    //     ],
    //     bump,

    //     seeds::program = token_metadata_program,
    // )]
    // pub metadata_account: UncheckedAccount<'info>,

    /// The mint metadata
    // #[account(
    //     seeds = [
    //         b"metadata",
    //         token_metadata_program.key().as_ref(),
    //         mint.key().as_ref(),
    //     ],
    //     bump,
    //     seeds::program = token_metadata_program,
    // )]
    // pub mint_metadata: Box<Account<'info, MetadataAccount>>,
    pub token_metadata_program: Program<'info, MetadataProgram>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(ctx: Context<CreateMarket>) -> Result<()> {
        {
            msg!("---- si ----");
            let metadata = &ctx.accounts.mint_metadata;
            // msg!("account_data: {:?}", account_data);
            // let token_metadata = Metadata::safe_deserialize(&account_data)?;

            msg!("{:?}", metadata);
        }

        let acc = ctx.accounts;

        // // cpi to token metadata program
        // let cpi_accounts = anchor_spl::metadata::CreateMetadataAccountsV3 {
        //     metadata: acc.metadata_account.to_account_info(),
        //     mint: acc.receipt_mint.to_account_info(),
        //     mint_authority: acc.market.to_account_info(),
        //     update_authority: acc.market.to_account_info(),
        //     payer: acc.authority.to_account_info(),
        //     system_program: acc.system_program.to_account_info(),
        //     rent: acc.rent.to_account_info(),
        // };

        // let mint_binding = acc.mint.key();
        // let signer_seeds: &[&[&[u8]]] =
        //     &[&[MARKET_SEED, mint_binding.as_ref(), &[ctx.bumps.market]]];

        // let cpi_ctx = CpiContext::new_with_signer(
        //     acc.token_metadata_program.key(),
        //     cpi_accounts,
        //     signer_seeds,
        // );

        // msg!("---- llega ----");

        let name = String::from("name");
        let symbol = String::from("symbol");
        let uri = String::from("uri");

        // let cpi_data = anchor_spl::metadata::mpl_token_metadata::types::DataV2 {
        //     name,
        //     symbol,
        //     uri,
        //     seller_fee_basis_points: 0,
        //     creators: None,
        //     collection: None,
        //     uses: None,
        // };

        // anchor_spl::metadata::create_metadata_accounts_v3(
        //     cpi_ctx, cpi_data, true, // is_mutable
        //     true, // update_authority_is_signer
        //     None, // collection_details
        // )?;

        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        // Add 4 extra bytes for size of MetadataExtension (2 bytes for type, 2 bytes for length)
        let data_len = 4 + token_metadata.get_packed_len()?;

        // Calculate lamports required for the additional metadata
        // let lamports =
        // data_len as u64 * DEFAULT_LAMPORTS_PER_BYTE_YEAR * DEFAULT_EXEMPTION_THRESHOLD as u64;

        let lamports = acc.rent.minimum_balance(data_len);

        // Transfer additional lamports to mint account
        system_program::transfer(
            CpiContext::new(
                acc.system_program.key(),
                system_program::Transfer {
                    from: acc.authority.to_account_info(),
                    to: acc.receipt_mint.to_account_info(),
                },
            ),
            lamports,
        )?;

        // Initialize token metadata
        token_metadata_initialize(
            CpiContext::new(
                acc.token_program.key(),
                TokenMetadataInitialize {
                    program_id: acc.token_program.to_account_info(),
                    mint: acc.receipt_mint.to_account_info(),
                    metadata: acc.receipt_mint.to_account_info(),
                    mint_authority: acc.market.to_account_info(),
                    update_authority: acc.market.to_account_info(),
                },
            ),
            name,
            symbol,
            uri,
        )?;

        // set market account
        acc.market.set_inner(Market {
            mint: acc.mint.key(),
            receipt_mint: acc.receipt_mint.key(),
            total_deposited: 0,
            total_claimed: 0,
            collected_fees: 0,
            fee_bps: acc.config.fee_bps,
            fees_claimed: false,
            status: MarketStatus::Open,
            bump_vault: ctx.bumps.vault,
            bump: ctx.bumps.market,
        });

        Ok(())
    }
}
