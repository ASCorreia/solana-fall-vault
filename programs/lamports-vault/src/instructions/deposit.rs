use anchor_lang::prelude::*;

use crate::{VAULT_SEED, VAULT_STATE_SEED, VaultState};

#[derive(Accounts)]
pub struct Deposit<'info> {
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

pub fn deposit_lamports(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    msg!("Depositing lamports to vault");
    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;
    msg!("Deposited {} lamports to vault", amount);

    Ok(())
}