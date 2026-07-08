use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    token_2022::{
        spl_token_2022::{
            extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
            state::Mint as Mint2022,
        },
        Token2022,
    },
    token_interface::{
        spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        TokenAccount, TokenMetadataInitialize,
    },
};

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
        mint::freeze_authority = market,
        mint::token_program = token_program,
        extensions::metadata_pointer::authority = market,
        extensions::metadata_pointer::metadata_address = receipt_mint,

    )]
    pub receipt_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateMarket<'info> {
    pub fn handler(ctx: Context<CreateMarket>) -> Result<()> {
        let acc = ctx.accounts;

        let mint_account_info = acc.mint.to_account_info();
        let mint_data = mint_account_info.try_borrow_data()?;

        // unpack mint state along with its extensions
        let mint_state = StateWithExtensions::<Mint2022>::unpack(&mint_data)?;
        let metadata: Result<TokenMetadata> =
            if let Ok(extension_bytes) = mint_state.get_extension_bytes::<TokenMetadata>() {
                // manually deserialize the borsh serialized byte slice
                TokenMetadata::try_from_slice(extension_bytes)
                    .map_err(|_| IntegralError::NoDeserializeExtension.into())
            } else {
                Err(IntegralError::NoDeserializeExtension.into())
            };

        let metadata = metadata?;

        // define token metadata
        let name = format!("I-{}", metadata.name);
        let symbol = format!("I-{}", metadata.symbol);
        let uri = metadata.uri;
        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        // calculate the space need for the mint account with the desired extensions
        let space =
            ExtensionType::try_calculate_account_len::<Mint2022>(&[ExtensionType::MetadataPointer])
                .unwrap();

        let meta_data_space = token_metadata.tlv_size_of().unwrap();

        let lamports = Rent::get()?.minimum_balance(space + meta_data_space);

        // transfer additional lamports to mint account
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

        let mint_binding = acc.mint.key();
        let signer_seeds: &[&[&[u8]]] =
            &[&[MARKET_SEED, mint_binding.as_ref(), &[ctx.bumps.market]]];

        // initialize token metadata
        token_metadata_initialize(
            CpiContext::new_with_signer(
                acc.token_program.key(),
                TokenMetadataInitialize {
                    program_id: acc.token_program.to_account_info(),
                    mint: acc.receipt_mint.to_account_info(),
                    metadata: acc.receipt_mint.to_account_info(),
                    mint_authority: acc.market.to_account_info(),
                    update_authority: acc.market.to_account_info(),
                },
                signer_seeds,
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
