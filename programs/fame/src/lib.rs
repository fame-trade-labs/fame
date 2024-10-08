use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("3u8R4PRxhPTVNsGpGmzchZM1A3viTnyvZLTEzQjTE9q6");

#[program]
pub mod fame {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn create_token(
        ctx: Context<CreateToken>,
        name: String,
        symbol: String,
        social_account_url: String,
    ) -> Result<()> {
        instructions::create_token(ctx, name, symbol, social_account_url)
    }

    pub fn mint_token(ctx: Context<MintToken>, amount_sol: u64) -> Result<()> {
        instructions::mint_token(ctx, amount_sol)
    }

    pub fn burn_token(ctx: Context<BurnToken>, amount_tokens: u64) -> Result<()> {
        instructions::burn_token(ctx, amount_tokens)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
        instructions::withdraw_fees(ctx, amount)
    }
}
