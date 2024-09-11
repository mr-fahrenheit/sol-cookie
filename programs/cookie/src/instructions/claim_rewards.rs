use std::borrow::BorrowMut;
use anchor_lang::prelude::*;

use crate::{helpers::*, state::*};

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    // Grab data from accounts
    let game_data = ctx.accounts.game_data.borrow_mut();
    let player_data = ctx.accounts.player_data.borrow_mut();

    // Calculate rewards
    let rewards = calculate_rewards(game_data.scaled_rewards_per_token, player_data.staked, player_data.scaled_payout);

    // Update accounting
    player_data.scaled_payout += rewards as i128 * FLOAT_SCALAR as i128;

    // Transfer SOL to player
    transfer_lamports_from_owned_pda(&ctx.accounts.game_data.to_account_info(), &ctx.accounts.player, rewards)?;

    emit!(ClaimRewardsEvent {
        player: ctx.accounts.player.key(),
        amount: rewards,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    player: Signer<'info>,

    /// CHECK: Address constraint in account trait
    #[account(address = game_data.authority)]
    authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [GameAccount::SEED, authority.key().as_ref()], bump)]
    game_data: Account<'info, GameAccount>,

    #[account(
        mut,
        seeds = [PlayerAccount::SEED, authority.key().as_ref(), player.key().as_ref()], 
        bump 
    )]
    player_data: Account<'info, PlayerAccount>,

    system_program: Program<'info, System>,
}


#[event]
struct ClaimRewardsEvent {
    player: Pubkey,
    amount: u64,
}