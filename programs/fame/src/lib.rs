use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod events;
pub mod instructions;

use instructions::*;

declare_id!("6MKdszXxg1V2E5E9ofXqov467arq5wCQUSSJETas2XHN");

#[program]
pub mod fame {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        instructions::initialize(ctx, admin)
    }

    pub fn create_token(ctx: Context<CreateToken>, name: String, symbol: String, social_account_url: String) -> Result<()> {
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

