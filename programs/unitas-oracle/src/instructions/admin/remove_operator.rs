use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::OperatorRemoved;
use crate::state::{Config, Operator};
use crate::{ADMIN_CONFIG_SEED, OPERATOR_SEED};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveOperator<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.is_admin(&admin.key()) @ ErrorCode::InvalidAdmin
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        close = admin,
        seeds = [OPERATOR_SEED.as_bytes(), user.as_ref()],
        bump,
        constraint = operator.user == user @ ErrorCode::InvalidOperator
    )]
    pub operator: Account<'info, Operator>,
    pub system_program: Program<'info, System>,
}

pub fn process_remove_operator(ctx: Context<RemoveOperator>, user: Pubkey) -> Result<()> {
    #[cfg(feature = "enable-log")]
    msg!(
        "remove_operator: admin:{}, user:{}, operator:{}",
        ctx.accounts.admin.key(),
        user,
        ctx.accounts.operator.key()
    );

    emit!(OperatorRemoved {
        user,
        operator: ctx.accounts.operator.key(),
    });
    Ok(())
}
