import type { Program } from '@coral-xyz/anchor';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, Keypair, ComputeBudgetProgram } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import { expect } from 'chai';
import { getGamePDA, DEFAULT_RULES, getPlayerData, getGameData } from './common';
import { getAssociatedTokenAddress } from '@solana/spl-token';

export async function click(program: Program<Cookie>, authority: PublicKey, mint: PublicKey, player: Keypair, clicks: number) {
    const connection = program.provider.connection;

    const gamePDA = getGamePDA(program, authority);
    const gameData = await getGameData(program, authority);

    const solBalanceBefore = await connection.getBalance(gamePDA);
    const clicksBefore = (await getPlayerData(program, authority, player.publicKey)).clicks.toNumber();

    await program.methods
        .click(new anchor.BN(clicks))
        .accounts({
            authority: authority,
            player: player.publicKey,
            tokenMint: mint,
        })
        .preInstructions([
            ComputeBudgetProgram.setComputeUnitLimit({
                units: 1400000,
            }),
        ])
        .signers([player])
        .rpc({commitment: 'confirmed'});

    const solBalanceAfter = await connection.getBalance(gamePDA);
    const clicksAfter = (await getPlayerData(program, authority, player.publicKey)).clicks.toNumber();

    const solDiff = solBalanceAfter - solBalanceBefore;
    const clicksDiff = clicksAfter - clicksBefore;

    const expectedCost = new anchor.BN(clicks)
        .mul(gameData.rules.costPerClick)
        .add(
            new anchor.BN(clicks - 1).mul(
                new anchor.BN(clicks)
            )
            .div(new anchor.BN(2))
        )
        .toNumber();

    expect(solDiff).to.be.equal(expectedCost);
    expect(clicksDiff).to.be.equal(clicks);
}