use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Error : Withdrawl amount requested exceeds the Max allowed Limit.")]
    WithdrawlLimitExceedError,
}
