pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;
declare_id!("8hJ48kuf3NYwQ7L4Z92HFQLwyExxVrYLjQwALbqbH98V");

#[program]
pub mod lamports_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, max_withdraw: u64) -> Result<()> {
        initialize::initialize_vault(ctx, max_withdraw)
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
