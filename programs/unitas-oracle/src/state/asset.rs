use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const MAX_ACCOUNTS_PER_TABLE: usize = 25;

#[account]
#[derive(Default)]
pub struct AssetLookupTable {
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,
    pub jlp_oracle_account: Pubkey,
    pub usdc_oracle_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub jlp_mint: Pubkey,
    pub usdu_config: Pubkey,
    pub usdc_token_account_owner: Pubkey,
    pub jlp_token_account_owners: Vec<Pubkey>,
}

impl AssetLookupTable {
    // total size: 232 bytes
    pub const LEN: usize = 8 + // discriminator
        16 + // aum_usd
        8 + // last_updated_timestamp
        32 + // jlp_oracle_account
        32 + // usdc_oracle_account
        32 + // usdc_mint
        32 + // jlp_mint
        32 + // usdu_config
        32 + // usdc_token_account_owner
        4 + // vec len
        32 * MAX_ACCOUNTS_PER_TABLE; // max accounts

    pub fn add_jlp_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        require!(!self.is_jlp_token_account_owner_contains(account), ErrorCode::AccountAlreadyAdded);
        require!(self.jlp_token_account_owners.len() < MAX_ACCOUNTS_PER_TABLE, ErrorCode::AccountLimitReached);
        
        self.jlp_token_account_owners.push(account);
        Ok(())
    }

    pub fn remove_jlp_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        if let Some(index) = self.jlp_token_account_owners.iter().position(|acc| *acc == account) {
            self.jlp_token_account_owners.remove(index);
            Ok(())
        } else {
            Err(ErrorCode::InvalidAccount.into())
        }
    }

    pub fn is_jlp_token_account_owner_contains(&self, account: Pubkey) -> bool {
        self.jlp_token_account_owners.contains(&account)
    }

    pub fn set_aum_usd(&mut self, aum_usd: u128) {
        self.aum_usd = aum_usd;
    }
}
