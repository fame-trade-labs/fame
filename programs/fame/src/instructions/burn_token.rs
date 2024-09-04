use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::state::{TokenInfo, BondingCurve, LiquidityPool, UserPortfolio};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = token_info.mint == mint.key() @ ErrorCode::InvalidToken
    )]
    pub token_info: Account<'info, TokenInfo>,

    #[account(
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

pub fn burn_token(ctx: Context<BurnToken>, amount_tokens: u64) -> Result<()> {
    let token_info = &mut ctx.accounts.token_info;
    let bonding_curve = &ctx.accounts.bonding_curve;
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let user_portfolio = &mut ctx.accounts.user_portfolio;
    let user = &ctx.accounts.user;

    // Calculate the amount of SOL to return based on the bonding curve
    let sol_to_return = calculate_sol_to_return(bonding_curve, token_info.total_supply, amount_tokens)?;

    // Calculate fee (1% of the transaction volume)
    let fee = sol_to_return / 100;
    let amount_to_user = sol_to_return - fee;

    // Ensure liquidity pool has enough balance
    require!(liquidity_pool.balance >= sol_to_return, ErrorCode::InsufficientLiquidity);

    // Update liquidity pool
    liquidity_pool.balance = liquidity_pool.balance.checked_sub(sol_to_return)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    liquidity_pool.accumulated_fees = liquidity_pool.accumulated_fees.checked_add(fee)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update token info
    token_info.total_supply = token_info.total_supply.checked_sub(amount_tokens)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Update user portfolio
    user_portfolio.balance = user_portfolio.balance.checked_sub(amount_tokens)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    // Transfer SOL from program to user
    **liquidity_pool.to_account_info().try_borrow_mut_lamports()? -= amount_to_user;
    **user.to_account_info().try_borrow_mut_lamports()? += amount_to_user;

    // Burn tokens from user
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, amount_tokens)?;

    Ok(())
}

fn calculate_sol_to_return(bonding_curve: &BondingCurve, current_supply: u64, amount_tokens: u64) -> Result<u64> {
    // Convert parameters to f64 for precise calculations
    let p0 = bonding_curve.initial_price as f64 / 1e9; // Convert to SOL (assuming 9 decimal places)
    let k = bonding_curve.slope as f64 / 1e6; // Assuming slope is stored as an integer representation of 0.0000921
    let current_supply = current_supply as f64;
    let amount_tokens = amount_tokens as f64;

    // Check for division by zero and invalid parameters
    if k == 0.0 || current_supply < amount_tokens {
        return Err(ErrorCode::InvalidBondingCurveParameters.into());
    }

    // Calculate the integral of the bonding curve function
    // The integral of P(n) = P₀ * e^(k * n) is (P₀ / k) * (e^(k * n) - 1)
    let integral = |n: f64| -> f64 {
        (p0 / k) * ((k * n).exp() - 1.0)
    };

    // Calculate the amount of SOL to return
    let start_integral = integral(current_supply);
    let end_integral = integral(current_supply - amount_tokens);
    let sol_to_return = start_integral - end_integral;

    // Check for underflow
    if sol_to_return < 0.0 {
        return Err(ErrorCode::ArithmeticUnderflow.into());
    }

    // Convert back to u64 (lamports), rounding down
    let lamports_to_return = (sol_to_return * 1e9).floor() as u64;

    Ok(lamports_to_return)
}