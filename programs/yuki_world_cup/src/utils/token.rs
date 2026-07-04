use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount};

pub fn transfer_checked<'info>(
    authority: &Signer<'info>,
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    mint: &InterfaceAccount<'info, Mint>,
    amount: u64,
    token_program: Pubkey,
) -> Result<()> {
    let cpi_accounts = token_interface::TransferChecked {
        authority: authority.to_account_info(),
        from: from.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(token_program, cpi_accounts);

    token_interface::transfer_checked(cpi_ctx, amount, mint.decimals)
}

pub fn mint_to_with_signer<'info>(
    mint: &InterfaceAccount<'info, Mint>,
    to: &InterfaceAccount<'info, TokenAccount>,
    authority: AccountInfo<'info>,
    amount: u64,
    token_program: Pubkey,
    seeds: &[&[u8]],
) -> Result<()> {
    let signer_seeds: &[&[&[u8]]] = &[seeds];

    let cpi_accounts = token_interface::MintToChecked {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority,
    };

    let cpi_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer_seeds);

    token_interface::mint_to_checked(cpi_ctx, amount, mint.decimals)
}
