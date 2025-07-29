use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Option<Pubkey>,
    pub pending_admin: Option<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct Operator {
    pub config: Pubkey,
    pub user: Pubkey,
}

impl Config {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub fn is_initialized(&self) -> bool {
        self.admin.is_some()
    }

    pub fn is_admin(&self, user: &Pubkey) -> bool {
        self.admin.is_some() && self.admin.unwrap() == *user
    }
}

impl Operator {
    pub const LEN: usize = 8 + Self::INIT_SPACE;

    pub fn is_operator(&self, user: &Pubkey, config: &Pubkey) -> bool {
        self.user == *user && self.config == *config
    }
}
