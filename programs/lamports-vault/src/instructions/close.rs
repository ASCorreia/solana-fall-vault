use anchor_lang::prelude::*;

use crate::{VaultState, VAULT_SEED, VAULT_STATE_SEED};

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        close = user,
        seeds = [VAULT_STATE_SEED, user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

pub fn close_vault(ctx: Context<Close>) -> Result<()> {
    msg!("Closing vault for user: {}", ctx.accounts.user.key());

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
    };

    let signer_seeds = &[
        VAULT_SEED,
        &ctx.accounts.user.key().to_bytes(),
        &[ctx.accounts.vault_state.vault_bump],
    ];
    let binding = [&signer_seeds[..]];
    let cpi_ctx =
        CpiContext::new_with_signer(ctx.accounts.system_program.key(), cpi_accounts, &binding);
    anchor_lang::system_program::transfer(cpi_ctx, ctx.accounts.vault.lamports())?;
    msg!("Vault closed successfully");

    Ok(())
}
