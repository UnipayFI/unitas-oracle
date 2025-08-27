use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::error::ErrorCode;
use crate::state::{AssetLookupTable, UnitasConfig};
use crate::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED};

#[derive(Accounts)]
#[instruction(args: CreateAssetLookupTableArgs)]
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
    pub asset_lookup_table: AccountLoader<'info, AssetLookupTable>,

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
    let mut asset_lookup_table = ctx.accounts.asset_lookup_table.load_init()?;
    asset_lookup_table.asset_mint = ctx.accounts.asset_mint.key();
    asset_lookup_table.oracle_account = args.oracle_account;
    asset_lookup_table.decimals = args.decimals;
    asset_lookup_table.token_account_owners_len = 0;

    Ok(())
}
