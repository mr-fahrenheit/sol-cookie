import type { Program } from '@coral-xyz/anchor';
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair } from '@solana/web3.js';
import { Cookie } from '../../target/types/cookie';
import { click } from './click';
import { resolve } from './resolve';
import { claimRewards } from './claim';
import { unstake, unstakeAll } from './unstake';
import { stake, stakeAll } from './stake';
import { withdraw, withdrawAndExpectFees } from './withdraw';
import { getGamePDA, getPlayerData, getTokenBalance, sendTokensTo } from './common';
import { expect } from 'chai';

// Wrapper around game functionality for test execution
export class Game {
    private connection: anchor.web3.Connection; 
    private initialBalance: number;

    constructor(private program: Program<Cookie>, public authority: Keypair, public mint: PublicKey) {
        this.connection = program.provider.connection;
    }

    async setInitialBalance() {
        this.initialBalance = await this.getGameBalance();
    }

    async expectBalance(expected: number, delta: number = 10) {
        const balance = await this.getGameBalance();
        expect(balance - this.initialBalance).to.be.approximately(expected, delta);
    }

    async click(player: Keypair, clicks: number) {
        return click(this.program, this.authority.publicKey, this.mint, player, clicks);
    }

    async resolve(player: Keypair) {
        return resolve(this.program, this.authority.publicKey, this.mint, player);
    }

    async claimRewards(player: Keypair) {
        return claimRewards(this.program, this.authority.publicKey, player)
    }

    async unstake(player: Keypair, amount: number) {
        return unstake(this.program, this.authority.publicKey, player, this.mint, amount);
    }

    async unstakeAll(player: Keypair) {
        return unstakeAll(this.program, this.authority.publicKey, player, this.mint);
    }

    async stake(player: Keypair, amount: number) {
        return stake(this.program, this.authority.publicKey, player, this.mint, amount);
    }

    async stakeAll(player: Keypair) {
        return stakeAll(this.program, this.authority.publicKey, player, this.mint);
    }

    async withdraw() {
        return withdraw(this.program, this.authority);
    }

    async withdrawAndExpectFees(expectedFees: number) {
        return withdrawAndExpectFees(this.program, this.authority, expectedFees);
    }

    async getTokenBalance(player: PublicKey): Promise<number> {
        return getTokenBalance(this.connection, this.mint, player);
    }

    async sendTokensTo(from: Keypair, to: PublicKey, amount: number): Promise<string> {
        return sendTokensTo(this.connection, this.mint, from, to, amount);
    }

    async getPlayerData(player: PublicKey) {
        return getPlayerData(this.program, this.authority.publicKey, player);
    }

    async getGameBalance(): Promise<number> {
        return await this.connection.getBalance(getGamePDA(this.program, this.authority.publicKey));
    }
}