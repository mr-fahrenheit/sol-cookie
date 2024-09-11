use anchor_lang::{prelude::*, solana_program::{clock::Slot, keccak, program_memory::sol_memcmp, pubkey::PUBKEY_BYTES}, system_program};
use arrayref::array_ref;

use crate::error::CookieError;

// Global constants for game
pub const FLOAT_SCALAR: u128 = u128::pow(2, 48); // 2**48

// Transfer lamports between accounts via CPI call to system program
pub fn transfer_lamports<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    system_program: &Program<'a, System>,
    lamports: u64,
) -> Result<()> {
    let cpi_accounts = system_program::Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
    };
    let cpi_program = system_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    system_program::transfer(cpi_context, lamports)?;
    Ok(())
}

// Transfer lamports from an owned PDA to another account
pub fn transfer_lamports_from_owned_pda<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
) -> Result<()> {

    **from.try_borrow_mut_lamports()? -= lamports;
    **to.try_borrow_mut_lamports()? += lamports;

    Ok(())
}

pub fn calculate_rewards(scaled_rewards_per_token: u128, player_staked: u64, player_scaled_payout: i128) -> u64 {
    let total_rewards = scaled_rewards_per_token * player_staked as u128;

    if player_scaled_payout > 0 {
        // Safe to cast positive i128 to u128
        let player_scaled_payout_unsigned = player_scaled_payout as u128;
        return if player_scaled_payout_unsigned > total_rewards {
            0
        } else {
            ((total_rewards - player_scaled_payout_unsigned) / FLOAT_SCALAR) as u64
        };
    } else {
        // Safe to cast negated i128 value to u128
        let player_scaled_payout_unsigned = -player_scaled_payout as u128;
        ((total_rewards + player_scaled_payout_unsigned) / FLOAT_SCALAR) as u64
    }
}

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn resolve_jackpot<'a>(slot_hashes: &UncheckedAccount<'a>, player: Pubkey, last_slot: Slot, last_clicks: u64, win_chance: u128) -> Result<bool> {
    if last_slot == 0 {
        // Ignore on first click
        return Ok(false)
    }

    // Load data from SlotHashes account
    let data = slot_hashes.data.borrow();

    // Get length of slot hash vector (maxes at 512 on mainnet)
    let bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let len = u64::from_le_bytes(bytes);

    let current_slot = Clock::get()?.slot;

    if current_slot == last_slot {
        // Ignore 2 click txs in the same slot
        Ok(false)
    } else {
        // Calculate how far into the vec the desired hash is
        let slot_offset = current_slot - last_slot - 1;

        if slot_offset > len {
            // Took too long to resolve (>512 slots)
            Ok(false)
        } else {
            // items.length (8 bytes) | items[0].slot (8 bytes) | items[0].hash (32 bytes) | items[1].slot (8 bytes) | items[1].hash (32 bytes)
            let hash_offset: usize = (8 + slot_offset * 40 + 8) as usize;
            let bytes: [u8; 32] = data[hash_offset..hash_offset + 32].try_into().unwrap();

            // Hash (slothash ++ player)
            let hash = keccak::hashv(
                &[
                    bytes.as_slice(),
                    player.to_bytes().as_slice(),
                ]
            ).to_bytes();

            // Grab first 16 bytes and convert to little endian u128
            let seed_data = array_ref![hash, 0, 16];
            let seed = u128::from_le_bytes(*seed_data);

            // Wrap around 1e18
            let rng: u128 = seed
                .checked_rem(1_000_000_000_000_000_000)
                .ok_or(CookieError::NumericalOverflow)? as u128;

            Ok(rng < win_chance * last_clicks as u128)
        }
    }
}