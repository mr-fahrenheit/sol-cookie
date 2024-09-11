use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

use crate::{state::*};

pub fn initialize(ctx: Context<Initialize>, mint: Pubkey, rules: GameRules) -> Result<()> {
    let game_data = ctx.accounts.game_data.borrow_mut();

    // Set defaults
    game_data.total_staked = 0;
    game_data.scaled_rewards_per_token = 0;
    game_data.dev_fees = 0;
    game_data.mint = mint;
    game_data.authority = ctx.accounts.authority.key();
    game_data.rules = rules;

    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    authority: Signer<'info>,

    #[account(
        init, 
        seeds = [GameAccount::SEED, authority.key().as_ref()], 
        bump, 
        payer = authority, 
        space = 8 + GameAccount::SPACE
    )]
    game_data: Account<'info, GameAccount>,

    #[account(mut)]
    token_mint: Account<'info, Mint>,

    #[account(
        init, 
        payer = authority, 
        associated_token::mint = token_mint, 
        associated_token::authority = game_data,
    )]
    game_token_account: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,

    system_program: Program<'info, System>,
}