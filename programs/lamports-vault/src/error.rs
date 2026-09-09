use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("Error message for Exceeds Max withdraw Limit ")]
    ExceedsMaxWithdraw,
}
