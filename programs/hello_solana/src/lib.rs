use anchor_lang::prelude::*;

declare_id!("47QmmR3G62syungBL9qLcDhDpVchcnSqTFdHxJCwiq4x");

#[program]
pub mod hello_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, max_withdraw: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.max_withdraw = max_withdraw;
        msg!("Vault initialized with max_withdraw: {}", max_withdraw);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        
        require!(amount <= vault.max_withdraw, CustomError::MaxWithdrawExceeded);

        msg!("Withdrawing amount: {}", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub vault: Account<'info, VaultState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, VaultState>,
}

#[account]
pub struct VaultState {
    pub max_withdraw: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("The withdrawal amount exceeds the maximum allowed limit.")]
    MaxWithdrawExceeded,
}
