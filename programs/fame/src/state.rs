use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[account]
pub struct TokenInfo {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub social_account_url: String,
    pub total_supply: u64,
    pub authority: Pubkey,
}

#[account]
pub struct BondingCurve {
    pub token: Pubkey,
    pub initial_price: u64,
    pub slope: u64,
    pub admin: Pubkey,
}

#[account]
pub struct LiquidityPool {
    pub token: Pubkey,
    pub balance: u64, 
    pub accumulated_fees: u64,
    pub authority: Pubkey,
}

#[account]
pub struct UserPortfolio {
    pub user: Pubkey,
    pub token: Pubkey,
    pub balance: u64,
}

#[account]
pub struct GlobalState {
    pub admin: Pubkey,
}


impl TokenInfo {
    pub const LEN: usize = 8 + 32 + 32 + 10 + 200 + 8 + 32;
}

impl BondingCurve {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 32;

    
    pub fn calculate_price(&self, supply: u64) -> Result<u64> {
        let price = self.initial_price
            .checked_mul(
                (self.slope as u128)
                    .checked_mul(supply as u128)
                    .ok_or(ErrorCode::ArithmeticOverflow)?
                    .checked_div(1_000_000)
                    .ok_or(ErrorCode::ArithmeticOverflow)? as u64
            )
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        Ok(price)
    }

    pub fn update_params(&mut self, initial_price: u64, slope: u64) -> Result<()> {
        self.initial_price = initial_price;
        self.slope = slope;
        Ok(())
    }
}

impl LiquidityPool {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 32;

    pub fn add_liquidity(&mut self, amount: u64) -> Result<()> {
        self.balance = self.balance.checked_add(amount).ok_or(ErrorCode::ArithmeticOverflow)?;
        Ok(())
    }

    pub fn remove_liquidity(&mut self, amount: u64) -> Result<()> {
        if self.balance < amount {
            return Err(ErrorCode::InsufficientLiquidity.into());
        }
        self.balance = self.balance.checked_sub(amount).ok_or(ErrorCode::ArithmeticOverflow)?;
        Ok(())
    }

    pub fn add_fee(&mut self, amount: u64) -> Result<()> {
        self.accumulated_fees = self.accumulated_fees.checked_add(amount).ok_or(ErrorCode::ArithmeticOverflow)?;
        Ok(())
    }
}

impl UserPortfolio {
    pub const LEN: usize = 8 + 32 + 32 + 8;
}