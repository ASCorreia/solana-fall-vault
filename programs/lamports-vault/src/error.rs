use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Exceeds Max Withdraw")]
    ExceedsMaxWithdraw,
    // In practice this should be a sane value acceptable by protocol and its users
    // since very little max withdraw fee would lock the user fund away
    // and result in Denial of Service.
    #[msg("Max Withdraw can't be less than 0")]
    MaxWithdrawIsZero,
}
