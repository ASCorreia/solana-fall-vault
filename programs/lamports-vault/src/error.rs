use anchor_lang::prelude::*;

#[error_code]
pub enum VaultProgramError {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Withdrawal amount exceeds this vault's maximum withdrawal limit")] 
    ExceedsMaxWithdraw,
}



