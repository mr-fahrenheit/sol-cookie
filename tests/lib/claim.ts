import type { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import { getRewardsFor } from './common';
import { expect } from 'chai';

export async function claimRewards(program: Program<Cookie>, authority: PublicKey, player: Keypair) {
    const connection = program.provider.connection;

    const solBalanceBefore = await connection.getBalance(player.publicKey);
    const expectedRewards = (await getRewardsFor(program, authority, player.publicKey)).toNumber();

    await program.methods
        .claimRewards()
        .accounts({
            authority: authority,
            player: player.publicKey,
        })
        .signers([player])
        .rpc({commitment: 'confirmed'});

    const solBalanceAfter = await connection.getBalance(player.publicKey);

    const solDiff = solBalanceAfter - solBalanceBefore;

    expect(solDiff).to.be.equal(expectedRewards);
}
