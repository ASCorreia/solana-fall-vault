use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Withdrawal amount exceeds the maximum allowed")]
    WithdrawalLimitExceeded,
}
