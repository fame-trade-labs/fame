/*
TokenCreated: Эмитируется при создании нового токена. Содержит информацию о токене и его создателе.
TokenMinted: Эмитируется при минтинге токенов. Включает информацию о токене, пользователе, количестве и цене.
TokenBurned: Эмитируется при сжигании токенов. Содержит данные о токене, пользователе, количестве сожженных токенов и возвращенных SOL.
LiquidityAdded: Эмитируется при добавлении ликвидности в пул.
LiquidityRemoved: Эмитируется при удалении ликвидности из пула.
FeeCollected: Эмитируется при сборе комиссии. Полезно для отслеживания доходов платформы.
PriceUpdate: Эмитируется при изменении цены токена. Это может быть полезно для отслеживания динамики цен.
*/

use anchor_lang::prelude::*;

#[event]
pub struct TokenCreated {
    pub token: Pubkey,
    pub name: String,
    pub symbol: String,
    pub social_account_url: String,
    pub creator: Pubkey,
}

#[event]
pub struct TokenMinted {
    pub token: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub price: u64,
}

#[event]
pub struct TokenBurned {
    pub token: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub refund: u64,
}

#[event]
pub struct LiquidityAdded {
    pub token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct LiquidityRemoved {
    pub token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct FeeCollected {
    pub token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PriceUpdate {
    pub token: Pubkey,
    pub new_price: u64,
}

#[event]
pub struct FeeWithdrawn {
    pub amount: u64,
    pub receiver: Pubkey,
}
