use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::IntegralError, utils, Config, Market, CONFIG_SEED, MARKET_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub farmer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = reward_mint
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [MARKET_SEED, mint.key().as_ref()],
        bump = market.bump,
        constraint = market.is_winner(),
        has_one = receipt_mint,
        constraint = market.total_deposited
                        .checked_sub(market.total_claimed)
                        .ok_or(IntegralError::MathOverflow)?
                        >= farmer_receipt_ata.amount
    )]
    pub market: Account<'info, Market>,

    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub receipt_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = config,
        token::token_program = token_program,
        seeds = [VAULT_SEED, reward_mint.key().as_ref()],
        bump = config.bump_reward_vault
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = market,
        token::token_program = token_program,
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

    #[account(
        init_if_needed,
        payer = farmer,
        associated_token::mint = reward_mint,
        associated_token::authority = farmer,
        associated_token::token_program = token_program
    )]
    pub farmer_reward_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimRewards<'info> {
    pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
        let acc = ctx.accounts;

        // update accounts (config, market)
        let amount = acc.farmer_receipt_ata.amount;
        acc.market.claim(amount)?;

        // burn receipt tokens
        utils::token::burn(
            &acc.receipt_mint,
            &acc.farmer_receipt_ata,
            &acc.farmer,
            amount,
            acc.token_program.key(),
        )?;

        // transfer outcome tokens
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
        )?;

        // calculate and transfer rewards
        let seeds = &[CONFIG_SEED, &[acc.config.bump]];

        // (farmer amount in market * total_rewards) / total_deposited in market;
        let farmer_rewards: u64 = (amount as u128)
            .checked_mul(acc.config.reward_amount as u128)
            .ok_or(IntegralError::MathOverflow)?
            .checked_div(acc.market.total_deposited as u128)
            .ok_or(IntegralError::MathOverflow)?
            .try_into()
            .map_err(|_| IntegralError::MathOverflow)?;

        require!(
            farmer_rewards
                .checked_add(acc.config.total_claimed)
                .ok_or(IntegralError::MathOverflow)?
                <= acc.config.reward_amount,
            IntegralError::InvalidAmount
        );
        acc.config.claim(farmer_rewards)?;

        utils::transfer_with_signer(
            acc.config.to_account_info(),
            &acc.reward_vault,
            &acc.farmer_reward_ata,
            &acc.reward_mint,
            farmer_rewards,
            acc.token_program.key(),
            seeds,
        )?;

        Ok(())
    }
}
