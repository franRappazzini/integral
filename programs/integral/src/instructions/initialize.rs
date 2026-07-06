use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{utils, Config, CONFIG_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Config::SIZE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

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
        token::mint = reward_mint,
        token::authority = config,
        token::token_program = token_program,
        seeds = [VAULT_SEED, reward_mint.key().as_ref()],
        bump
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn handler(ctx: Context<Initialize>, reward_amount: u64, fee_bps: u16) -> Result<()> {
        let acc = ctx.accounts;

        // transfer rewards to vault
        utils::token::transfer(
            &acc.authority,
            &acc.authority_ata,
            &acc.reward_vault,
            &acc.reward_mint,
            reward_amount,
            acc.token_program.key(),
        )?;

        // set config account
        acc.config.set_inner(Config {
            authority: acc.authority.key(),
            reward_mint: acc.reward_mint.key(),
            reward_amount,
            total_claimed: 0,
            fee_bps,
            bump_reward_vault: ctx.bumps.reward_vault,
            bump: ctx.bumps.config,
        });

        Ok(())
    }
}
