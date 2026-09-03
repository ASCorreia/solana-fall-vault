use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Invalid amount: Must be bigger than zero")]
    ZeroAmount,
    #[msg("Invalid amount: Above max withdraw")]
    MaxWidthdraw,
    #[msg("Invalid amount: Not enough balance")]
    NotEnoughBalance,
}
