use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("The maximum allowed withdrawal limit has been exceeded")]
    MaxWithdrawalExceeded
}
