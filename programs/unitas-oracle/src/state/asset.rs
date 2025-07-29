use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const MAX_ACCOUNTS_PER_TABLE: usize = 25;

#[account]
#[derive(Default)]
pub struct AssetLookupTable {
    pub index: u8,
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,
    pub accounts: Vec<Pubkey>,
}

impl AssetLookupTable {
    pub const LEN: usize = 8 + // discriminator
        1 + // index
        16 + // aum_usd
        8 + // last_updated_timestamp
        4 + // vec len
        32 * MAX_ACCOUNTS_PER_TABLE; // max accounts

    pub fn add_account(&mut self, account: Pubkey) -> Result<()> {
        require!(!self.is_contains(account), ErrorCode::AccountAlreadyAdded);
        require!(self.accounts.len() < MAX_ACCOUNTS_PER_TABLE, ErrorCode::AccountLimitReached);
        
        self.accounts.push(account);
        Ok(())
    }

    pub fn remove_account(&mut self, account: Pubkey) -> Result<()> {
        if let Some(index) = self.accounts.iter().position(|acc| *acc == account) {
            self.accounts.remove(index);
            Ok(())
        } else {
            Err(ErrorCode::InvalidAccount.into())
        }
    }

    pub fn is_contains(&self, account: Pubkey) -> bool {
        self.accounts.contains(&account)
    }

    pub fn set_aum_usd(&mut self, aum_usd: u128) {
        self.aum_usd = aum_usd;
    }
}
