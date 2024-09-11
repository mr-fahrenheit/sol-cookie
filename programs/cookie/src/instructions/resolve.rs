use std::borrow::BorrowMut;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}, token_interface::{mint_to, MintTo}};
use anchor_lang::{prelude::*, solana_program::sysvar::{self}};

use crate::{helpers::*, state::*, JackpotEvent};

pub fn resolve(ctx: Context<Resolve>) -> Result<()> {
    let player_key = ctx.accounts.player.key();
    let rules = ctx.accounts.game_data.rules.clone();

    let player_data = ctx.accounts.player_data.borrow_mut();

    // Resolve jackpot
    if resolve_jackpot(&ctx.accounts.slot_hashes, ctx.accounts.player.key(), player_data.last_slot, player_data.last_clicks, rules.win_chance)? {
        let game_data = ctx.accounts.game_data.borrow_mut();

        emit!(JackpotEvent {
            player: player_key,
        });

        // Accounting
        let mint_amount = rules.tokens_on_win;
        game_data.total_staked += mint_amount;
        player_data.staked += mint_amount;
        player_data.scaled_payout += mint_amount as i128 * game_data.scaled_rewards_per_token as i128;

        // Mint cookies directly to game account
        let mint_authority = ctx.accounts.game_data.to_account_info();
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
            mint_amount,
        )?;
    }

    // Reset jackpot data
    player_data.last_clicks = 0;
    player_data.last_slot = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct Resolve<'info> {
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
