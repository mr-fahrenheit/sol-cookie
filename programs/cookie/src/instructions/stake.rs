use std::borrow::BorrowMut;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}, token_interface::{transfer_checked, TransferChecked}};

use crate::{error::*, state::*};

pub fn stake_all(ctx: Context<Stake>) -> Result<()> {
    let player_balance = ctx.accounts.player_token_account.amount;
    stake(ctx, player_balance)
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    // Check player balance
    let player_balance = ctx.accounts.player_token_account.amount;
    if amount > player_balance {
        return err!(CookieError::InsufficientBalance);
    }

    // Grab data from accounts
    let game_data = ctx.accounts.game_data.borrow_mut();
    let player_data = ctx.accounts.player_data.borrow_mut();

    // Transfer tokens
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.player_token_account.to_account_info(),
                to: ctx.accounts.game_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            }
        ),
        amount as u64,
        ctx.accounts.token_mint.decimals,
    )?;

    // Perform accounting
    game_data.total_staked += amount;
    player_data.staked += amount;
    player_data.scaled_payout += amount as i128 * game_data.scaled_rewards_per_token as i128;

    // Emit event
    emit!(StakeEvent {
        player: ctx.accounts.player.key(),
        amount: amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    player: Signer<'info>,

    /// CHECK: Address constraint in account trait
    #[account(address = game_data.authority)]
    authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [GameAccount::SEED, authority.key().as_ref()], bump)]
    game_data: Account<'info, GameAccount>,

    #[account(
        init_if_needed, 
        seeds = [PlayerAccount::SEED, authority.key().as_ref(), player.key().as_ref()], 
        bump, 
        payer = player, 
        space = 8 + PlayerAccount::SPACE
    )]
    player_data: Account<'info, PlayerAccount>,

    #[account(mut, address = game_data.mint)]
    token_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint, 
        associated_token::authority = game_data,
    )]
    game_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = token_mint, 
        associated_token::authority = player,
    )]
    player_token_account: Account<'info, TokenAccount>,

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,

    system_program: Program<'info, System>,
}


#[event]
struct StakeEvent {
    player: Pubkey,
    amount: u64,
}