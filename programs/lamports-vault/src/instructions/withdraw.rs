use anchor_lang::prelude::*;

use crate::{VaultError, VaultState, VAULT_SEED, VAULT_STATE_SEED};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        seeds = [VAULT_STATE_SEED, user.key().as_ref()],
        bump = vault_state.bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw_lamports(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    msg!("Withdrawing lamports from vault");
    let vault_state = &ctx.accounts.vault_state;
    let vault = &ctx.accounts.vault;

    //Checks
    require!(amount > 0, VaultError::ZeroAmount); //amount must be bigger than zero
    require!(amount <= vault_state.max_withdraw, VaultError::MaxWidthdraw); // amount must be below/equal to MaxWithdraw
    require!(amount <= vault.lamports(), VaultError::NotEnoughBalance); //Vault must have amount in balance

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
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;
    msg!("Withdrawn {} lamports from vault", amount);

    Ok(())
}
