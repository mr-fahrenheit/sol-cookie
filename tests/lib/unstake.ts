import type { Program } from '@coral-xyz/anchor';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import { getGamePDA, getPlayerData, getRewardsFor, getTokenBalance } from './common';
import { expect } from 'chai';

export async function unstake(program: Program<Cookie>, authority: PublicKey, player: Keypair, mint: PublicKey, amount: number) {
    const connection = program.provider.connection;
    const gamePDA = getGamePDA(program, authority);

    const playerBalanceBefore = await getTokenBalance(connection, mint, player.publicKey);
    const gameBalanceBefore = await getTokenBalance(connection, mint, gamePDA);
    const rewardsBefore = (await getRewardsFor(program, authority, player.publicKey)).toNumber();
    const playerDataBefore = await getPlayerData(program, authority, player.publicKey);

    await program.methods
        .unstake(new anchor.BN(amount))
        .accounts({
            authority: authority,
            tokenMint: mint,
            player: player.publicKey,
        })
        .signers([player])
        .rpc({commitment: 'confirmed'});

    const playerBalanceAfter = await getTokenBalance(connection, mint, player.publicKey);
    const gameBalanceAfter = await getTokenBalance(connection, mint, gamePDA);
    const rewardsAfter = (await getRewardsFor(program, authority, player.publicKey)).toNumber();
    const playerDataAfter = await getPlayerData(program, authority, player.publicKey);

    const playerDiff = playerBalanceAfter - playerBalanceBefore;
    const gameDiff = gameBalanceAfter - gameBalanceBefore;
    const rewardsDiff = rewardsAfter - rewardsBefore;
    const stakedDiff = playerDataAfter.staked.sub(playerDataBefore.staked).toNumber();

    expect(gameDiff).to.be.equal(-playerDiff);
    expect(playerDiff).to.be.equal(amount);
    expect(rewardsDiff).to.be.equal(0);
    expect(stakedDiff).to.be.equal(-amount);
}

export async function unstakeAll(program: Program<Cookie>, authority: PublicKey, player: Keypair, mint: PublicKey) {
    const connection = program.provider.connection;
    const gamePDA = getGamePDA(program, authority);

    const playerBalanceBefore = await getTokenBalance(connection, mint, player.publicKey);
    const gameBalanceBefore = await getTokenBalance(connection, mint, gamePDA);
    const rewardsBefore = (await getRewardsFor(program, authority, player.publicKey)).toNumber();

    await program.methods
        .unstakeAll()
        .accounts({
            authority: authority,
            tokenMint: mint,
            player: player.publicKey,
        })
        .signers([player])
        .rpc({commitment: 'confirmed'});

    const playerBalanceAfter = await getTokenBalance(connection, mint, player.publicKey);
    const gameBalanceAfter = await getTokenBalance(connection, mint, gamePDA);
    const rewardsAfter = (await getRewardsFor(program, authority, player.publicKey)).toNumber();
    const playerDataAfter = await getPlayerData(program, authority, player.publicKey);

    const playerDiff = playerBalanceAfter - playerBalanceBefore;
    const gameDiff = gameBalanceAfter - gameBalanceBefore;
    const rewardsDiff = rewardsAfter - rewardsBefore;

    expect(gameDiff).to.be.equal(-playerDiff);
    expect(rewardsDiff).to.be.equal(0);
    expect(playerDataAfter.staked.toNumber()).to.be.equal(0);
}