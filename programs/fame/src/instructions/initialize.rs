use crate::state::GlobalState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32 // discriminator + pubkey
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    global_state.admin = admin;

    msg!("Contract initialized. Admin: {:?}", admin);
    Ok(())
}
