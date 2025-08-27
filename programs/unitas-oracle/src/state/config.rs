use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct UnitasConfig {
    // Auth
    pub admin: Pubkey,
    pub pending_admin: Pubkey,

    // AUM
    pub aum_usd: u128,
    pub last_updated_timestamp: i64,

    // Other configs
    pub usdu_config: Pubkey,
}

impl UnitasConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // pending_admin
        16 + // aum_usd
        8 +  // last_updated_timestamp
        32;  // usdu_config

    pub fn is_admin(&self, key: &Pubkey) -> bool {
        self.admin == *key
    }
}

// Operator is part of the auth model, keep it here.
#[account]
#[derive(Default)]
pub struct Operator {
    pub user: Pubkey,
}

impl Operator {
    pub const LEN: usize = 8 + 32;
}
