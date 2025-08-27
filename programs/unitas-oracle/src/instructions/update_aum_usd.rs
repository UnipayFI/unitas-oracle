use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::AumUsdUpdated;
use crate::state::{UnitasConfig, Operator};
use crate::ADMIN_CONFIG_SEED;

#[derive(Accounts)]
pub struct UpdateAumUsd<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, UnitasConfig>,
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
    ctx.accounts.config.aum_usd = aum_usd;
    ctx.accounts.config.last_updated_timestamp = last_updated_timestamp;
    emit!(AumUsdUpdated {
        aum_usd,
        last_updated_timestamp,
        config: ctx.accounts.config.key(),
    });
    Ok(())
}
