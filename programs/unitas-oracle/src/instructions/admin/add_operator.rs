use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::OperatorAdded;
use crate::state::{Operator, UnitasConfig};
use crate::{ADMIN_CONFIG_SEED, OPERATOR_SEED};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddOperator<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.is_admin(&admin.key()) @ ErrorCode::InvalidAdmin
    )]
    pub config: Account<'info, UnitasConfig>,
    #[account(
        init_if_needed,
        payer = admin,
        space = Operator::LEN,
        seeds = [OPERATOR_SEED.as_bytes(), user.as_ref()],
        bump
    )]
    pub operator: Account<'info, Operator>,
    pub system_program: Program<'info, System>,
}

pub fn process_add_operator(ctx: Context<AddOperator>, user: Pubkey) -> Result<()> {
    let operator = Operator { user };
    #[cfg(feature = "enable-log")]
    msg!(
        "add_operator: admin:{}, user:{}, operator:{}",
        ctx.accounts.admin.key(),
        user,
        ctx.accounts.operator.key()
    );

    ctx.accounts.operator.set_inner(operator);
    emit!(OperatorAdded {
        user,
        operator: ctx.accounts.operator.key(),
    });
    Ok(())
}
