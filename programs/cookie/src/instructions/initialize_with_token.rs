use std::borrow::BorrowMut;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}, token_interface::{mint_to, MintTo}};
use mpl_token_metadata::{
    instructions::CreateV1CpiBuilder,
    types::TokenStandard,
};

use crate::state::*;

pub fn initialize_with_token(ctx: Context<InitializeWithToken>, rules: GameRules, token_details: TokenDetails) -> Result<()> {
    // Grab account info we need for CPI call later
    let mint_authority_info = ctx.accounts.game_data.to_account_info();
    let mint_info = &ctx.accounts.token_mint.to_account_info();
    let sysvar_info = &ctx.accounts.rent.to_account_info();

    // Set initial game state and rules
    let game_data = (*ctx.accounts.game_data).borrow_mut();
    game_data.total_staked = 0;
    game_data.scaled_rewards_per_token = 0;
    game_data.dev_fees = 0;
    game_data.mint = ctx.accounts.token_mint.clone().key();
    game_data.authority = ctx.accounts.authority.key();
    game_data.rules = rules;

    // Construct the CPI context
    let mut cpi_context = CreateV1CpiBuilder::new(&ctx.accounts.token_metadata_program);
    let create_cpi = cpi_context
        .metadata(&ctx.accounts.metadata_account)
        .mint(&mint_info, false)
        .authority(&mint_authority_info)
        .payer(&ctx.accounts.authority)
        .update_authority(&ctx.accounts.authority, false)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&sysvar_info)
        .token_standard(TokenStandard::Fungible)
        .name(token_details.name)
        .is_mutable(false)
        .uri(token_details.uri)
        .symbol(token_details.symbol)
        .seller_fee_basis_points(0);

    // Construct signer seeds
    let authority_key_bytes = ctx.accounts.authority.key().to_bytes();
    let signer_seeds: &[&[&[u8]]] = &[&[GameAccount::SEED, authority_key_bytes.as_slice(), &[ctx.bumps.game_data]]];

    // Invoke
    create_cpi.invoke_signed(signer_seeds)?;

    // Mint initial supply
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.authority_token_account.to_account_info(),
                authority: ctx.accounts.game_data.to_account_info(),
            },
            signer_seeds,
        ),
        token_details.initial_supply,
    )?;

    Ok(())
}


#[derive(Accounts)]
pub struct InitializeWithToken<'info> {
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

    #[account(
        init,
        seeds = [b"mint", authority.key().as_ref()], 
        bump,
        payer = authority,
        mint::decimals = 6,
        mint::authority = game_data.key(),
        owner = token_program.key(),
    )]
    token_mint: Account<'info, Mint>,

    #[account(
        init, 
        payer = authority, 
        associated_token::mint = token_mint, 
        associated_token::authority = authority,
    )]
    authority_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Metaplex will check this
    #[account(mut)]
    metadata_account: UncheckedAccount<'info>,

    rent: Sysvar<'info, Rent>,

    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Metaplex will check this
    token_metadata_program: UncheckedAccount<'info>,

    system_program: Program<'info, System>,
}