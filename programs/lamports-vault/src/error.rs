use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("exceeds max allowed withdrawal")]
    ExceedsMaxWithdraw,

}
