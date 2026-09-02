use anchor_lang::prelude::*;

use crate::{ANCHOR_DISCRIMINATOR_LENGTH, VAULT_SEED, VAULT_STATE_SEED, VaultState};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init,
        payer = user,
        space = ANCHOR_DISCRIMINATOR_LENGTH + VaultState::INIT_SPACE,
        seeds = [VAULT_STATE_SEED, user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_vault(ctx: Context<Initialize>) -> Result<()> {
    msg!("Initializing vault for user: {}", ctx.accounts.user.key());

    let cpi_acccounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.key(), cpi_acccounts);

    let rent = Rent::get()?.minimum_balance(ctx.accounts.vault.data_len());

    anchor_lang::system_program::transfer(cpi_ctx, rent)?;

    ctx.accounts.vault_state.set_inner(VaultState {
        vault_bump: ctx.bumps.vault, 
        bump: ctx.bumps.vault_state
    });
    
    Ok(())
}