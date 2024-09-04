use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{TokenInfo, BondingCurve, LiquidityPool};
use crate::errors::ErrorCode;
use crate::events::TokenCreated;


#[derive(Accounts)]
pub struct CreateToken<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = creator,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = creator,
        space = TokenInfo::LEN
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(
        init,
        payer = creator,
        space = BondingCurve::LEN
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
    
    #[account(
        init,
        payer = creator,
        space = LiquidityPool::LEN
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateBondingCurveParams<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        constraint = bonding_curve.admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub bonding_curve: Account<'info, BondingCurve>,
}


pub fn create_token(
    ctx: Context<CreateToken>,
    name: String,
    symbol: String,
    social_account_url: String,
) -> Result<()> {
    // Validate input
    require!(name.len() <= 32, ErrorCode::InvalidTokenName);
    require!(symbol.len() <= 10, ErrorCode::InvalidTokenSymbol);
    require!(social_account_url.len() <= 200, ErrorCode::InvalidSocialAccountUrl);

    // Initialize accounts
    let token_info = &mut ctx.accounts.token_info;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let creator = &ctx.accounts.creator;
    let mint = &ctx.accounts.mint;

    // Set up TokenInfo
    token_info.mint = mint.key();
    token_info.name = name.clone();
    token_info.symbol = symbol.clone();
    token_info.social_account_url = social_account_url.clone();
    token_info.total_supply = 0;
    token_info.authority = creator.key();

    // Set up BondingCurve
    bonding_curve.token = mint.key();
    bonding_curve.initial_price = 10_000_000; // 0.01 SOL (assuming 9 decimals)
    bonding_curve.slope = 92; // This represents 0.0000921 in our calculation
    bonding_curve.admin = creator.key();

    // Set up LiquidityPool
    liquidity_pool.token = mint.key();
    liquidity_pool.balance = 0;
    liquidity_pool.accumulated_fees = 0;
    liquidity_pool.authority = creator.key();

    // Emit TokenCreated event
    emit!(TokenCreated {
        token: mint.key(),
        name,
        symbol,
        social_account_url,
        creator: creator.key(),
    });

    Ok(())
}

pub fn update_bonding_curve_params(
    ctx: Context<UpdateBondingCurveParams>,
    new_initial_price: u64,
    new_slope: u64,
) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    bonding_curve.update_params(new_initial_price, new_slope)?;
    Ok(())
}