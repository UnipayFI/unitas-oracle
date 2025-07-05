use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::AccountRemoved;
use crate::state::{AssetLookupTable, Config, Operator};
use crate::ADMIN_CONFIG_SEED;

#[derive(Accounts)]
#[instruction(account: Pubkey)]
pub struct RemoveAccount<'info> {
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
    pub operator: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process_remove_account(ctx: Context<RemoveAccount>, account: Pubkey) -> Result<()> {
    if !ctx.accounts.config.is_admin(&ctx.accounts.user.key()) {
        let mut data: &[u8] = &ctx.accounts.operator.data.borrow();
        let operator_account = Operator::try_deserialize_unchecked(&mut data)?;
        require!(
            operator_account.user == ctx.accounts.user.key(),
            ErrorCode::InvalidOperator
        );
    } else {
        require!(
            ctx.accounts.config.is_admin(&ctx.accounts.user.key()),
            ErrorCode::InvalidAdmin
        );
    }
    ctx.accounts
        .asset_lookup_table
        .remove_account(account)?;
    emit!(AccountRemoved {
        account,
        lookup_table: ctx.accounts.asset_lookup_table.key(),
    });
    Ok(())
}
