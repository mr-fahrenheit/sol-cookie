use anchor_lang::{prelude::*, solana_program::clock::Slot};

#[account]
pub struct GameAccount {
    pub authority: Pubkey,
    pub scaled_rewards_per_token: u128,
    pub total_staked: u64,
    pub dev_fees: u64,
    pub mint: Pubkey,
    pub total_clicks: u64,
    pub rules: GameRules,
}

impl GameAccount {
    pub const SPACE: usize = 32 + 16 + 8 + 8 + 32 + 8 + GameRules::SPACE;
    pub const SEED: &'static [u8] = b"game"; 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct GameRules {
    pub cost_per_click: u64,
    pub win_chance: u128,
    pub tokens_on_win: u64,
    pub tokens_on_loss: u64,
    pub dev_fee: u64,
    pub dev_1: Pubkey,
    pub dev_2: Pubkey,
    pub dev_3: Pubkey,
}

impl GameRules {
    pub const SPACE: usize = 8 + 16 + 8 + 8 + 8 + 32 + 32 + 32;
}

#[account]
pub struct PlayerAccount {
    pub scaled_payout: i128,
    pub staked: u64,
    pub clicks: u64,
    pub last_clicks: u64,
    pub last_slot: Slot,
}

impl PlayerAccount {
    pub const SPACE: usize = 32 + 16 + 16;
    pub const SEED: &'static [u8] = b"player"; 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenDetails {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub initial_supply: u64,
}