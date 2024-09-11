use std::borrow::BorrowMut;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}, token_interface::{mint_to, MintTo}};
use anchor_lang::{prelude::*, solana_program::sysvar::{self}};

use crate::{error::*, helpers::*, state::*, JackpotEvent};

pub fn click(ctx: Context<Click>, clicks: u64) -> Result<()> {
    if clicks == 0 || clicks > 1000 {
        return err!(CookieError::InvalidArgument)
    }

    let player_key = ctx.accounts.player.key();
    let mint_authority = ctx.accounts.game_data.to_account_info();
    let rules = ctx.accounts.game_data.rules.clone();

    let game_data = ctx.accounts.game_data.borrow_mut();
    let player_data = ctx.accounts.player_data.borrow_mut();

    let current_slot = Clock::get()?.slot;
    if player_data.last_slot == current_slot {
        return err!(CookieError::RateLimit)
    }

    let mut total_mint: u64 = 0;
    let mut total_cost: u64 = 0;

    // Resolve jackpot
    if resolve_jackpot(&ctx.accounts.slot_hashes, ctx.accounts.player.key(), player_data.last_slot, player_data.last_clicks, rules.win_chance)? {
        emit!(JackpotEvent {
            player: player_key,
        });

        let mint_amount = rules.tokens_on_win;
        total_mint += mint_amount;

        // Update token accounting
        game_data.total_staked += mint_amount;
        player_data.staked += mint_amount;
        player_data.scaled_payout += mint_amount as i128 * game_data.scaled_rewards_per_token as i128;
    }

    // Left intentionally in-efficient to make multiple clicks a similar compute cost to a single one
    for _ in 0..clicks {
        // Calculate SOL cost
        let click_cost  = game_data.rules.cost_per_click;
        game_data.rules.cost_per_click += 1;
        total_cost += click_cost;

        let dev_amount = click_cost / rules.dev_fee;
        let player_amount = click_cost - dev_amount;

        let mint_amount = rules.tokens_on_loss;

        total_mint += mint_amount;

        // Update token accounting
        game_data.total_staked += mint_amount;
        game_data.total_clicks += 1;
        player_data.staked += mint_amount;
        player_data.scaled_payout += mint_amount as i128 * game_data.scaled_rewards_per_token as i128;
        player_data.clicks += 1;

        // Disperse funds
        game_data.dev_fees += dev_amount;
        game_data.scaled_rewards_per_token += player_amount as u128 * FLOAT_SCALAR as u128 / game_data.total_staked as u128;
    }

    // Transfer SOL to game account
    transfer_lamports(&ctx.accounts.player, &ctx.accounts.game_data.to_account_info(), &ctx.accounts.system_program, total_cost)?;

    // Mint cookies directly to game account
    let authority_key_bytes = ctx.accounts.authority.key().to_bytes();
    let signer_seeds: &[&[&[u8]]] = &[&[GameAccount::SEED, authority_key_bytes.as_slice(), &[ctx.bumps.game_data]]];
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.game_token_account.to_account_info(),
                authority: mint_authority.clone(),
            },
            signer_seeds,
        ),
        total_mint,
    )?;

    emit!(ClickEvent {
        player: player_key,
        clicks: clicks,
        cost: total_cost,
    });

    // Store data for jackpot
    player_data.last_clicks = clicks;
    player_data.last_slot = Clock::get()?.slot;

    Ok(())
}

#[derive(Accounts)]
pub struct Click<'info> {
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

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Address constraint in account trait
    #[account(address = sysvar::slot_hashes::id())]
    slot_hashes: UncheckedAccount<'info>,

    /// CHECK: Address constraint in account trait
    #[account(address = sysvar::instructions::id())]
    instructions: UncheckedAccount<'info>,

    system_program: Program<'info, System>,
}

#[event]
struct ClickEvent {
    player: Pubkey,
    clicks: u64,
    cost: u64,
}
