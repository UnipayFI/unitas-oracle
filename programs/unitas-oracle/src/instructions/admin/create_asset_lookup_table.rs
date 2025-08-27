use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::error::ErrorCode;
use crate::state::{AssetLookupTable, UnitasConfig, MAX_ACCOUNTS_PER_ASSET};
use crate::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED};

#[derive(Accounts)]
pub struct CreateAssetLookupTable<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = AssetLookupTable::LEN,
        seeds = [ASSET_LOOKUP_TABLE_SEED.as_bytes(), asset_mint.key().as_ref()],
        bump
    )]
    pub asset_lookup_table: Account<'info, AssetLookupTable>,

    pub asset_mint: Account<'info, Mint>,

    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.is_admin(&admin.key()) @ ErrorCode::InvalidAdmin
    )]
    pub config: Account<'info, UnitasConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateAssetLookupTableArgs {
    pub oracle_account: Pubkey,
    pub decimals: u8,
}

pub fn process_create_asset_lookup_table(
    ctx: Context<CreateAssetLookupTable>,
    args: CreateAssetLookupTableArgs,
) -> Result<()> {
    ctx.accounts.asset_lookup_table.set_inner(AssetLookupTable {
        asset_mint: ctx.accounts.asset_mint.key(),
        oracle_account: args.oracle_account,
        decimals: args.decimals,
        token_account_owners: Vec::with_capacity(MAX_ACCOUNTS_PER_ASSET),
    });

    Ok(())
}
