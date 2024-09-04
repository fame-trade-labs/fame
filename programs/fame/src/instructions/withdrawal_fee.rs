use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::events::FeeWithdrawn;
use crate::state::LiquidityPool;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        constraint = liquidity_pool.authority == admin.key() @ ErrorCode::Unauthorized
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    /// CHECK: This account is not read or written in this instruction, it's just used as a fund recipient
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;

    // Ensure there are enough accumulated fees
    require!(liquidity_pool.accumulated_fees >= amount, ErrorCode::InsufficientBalance);

    // Decrease accumulated fees
    liquidity_pool.accumulated_fees = liquidity_pool.accumulated_fees
        .checked_sub(amount)
        .ok_or(ErrorCode::ArithmeticUnderflow)?;

    // Transfer SOL from liquidity pool to fee receiver
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: liquidity_pool.to_account_info(),
            to: ctx.accounts.fee_receiver.to_account_info(),
        },
    );
    system_program::transfer(cpi_context, amount)?;

    // Emit an event for fee withdrawal
    emit!(FeeWithdrawn {
        amount,
        receiver: ctx.accounts.fee_receiver.key(),
    });

    Ok(())
}
