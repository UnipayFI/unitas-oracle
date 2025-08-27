use crate::error::ErrorCode;
use anchor_lang::prelude::*;

pub const MAX_ACCOUNTS_PER_ASSET: usize = 128;

#[account]
#[derive(Default)]
pub struct AssetLookupTable {
    pub asset_mint: Pubkey,
    pub oracle_account: Pubkey,
    pub decimals: u8,
    pub token_account_owners: Vec<Pubkey>,
}

impl AssetLookupTable {
    pub const LEN: usize = 8 + // discriminator
        32 + // asset_mint
        32 + // oracle_account
        1 +  // decimals
        (4 + 32 * MAX_ACCOUNTS_PER_ASSET); // token_account_owners

    pub fn add_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        require!(
            !self.token_account_owners.contains(&account),
            ErrorCode::AccountAlreadyAdded
        );
        require!(
            self.token_account_owners.len() < MAX_ACCOUNTS_PER_ASSET,
            ErrorCode::AccountLimitReached
        );
        self.token_account_owners.push(account);
        Ok(())
    }

    pub fn remove_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        if let Some(index) = self
            .token_account_owners
            .iter()
            .position(|acc| *acc == account)
        {
            self.token_account_owners.remove(index);
            Ok(())
        } else {
            Err(ErrorCode::InvalidAccount.into())
        }
    }
}
