use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::error::ErrorCode;
use crate::event::AccountAdded;
use crate::state::{AssetLookupTable, UnitasConfig, Operator};
use crate::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED};

#[derive(Accounts)]
#[instruction(account: Pubkey)]
pub struct AddAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ASSET_LOOKUP_TABLE_SEED.as_bytes(), asset_mint.key().as_ref()],
        bump
    )]
    pub asset_lookup_table: Account<'info, AssetLookupTable>,
    
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, UnitasConfig>,
    
    /// CHECK: This is the operator account, it is checked in the instruction
    pub operator: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn process_add_account(ctx: Context<AddAccount>, account: Pubkey) -> Result<()> {
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
        .add_token_account_owner(account)?;
    emit!(AccountAdded {
        account,
        lookup_table: ctx.accounts.asset_lookup_table.key()
    });
    Ok(())
}
