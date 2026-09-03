use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Withdrawal amount requested exceeds the max allowed limit.")]
    WithdrawalLimitExceedError,
}
