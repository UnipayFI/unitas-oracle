use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::AumUsdUpdated;
use crate::state::{AssetLookupTable, Config, Operator};
use crate::ADMIN_CONFIG_SEED;

#[derive(Accounts)]
pub struct UpdateAumUsd<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub asset_lookup_table: Account<'info, AssetLookupTable>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
    /// CHECK: This is the operator account, it is checked in the instruction
    pub operator: Account<'info, Operator>,
    pub system_program: Program<'info, System>,
}

pub fn process_update_aum_usd(ctx: Context<UpdateAumUsd>, aum_usd: u128) -> Result<()> {
    if !ctx.accounts.config.is_admin(&ctx.accounts.user.key()) {
        require!(
            ctx.accounts.operator.user == ctx.accounts.user.key(),
            ErrorCode::InvalidOperator
        );
    } else {
        require!(
            ctx.accounts.config.is_admin(&ctx.accounts.user.key()),
            ErrorCode::InvalidAdmin
        );
    }
    let last_updated_timestamp = Clock::get()?.unix_timestamp;
    ctx.accounts.asset_lookup_table.set_aum_usd(aum_usd);
    ctx.accounts.asset_lookup_table.last_updated_timestamp = last_updated_timestamp;
    emit!(AumUsdUpdated {
        aum_usd,
        last_updated_timestamp,
        lookup_table: ctx.accounts.asset_lookup_table.key(),
    });
    Ok(())
}
