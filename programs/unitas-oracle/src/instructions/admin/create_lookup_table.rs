use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::AssetLookupTableCreated;
use crate::state::{AssetLookupTable, Config};
use crate::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED, MAX_ACCOUNTS_PER_TABLE};

#[derive(Accounts)]
#[instruction(index: u8)]
pub struct CreateAssetLookupTable<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = AssetLookupTable::LEN,
        seeds = [ASSET_LOOKUP_TABLE_SEED.as_bytes(), index.to_string().as_bytes()],
        bump
    )]
    pub asset_lookup_table: Account<'info, AssetLookupTable>,
    #[account(
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.is_admin(&admin.key()) @ ErrorCode::InvalidAdmin
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn process_create_asset_lookup_table(
    ctx: Context<CreateAssetLookupTable>,
    index: u8,
) -> Result<()> {
    #[cfg(feature = "enable-log")]
    msg!(
        "create_asset_lookup_table: admin:{}, index:{}, lookup_table:{}",
        ctx.accounts.admin.key(),
        index,
        ctx.accounts.asset_lookup_table.key()
    );
    ctx.accounts.asset_lookup_table.set_inner(AssetLookupTable {
        index,
        aum_usd: 0,
        last_updated_timestamp: 0,
        accounts: Vec::with_capacity(MAX_ACCOUNTS_PER_TABLE),
    });
    emit!(AssetLookupTableCreated {
        index,
        lookup_table: ctx.accounts.asset_lookup_table.key(),
    });
    Ok(())
}
