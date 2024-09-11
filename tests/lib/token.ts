import type { Program } from '@coral-xyz/anchor';
import { Cookie } from '../../target/types/cookie';
import { Keypair, SystemProgram, Transaction, sendAndConfirmTransaction } from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createInitializeMintInstruction,
  getMintLen,
} from '@solana/spl-token';
import { getGamePDA } from './common';

export async function createToken(program: Program<Cookie>, authority: Keypair, mint: Keypair) {
    const connection = program.provider.connection;
    const extensions = [];
    const decimals = 6;
    const mintLen = getMintLen(extensions);
    const lamports = await connection.getMinimumBalanceForRentExemption(mintLen);

    const gamePDA = getGamePDA(program, authority.publicKey);

    const transaction = new Transaction().add(
        SystemProgram.createAccount({
            fromPubkey: authority.publicKey,
            newAccountPubkey: mint.publicKey,
            space: mintLen,
            lamports: lamports,
            programId: TOKEN_PROGRAM_ID,
        }),
        createInitializeMintInstruction(mint.publicKey, decimals, gamePDA, null),
    );

    await sendAndConfirmTransaction(connection, transaction, [authority, mint], {
        skipPreflight: true,
        commitment: 'confirmed',
    });
}
