use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{utils, Config, CONFIG_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct AddRewards<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = reward_mint,
        constraint = !config.winner_settled
    )]
    pub config: Account<'info, Config>,

    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub signer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = config,
        token::token_program = token_program,
        seeds = [VAULT_SEED, reward_mint.key().as_ref()],
        bump = config.bump_reward_vault
    )]
    pub reward_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

impl<'info> AddRewards<'info> {
    pub fn handler(ctx: Context<AddRewards>, amount: u64) -> Result<()> {
        let acc = ctx.accounts;

        // transfer rewards to vault
        utils::token::transfer(
            &acc.signer,
            &acc.signer_ata,
            &acc.reward_vault,
            &acc.reward_mint,
            amount,
            acc.token_program.key(),
        )?;

        // update config account
        acc.config.add_rewards(amount)
    }
}
