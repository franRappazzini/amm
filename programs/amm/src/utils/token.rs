use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

pub fn transfer_spl<'info>(
    authority: &AccountInfo<'info>,
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    mint: &InterfaceAccount<'info, Mint>,
    amount: u64,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = token_interface::TransferChecked {
        authority: authority.to_account_info(),
        from: from.to_account_info(),
        to: to.to_account_info(),
        mint: mint.to_account_info(),
    };

    let mut cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
    if let Some(seeds) = signer_seeds {
        cpi_ctx = cpi_ctx.with_signer(seeds);
    }

    token_interface::transfer_checked(cpi_ctx, amount, mint.decimals)
}

pub fn mint_to<'info>(
    authority: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let cpi_accounts = token_interface::MintToChecked {
        authority: authority.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
    };

    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds);

    token_interface::mint_to_checked(cpi_ctx, amount, mint.decimals)
}

pub fn burn_spl<'info>(
    authority: &Signer<'info>,
    from: &InterfaceAccount<'info, TokenAccount>,
    mint: &InterfaceAccount<'info, Mint>,
    amount: u64,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let cpi_accounts = token_interface::BurnChecked {
        authority: authority.to_account_info(),
        from: from.to_account_info(),
        mint: mint.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);

    token_interface::burn_checked(cpi_ctx, amount, mint.decimals)
}
