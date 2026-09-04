use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("The vault does not hold enough lamports for this withdrawal")]
    InsufficientFunds,
    #[msg("Withdrawing this amount would leave the vault below rent exemption")]
    BelowRentExempt,
    #[msg("Arithmetic overflow")]
    Overflow,
}
