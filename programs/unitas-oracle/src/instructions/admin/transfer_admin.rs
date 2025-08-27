use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::event::{AdminTransferCompleted, AdminTransferProposed};
use crate::state::Config;
use crate::ADMIN_CONFIG_SEED;

#[derive(Accounts)]
pub struct ProposeNewAdmin<'info> {
    #[account(mut)]
    pub current_admin: Signer<'info>,

    /// CHECK: This is the proposed new admin, no signature required
    pub proposed_admin: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.admin == current_admin.key() @ ErrorCode::OnlyAdminCanProposeNewAdmin,
    )]
    pub config: Box<Account<'info, Config>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptAdminTransfer<'info> {
    #[account(mut)]
    pub new_admin: Signer<'info>,
    #[account(
        mut,
        seeds = [ADMIN_CONFIG_SEED.as_bytes()],
        bump,
        constraint = config.pending_admin == new_admin.key() @ ErrorCode::OnlyProposedAdminCanAccept,
    )]
    pub config: Box<Account<'info, Config>>,
    pub system_program: Program<'info, System>,
}

pub fn process_propose_new_admin(ctx: Context<ProposeNewAdmin>) -> Result<()> {
    let config = &mut ctx.accounts.config;

    require!(
        config.pending_admin != ctx.accounts.proposed_admin.key(),
        ErrorCode::ProposedAdminAlreadySet
    );

    require!(
        config.admin != ctx.accounts.proposed_admin.key(),
        ErrorCode::ProposedAdminIsCurrentAdmin
    );

    config.pending_admin = ctx.accounts.proposed_admin.key();

    emit!(AdminTransferProposed {
        current_admin: ctx.accounts.current_admin.key(),
        proposed_admin: ctx.accounts.proposed_admin.key(),
    });

    Ok(())
}

pub fn process_accept_admin_transfer(ctx: Context<AcceptAdminTransfer>) -> Result<()> {
    let config = &mut ctx.accounts.config;

    let previous_admin = config.admin;

    config.admin = ctx.accounts.new_admin.key();
    config.pending_admin = Pubkey::default();

    emit!(AdminTransferCompleted {
        previous_admin: previous_admin,
        new_admin: ctx.accounts.new_admin.key(),
    });

    Ok(())
}
