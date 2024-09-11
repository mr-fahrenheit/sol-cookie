import type { Program } from '@coral-xyz/anchor';
import * as anchor from '@coral-xyz/anchor';
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import {
    createTransferInstruction,
    getAssociatedTokenAddressSync,
    getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';

export const devs = [
    Keypair.generate(),
    Keypair.generate(),
    Keypair.generate(),
]

export const DEFAULT_RULES = {
    costPerClick: new anchor.BN(100_000),
    winChance: new anchor.BN(100_000_000_000_000),
    tokensOnWin: new anchor.BN(10_000_000_000_000),
    tokensOnLoss: new anchor.BN(1_000_000),
    devFee: new anchor.BN(10),
    dev1: devs[0].publicKey,
    dev2: devs[1].publicKey,
    dev3: devs[2].publicKey,
};

export function getGamePDA(program: Program<Cookie>, authority: PublicKey) {
    const [gamePDA] = PublicKey.findProgramAddressSync([Buffer.from('game'), authority.toBuffer()], program.programId);
    return gamePDA;
}

export function getPlayerPDA(program: Program<Cookie>, authority: PublicKey, player: PublicKey) {
    const [playerPDA] = PublicKey.findProgramAddressSync([Buffer.from('player'), authority.toBuffer(), player.toBuffer()], program.programId);
    return playerPDA;
}

export async function getPlayerData(program: Program<Cookie>, authority: PublicKey, player: PublicKey) {
    const playerPDA = getPlayerPDA(program, authority, player);
    try {
        return await program.account.playerAccount.fetch(playerPDA);
    } catch(e) {
        return {
            scaledPayout: new anchor.BN(0),
            staked: new anchor.BN(0),
            clicks: new anchor.BN(0),
        };
    }
}

export async function getGameData(program: Program<Cookie>, authority: PublicKey) {
    const gamePDA = getGamePDA(program, authority);
    return await program.account.gameAccount.fetch(gamePDA);
}

export async function airdrop(connection: anchor.web3.Connection, recipient: PublicKey, amount: number) {
    let token_airdrop = await connection.requestAirdrop(recipient, amount * LAMPORTS_PER_SOL);
    const latestBlockHash = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        signature: token_airdrop,
    });
}

export async function printTxLog(connection: anchor.web3.Connection, txSig: string) {
    const txDetails = await connection.getTransaction(txSig, {
        maxSupportedTransactionVersion: 0,
        commitment: 'confirmed',
    });
    console.log(txDetails.meta.logMessages);
}

export async function getTxFee(connection: anchor.web3.Connection, txSig: string) {
    const txDetails = await connection.getTransaction(txSig, {
        maxSupportedTransactionVersion: 0,
        commitment: 'confirmed',
    });
    return txDetails.meta.fee;
}

export async function getTokenBalance(connection: anchor.web3.Connection, mint: PublicKey, player: PublicKey): Promise<number> {
    
    const playerTokenAccount = getAssociatedTokenAddressSync(mint, player, true);
    try {
        const value = await connection.getTokenAccountBalance(playerTokenAccount);
        return Number(value.value.amount);
    } catch(e){ 
        return 0;
    }
}

export async function sendTokensTo(connection: anchor.web3.Connection, mint: PublicKey, from: Keypair, to: PublicKey, amount: number): Promise<string> {
    const fromTokenAccount = await getOrCreateAssociatedTokenAccount(connection, from, mint, from.publicKey);
    const toTokenAccount = await getOrCreateAssociatedTokenAccount(connection, from, mint, to);

    const tx = new anchor.web3.Transaction()
    .add(
        createTransferInstruction(
            fromTokenAccount.address,
            toTokenAccount.address,
            from.publicKey,
            amount
        )
    );

    return anchor.web3.sendAndConfirmTransaction(connection, tx, [from]);
}

export async function getRewardsFor(program: Program<Cookie>, authority: PublicKey, player: PublicKey): Promise<anchor.BN> {
    // Fetch account data
    const gamePDA = getGamePDA(program, authority);
    const gameData = await program.account.gameAccount.fetch(gamePDA);

    try {
        const playerData = await getPlayerData(program, authority, player);

        // Get inputs
        const staked = playerData.staked;
        const scaledRewardsPerToken = gameData.scaledRewardsPerToken;
        const scaledPayout = playerData.scaledPayout;
        const FLOAT_SCALAR = new anchor.BN(2**48);

        // Calculate and return
        return scaledRewardsPerToken.mul(staked).sub(scaledPayout).div(FLOAT_SCALAR);
    } catch(e) {
        // Player data not initialized, so 0 rewards
        return new anchor.BN(0);
    }
}
