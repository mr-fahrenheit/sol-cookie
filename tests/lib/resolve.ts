import type { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';

export async function resolve(program: Program<Cookie>, authority: PublicKey, mint: PublicKey, player: Keypair) {
    await program.methods
        .resolve()
        .accounts({
            authority: authority,
            player: player.publicKey,
            tokenMint: mint,
        })
        .signers([player])
        .rpc({commitment: 'confirmed'});
}