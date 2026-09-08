pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("EJCj8YpZHaAyfvgu7WZPT2UgHmSfYWyJ5iqHvpPi3RXM");

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


/*lamports-vault already works. Someone can create one, put SOL in, and take SOL out. 
The catch: right now they can take everything out in a single go.
Your job is to add a spending limit. Each vault gets a max_withdraw amount, set when it is created, 
and any withdrawal bigger than that gets turned away.
The change itself is small — one new field, one check, and a clear error message. 
Most of your time will go into updating the tests so they pass again. That is normal, 
and it is a big part of what changing someone else's code actually looks like. */