use anchor_lang::prelude::*;

use crate::instructions::*;

pub mod instructions;
pub mod state;
pub mod error;
pub mod helpers;

declare_id!("cookieAAc8qiHmnJLuhHokCLLccipBYnKRzyFCMgcFE");

#[program]
pub mod cookie {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, mint: Pubkey, rules: state::GameRules) -> Result<()> {
        instructions::initialize(ctx, mint, rules)
    }

    pub fn initialize_with_token(ctx: Context<InitializeWithToken>, rules: state::GameRules, token_details: state::TokenDetails) -> Result<()> {
        instructions::initialize_with_token(ctx, rules, token_details)
    }

    pub fn click(ctx: Context<Click>, clicks: u64) -> Result<()> {
        instructions::click(ctx, clicks)
    }

    pub fn resolve(ctx: Context<Resolve>) -> Result<()> {
        instructions::resolve(ctx)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake(ctx, amount)
    }

    pub fn stake_all(ctx: Context<Stake>) -> Result<()> {
        instructions::stake_all(ctx)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::unstake(ctx, amount)
    }

    pub fn unstake_all(ctx: Context<Unstake>) -> Result<()> {
        instructions::unstake_all(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}

#[event]
struct JackpotEvent {
    player: Pubkey,
}