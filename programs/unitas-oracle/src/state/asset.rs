use crate::error::ErrorCode;
use anchor_lang::prelude::*;

pub const MAX_ACCOUNTS_PER_ASSET: usize = 16;

#[account(zero_copy)]
#[repr(C)]
#[derive(Debug)]
pub struct AssetLookupTable {
    // 8-byte alignment
    pub asset_mint: Pubkey,
    pub oracle_account: Pubkey,
    pub token_account_owners: [Pubkey; MAX_ACCOUNTS_PER_ASSET],
    // 4-byte alignment
    pub token_account_owners_len: u32,
    // 1-byte alignment
    pub decimals: u8,
    // trailing paddings
    pub paddings: [u8; 3],
}

impl AssetLookupTable {
    pub const LEN: usize = 8 + std::mem::size_of::<AssetLookupTable>();

    pub fn add_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        let len = self.token_account_owners_len as usize;
        require!(len < MAX_ACCOUNTS_PER_ASSET, ErrorCode::AccountLimitReached);

        for i in 0..len {
            if self.token_account_owners[i] == account {
                return err!(ErrorCode::AccountAlreadyAdded);
            }
        }

        self.token_account_owners[len] = account;
        self.token_account_owners_len += 1;
        Ok(())
    }

    pub fn remove_token_account_owner(&mut self, account: Pubkey) -> Result<()> {
        let len = self.token_account_owners_len as usize;
        if len == 0 {
            return err!(ErrorCode::InvalidAccount);
        }

        let mut found_index = None;
        for i in 0..len {
            if self.token_account_owners[i] == account {
                found_index = Some(i);
                break;
            }
        }

        if let Some(index) = found_index {
            // Swap remove
            self.token_account_owners[index] = self.token_account_owners[len - 1];
            self.token_account_owners[len - 1] = Pubkey::default();
            self.token_account_owners_len -= 1;
            Ok(())
        } else {
            err!(ErrorCode::InvalidAccount)
        }
    }
}
