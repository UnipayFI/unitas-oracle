use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::AssetLookupTableCreated;
use crate::state::{AssetLookupTable, Config};
use crate::{ADMIN_CONFIG_SEED, ASSET_LOOKUP_TABLE_SEED, MAX_ACCOUNTS_PER_TABLE};

#[derive(Accounts)]
pub struct CreateAssetLookupTable<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = AssetLookupTable::LEN,
        seeds = [ASSET_LOOKUP_TABLE_SEED.as_bytes()],
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

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateAssetLookupTableArgs {
    pub jlp_oracle_account: Pubkey,
    pub usdc_oracle_account: Pubkey,
    pub usdc_mint: Pubkey,
    pub jlp_mint: Pubkey,
    pub usdu_config: Pubkey,
}

pub fn process_create_asset_lookup_table(
    ctx: Context<CreateAssetLookupTable>,
    args: CreateAssetLookupTableArgs,
) -> Result<()> {
    #[cfg(feature = "enable-log")]
    msg!(
        "create_asset_lookup_table: admin:{}, index:{}, lookup_table:{}",
        ctx.accounts.admin.key(),
        index,
        ctx.accounts.asset_lookup_table.key()
    );
    ctx.accounts.asset_lookup_table.set_inner(AssetLookupTable {
        aum_usd: 0,
        last_updated_timestamp: 0,
        jlp_oracle_account: args.jlp_oracle_account,
        usdc_oracle_account: args.usdc_oracle_account,
        usdc_mint: args.usdc_mint,
        jlp_mint: args.jlp_mint,
        usdu_config: args.usdu_config,
        token_account_owners: Vec::with_capacity(MAX_ACCOUNTS_PER_TABLE),
    });
    emit!(AssetLookupTableCreated {
        lookup_table: ctx.accounts.asset_lookup_table.key(),
    });
    Ok(())
}
