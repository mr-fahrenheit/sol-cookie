import type { Program } from '@coral-xyz/anchor';
import { Keypair } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import { getGameData } from './common';
import { expect } from 'chai';

export async function withdraw(program: Program<Cookie>, authority: Keypair) {
    const gameData = await getGameData(program, authority.publicKey);
    const expectedFees = gameData.devFees.toNumber();
    await withdrawAndExpectFees(program, authority, expectedFees);
}

export async function withdrawAndExpectFees(program: Program<Cookie>, authority: Keypair, expectedFees: number) {
    const connection = program.provider.connection;

    // Fetch game data to get the dev addresses
    const gameData = await getGameData(program, authority.publicKey);
    const devs = [gameData.rules.dev1, gameData.rules.dev2, gameData.rules.dev3];

    const devBalancesBefore = await Promise.all(devs.map(dev => connection.getBalance(dev)));

    await program.methods
        .withdraw()
        .accounts({
            authority: authority.publicKey,
            dev1: gameData.rules.dev1,
            dev2: gameData.rules.dev2,
            dev3: gameData.rules.dev3,
        })
        .signers([authority])
        .rpc({commitment: 'confirmed'});

    const devBalancesAfter = await Promise.all(devs.map(dev => connection.getBalance(dev)));
    const diffs = devBalancesBefore.map((before, i) => devBalancesAfter[i] - before);
    diffs.map(diff => expect(diff).to.be.equal(Math.floor(expectedFees / 3)));
}
