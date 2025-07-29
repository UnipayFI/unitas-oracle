use anchor_lang::prelude::*;

/// Admin config
#[event]
pub struct AdminConfigCreated {
    pub admin: Pubkey,
    pub config: Pubkey,
}

#[event]
pub struct OperatorAdded {
    pub operator: Pubkey,
}

#[event]
pub struct OperatorRemoved {
    pub operator: Pubkey,
}

#[event]
pub struct AdminTransferProposed {
    pub current_admin: Pubkey,
    pub proposed_admin: Pubkey,
}

#[event]
pub struct AdminTransferAccepted {
    pub current_admin: Pubkey,
    pub proposed_admin: Pubkey,
}

#[event]
pub struct AdminTransferCompleted {
    pub previous_admin: Pubkey,
    pub new_admin: Pubkey,
}

/// Asset lookup table
#[event]
pub struct AssetLookupTableCreated {
    pub index: u8,
    pub lookup_table: Pubkey,
}

#[event]
pub struct AccountAdded {
    pub account: Pubkey,
    pub lookup_table: Pubkey,
}

#[event]
pub struct AccountRemoved {
    pub account: Pubkey,
    pub lookup_table: Pubkey,
}

#[event]
pub struct AumUsdUpdated {
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,
    pub lookup_table: Pubkey,
}
