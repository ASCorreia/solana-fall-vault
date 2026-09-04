pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4ERWp6YavM5ZEQwYCwAQ7rVhiwMCjo4iJ2W4c3qQ35qc");

#[program]
pub mod lamports_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::initialize_vault(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        deposit::deposit_lamports(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        withdraw::withdraw_lamports(ctx, amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        close::close_vault(ctx)
    }
}
