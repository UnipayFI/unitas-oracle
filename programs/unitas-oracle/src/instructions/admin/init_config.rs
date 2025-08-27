use anchor_lang::prelude::*;

use crate::event::AdminConfigCreated;
use crate::state::Config;
use crate::ADMIN_CONFIG_SEED;

#[derive(Accounts)]
pub struct InitAdminConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn process_init_admin_config(ctx: Context<InitAdminConfig>, usdu_config: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.set_inner(Config {
        admin: ctx.accounts.admin.key(),
        pending_admin: Pubkey::default(),
        aum_usd: 0,
        last_updated_timestamp: 0,
        usdu_config,
    });

    emit!(AdminConfigCreated {
        admin: ctx.accounts.admin.key(),
        config: ctx.accounts.config.key(),
    });

    Ok(())
}
