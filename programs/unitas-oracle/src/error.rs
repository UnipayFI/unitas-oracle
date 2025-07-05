use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid admin")]
    InvalidAdmin,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Invalid operator")]
    InvalidOperator,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Account limit reached")]
    AccountLimitReached,
    #[msg("Account already added")]
    AccountAlreadyAdded,
    #[msg("Only admin can propose new admin")]
    OnlyAdminCanProposeNewAdmin,
    #[msg("Only proposed admin can accept admin transfer")]
    OnlyProposedAdminCanAccept,
    #[msg("Proposed admin already set")]
    ProposedAdminAlreadySet,
    #[msg("Proposed admin is current admin")]
    ProposedAdminIsCurrentAdmin,
}
