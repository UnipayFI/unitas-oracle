pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("UTPY8UzTNZ9GwNbGTtAa5FVxV2upW3ucRX8DN4tnw7K");

#[program]
pub mod unitas_oracle {
    use super::*;

    pub fn init_admin_config(ctx: Context<InitAdminConfig>) -> Result<()> {
        instructions::admin::process_init_admin_config(ctx)
    }

    pub fn add_operator(ctx: Context<AddOperator>, user: Pubkey) -> Result<()> {
        instructions::admin::process_add_operator(ctx, user)
    }

    pub fn remove_operator(ctx: Context<RemoveOperator>, user: Pubkey) -> Result<()> {
        instructions::admin::process_remove_operator(ctx, user)
    }

    pub fn create_asset_lookup_table(
        ctx: Context<CreateAssetLookupTable>,
        index: u8,
        mint: Pubkey,
        decimals: u8,
    ) -> Result<()> {
        instructions::admin::process_create_asset_lookup_table(ctx, index, mint, decimals)
    }

    pub fn add_account(ctx: Context<AddAccount>, account: Pubkey) -> Result<()> {
        instructions::add_account::process_add_account(ctx, account)
    }

    pub fn remove_account(ctx: Context<RemoveAccount>, account: Pubkey) -> Result<()> {
        instructions::remove_account::process_remove_account(ctx, account)
    }

    pub fn update_aum_usd(ctx: Context<UpdateAumUsd>, aum_usd: u128) -> Result<()> {
        instructions::update_aum_usd::process_update_aum_usd(ctx, aum_usd)
    }

    pub fn propose_new_admin(ctx: Context<ProposeNewAdmin>) -> Result<()> {
        instructions::admin::process_propose_new_admin(ctx)
    }

    pub fn accept_admin_transfer(ctx: Context<AcceptAdminTransfer>) -> Result<()> {
        instructions::admin::process_accept_admin_transfer(ctx)
    }
}
