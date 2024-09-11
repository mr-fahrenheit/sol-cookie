import type { Program } from '@coral-xyz/anchor';
import * as anchor from "@coral-xyz/anchor";
import { Cookie } from '../../target/types/cookie';
import { PublicKey, Keypair, Transaction } from '@solana/web3.js';
import { DEFAULT_RULES, getGameData, getGamePDA, getTokenBalance } from './common';
import { createAssociatedTokenAccountInstruction, getAssociatedTokenAddress } from '@solana/spl-token';

import { expect } from 'chai';
import { MPL_TOKEN_METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import { toWeb3JsPublicKey } from '@metaplex-foundation/umi-web3js-adapters';

export async function initializeGame(program: Program<Cookie>, authority: Keypair, mint: Keypair) {
    await program.methods
        .initialize(mint.publicKey, DEFAULT_RULES)
        .accounts({
            authority: authority.publicKey,
            tokenMint: mint.publicKey,
        },
        )
        .signers([authority])
        .rpc({ commitment: 'confirmed' });

    // Fetch game data
    const gameData = await getGameData(program, authority.publicKey);

    // Check initial game state
    expect(gameData.totalStaked).to.be.a.bignumber.that.equals('0');
    expect(gameData.scaledRewardsPerToken).to.be.a.bignumber.that.equals('0');
    expect(gameData.devFees).to.be.a.bignumber.that.equals('0');
    expect(gameData.mint.toString()).to.equal(mint.publicKey.toString());
    expect(gameData.authority.toString()).to.equal(authority.publicKey.toString());

    // Check game rules
    expect(gameData.rules.costPerClick).to.be.a.bignumber.that.equals(DEFAULT_RULES.costPerClick);
    expect(gameData.rules.winChance).to.be.a.bignumber.that.equals(DEFAULT_RULES.winChance);
    expect(gameData.rules.tokensOnWin).to.be.a.bignumber.that.equals(DEFAULT_RULES.tokensOnWin);
    expect(gameData.rules.tokensOnLoss).to.be.a.bignumber.that.equals(DEFAULT_RULES.tokensOnLoss);
    expect(gameData.rules.devFee).to.be.a.bignumber.that.equals(DEFAULT_RULES.devFee);
    expect(gameData.rules.dev1.toString()).to.be.equal(DEFAULT_RULES.dev1.toString());
    expect(gameData.rules.dev2.toString()).to.be.equal(DEFAULT_RULES.dev2.toString());
    expect(gameData.rules.dev3.toString()).to.be.equal(DEFAULT_RULES.dev3.toString());
}

export async function initializeGameWithToken(program: Program<Cookie>, authority: Keypair, tokenName: string, tokenSymbol: string, tokenUri: string, initialSupply: anchor.BN) {
    // Calculate PDA account addresses
    const [mintPDA] = PublicKey.findProgramAddressSync([Buffer.from('mint'), authority.publicKey.toBuffer()], program.programId);
    const [metadataPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID).toBuffer(),
          mintPDA.toBuffer(),
        ],
        toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
    );

    // Construct token details object
    const tokenDetails = {
        name: tokenName,
        symbol: tokenSymbol,
        uri: tokenUri,
        initialSupply: initialSupply,
    };

    const gamePDA = getGamePDA(program, authority.publicKey);
    const authorityTokenAddress = await getAssociatedTokenAddress(mintPDA, gamePDA, true);

    // Call initialize with token
    await program.methods
        .initializeWithToken(DEFAULT_RULES, tokenDetails)
        .accounts({
            authority: authority.publicKey,
            tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
            metadataAccount: metadataPDA,
        },
        )
        .signers([authority])
        .postInstructions([
            createAssociatedTokenAccountInstruction(
                authority.publicKey,
                authorityTokenAddress,
                gamePDA,
                mintPDA,
            )
        ])
        .rpc({commitment: 'confirmed'});

    // Fetch game data
    const gameData = await getGameData(program, authority.publicKey);
    const authorityTokenBalance = await getTokenBalance(program.provider.connection, gameData.mint, authority.publicKey);

    // Check initial game state
    expect(gameData.totalStaked).to.be.a.bignumber.that.equals('0');
    expect(gameData.scaledRewardsPerToken).to.be.a.bignumber.that.equals('0');
    expect(gameData.devFees).to.be.a.bignumber.that.equals('0');
    expect(gameData.authority.toString()).to.equal(authority.publicKey.toString());
    expect(authorityTokenBalance).to.be.equal(initialSupply.toNumber());

    // Check game rules
    expect(gameData.rules.costPerClick).to.be.a.bignumber.that.equals(DEFAULT_RULES.costPerClick);
    expect(gameData.rules.winChance).to.be.a.bignumber.that.equals(DEFAULT_RULES.winChance);
    expect(gameData.rules.tokensOnWin).to.be.a.bignumber.that.equals(DEFAULT_RULES.tokensOnWin);
    expect(gameData.rules.tokensOnLoss).to.be.a.bignumber.that.equals(DEFAULT_RULES.tokensOnLoss);
    expect(gameData.rules.devFee).to.be.a.bignumber.that.equals(DEFAULT_RULES.devFee);
    expect(gameData.rules.dev1.toString()).to.be.equal(gameData.rules.dev1.toString());
    expect(gameData.rules.dev2.toString()).to.be.equal(gameData.rules.dev2.toString());
    expect(gameData.rules.dev3.toString()).to.be.equal(gameData.rules.dev3.toString());
}