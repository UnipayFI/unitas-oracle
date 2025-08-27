pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("UtydtkrzVyT4UDNVp2zoEycyaonoQCKHRKa4wMyEJrw");

#[program]
pub mod unitas_oracle {
    use super::*;

    pub fn init_admin_config(ctx: Context<InitAdminConfig>, usdu_config: Pubkey) -> Result<()> {
        instructions::admin::process_init_admin_config(ctx, usdu_config)
    }

    pub fn propose_new_admin(ctx: Context<ProposeNewAdmin>) -> Result<()> {
        instructions::admin::process_propose_new_admin(ctx)
    }

    pub fn accept_admin_transfer(ctx: Context<AcceptAdminTransfer>) -> Result<()> {
        instructions::admin::process_accept_admin_transfer(ctx)
    }

    pub fn add_operator(ctx: Context<AddOperator>, user: Pubkey) -> Result<()> {
        instructions::admin::process_add_operator(ctx, user)
    }

    pub fn remove_operator(ctx: Context<RemoveOperator>, user: Pubkey) -> Result<()> {
        instructions::admin::process_remove_operator(ctx, user)
    }

    pub fn create_asset_lookup_table(
        ctx: Context<CreateAssetLookupTable>,
        args: CreateAssetLookupTableArgs,
    ) -> Result<()> {
        instructions::admin::process_create_asset_lookup_table(ctx, args)
    }

    pub fn add_account(ctx: Context<AddAccount>, account: Pubkey) -> Result<()> {
        process_add_account(ctx, account)
    }

    pub fn remove_account(ctx: Context<RemoveAccount>, account: Pubkey) -> Result<()> {
        process_remove_account(ctx, account)
    }

    pub fn update_aum_usd(ctx: Context<UpdateAumUsd>, aum_usd: u128) -> Result<()> {
        process_update_aum_usd(ctx, aum_usd)
    }
}
