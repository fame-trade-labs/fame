use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::state::{TokenInfo, BondingCurve, LiquidityPool, UserPortfolio};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = token_info.mint == mint.key() @ ErrorCode::InvalidToken
    )]
    pub token_info: Account<'info, TokenInfo>,

    #[account(
        mut,
        constraint = bonding_curve.token == mint.key() @ ErrorCode::InvalidToken
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        mut,
        constraint = liquidity_pool.token == mint.key() @ ErrorCode::InvalidToken
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        constraint = user_portfolio.user == user.key() @ ErrorCode::Unauthorized,
        constraint = user_portfolio.token == mint.key() @ ErrorCode::InvalidToken
    )]
    pub user_portfolio: Account<'info, UserPortfolio>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn mint_token(ctx: Context<MintToken>, amount_sol: u64) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    let bonding_curve = &ctx.accounts.bonding_curve;
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let user_portfolio = &mut ctx.accounts.user_portfolio;
    let user = &ctx.accounts.user;

    // Calculate the number of tokens to mint based on the bonding curve
    let tokens_to_mint = calculate_tokens_to_mint(bonding_curve, token_info.total_supply, amount_sol)?;

    // Calculate fee (1% of the transaction volume)
    let fee = amount_sol / 100;
    let amount_to_pool = amount_sol - fee;

    // Update liquidity pool
    liquidity_pool.balance = liquidity_pool.balance.checked_add(amount_to_pool)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    liquidity_pool.accumulated_fees = liquidity_pool.accumulated_fees.checked_add(fee)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update token info
    token_info.total_supply = token_info.total_supply.checked_add(tokens_to_mint)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update user portfolio
    user_portfolio.balance = user_portfolio.balance.checked_add(tokens_to_mint)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Transfer SOL from user to program
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: user.to_account_info(),
            to: liquidity_pool.to_account_info(),
        },
    );
    anchor_lang::system_program::transfer(cpi_context, amount_sol)?;

    // Mint tokens to user
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: token_info.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::mint_to(cpi_ctx, tokens_to_mint)?;

    Ok(())
}

fn calculate_tokens_to_mint(bonding_curve: &BondingCurve, current_supply: u64, amount_sol: u64) -> Result<u64> {
    // Convert parameters to f64 for precise calculations
    let p0 = bonding_curve.initial_price as f64 / 1e9; // Convert to SOL (assuming 9 decimal places)
    let k = bonding_curve.slope as f64 / 1e6; // Assuming slope is stored as an integer representation of 0.0000921
    let current_supply = current_supply as f64;
    let amount_sol = amount_sol as f64 / 1e9; // Convert to SOL

    // Calculate the integral of the bonding curve function
    // The integral of P(n) = P₀ * e^(k * n) is (P₀ / k) * (e^(k * n) - 1)
    let integral = |n: f64| -> f64 {
        (p0 / k) * ((k * n).exp() - 1.0)
    };

    // Calculate the number of tokens to mint
    let current_integral = integral(current_supply);
    let target_integral = current_integral + amount_sol;

    // Solve for n: target_integral = (P₀ / k) * (e^(k * n) - 1)
    let new_supply = (1.0 / k) * ((target_integral * k / p0 + 1.0).ln());

    // Calculate the difference to get the number of new tokens
    let tokens_to_mint = (new_supply - current_supply).max(0.0);

    // Convert back to u64, rounding down
    Ok(tokens_to_mint.floor() as u64)
}