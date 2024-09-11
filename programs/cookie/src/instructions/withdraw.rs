use std::borrow::BorrowMut;
use anchor_lang::prelude::*;

use crate::{helpers::*, state::*};

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    // Grab data from accounts
    let game_balance = ctx.accounts.game_data.get_lamports();
    let game_data = ctx.accounts.game_data.borrow_mut();

    // Calculate amount to send and update state
    let mut amount = game_data.dev_fees;
    if amount > game_balance {
        amount = game_balance;
    }

    // Split into thirds
    let amount_per = amount / 3;
    amount = amount_per * 3;

    // Accounting
    game_data.dev_fees -= amount;

    // Transfer SOL to devs
    transfer_lamports_from_owned_pda(&ctx.accounts.game_data.to_account_info(), &ctx.accounts.dev_1.to_account_info(), amount_per)?;
    transfer_lamports_from_owned_pda(&ctx.accounts.game_data.to_account_info(), &ctx.accounts.dev_2.to_account_info(), amount_per)?;
    transfer_lamports_from_owned_pda(&ctx.accounts.game_data.to_account_info(), &ctx.accounts.dev_3.to_account_info(), amount_per)?;

    emit!(WithdrawEvent {
        amount: amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    authority: Signer<'info>,

    #[account(mut, seeds = [GameAccount::SEED, authority.key().as_ref()], bump)]
    game_data: Account<'info, GameAccount>,

    /// CHECK: Address constraint in account trait
    #[account(mut, address = game_data.rules.dev_1)]
    dev_1: UncheckedAccount<'info>,

    /// CHECK: Address constraint in account trait
    #[account(mut, address = game_data.rules.dev_2)]
    dev_2: UncheckedAccount<'info>,

    /// CHECK: Address constraint in account trait
    #[account(mut, address = game_data.rules.dev_3)]
    dev_3: UncheckedAccount<'info>,

    system_program: Program<'info, System>,
}


#[event]
struct WithdrawEvent {
    amount: u64,
}