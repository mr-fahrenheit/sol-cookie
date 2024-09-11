import * as anchor from "@coral-xyz/anchor";
import type { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import chai, { expect } from "chai";
import chaiAsPromised from "chai-as-promised";
import chaiBn from "chai-bn";
import { Cookie } from "../target/types/cookie";
import { initializeGame, initializeGameWithToken } from "./lib/initialise";
import { createToken } from "./lib/token";
import { DEFAULT_RULES, airdrop, getGameData, getGamePDA } from "./lib/common";
import { Game } from "./lib/game";

chai.use(chaiAsPromised);
chai.use(chaiBn(anchor.BN));

describe("cookie", () => {
    //
    // SETUP
    //

    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Cookie as Program<Cookie>;
    const connection = provider.connection;
    const wallet = provider.wallet as anchor.Wallet;

    //
    // HELPERS
    //

    async function setupGame(): Promise<Game> {
        const authority = Keypair.generate();
        const mint = Keypair.generate();

        // Send the authority some SOL to pay for setup
        await airdrop(connection, authority.publicKey, 1);
        await airdrop(connection, DEFAULT_RULES.dev1, 1);
        await airdrop(connection, DEFAULT_RULES.dev2, 1);
        await airdrop(connection, DEFAULT_RULES.dev3, 1);

        await createToken(program, authority, mint);
        await initializeGame(program, authority, mint);

        const game = new Game(program, authority, mint.publicKey);
        await game.setInitialBalance();
        return game;
    }

    async function setupPlayers(count: number): Promise<Array<Keypair>> {
        let players = [];
        for (let i = 0; i < count; i++) {
            const player = Keypair.generate();
            await airdrop(connection, player.publicKey, 10);
            players.push(player);
        }
        return players;
    }

    //
    // TESTS
    //

    it("Basic test all features", async () => {
        const game = await setupGame();
        const [player] = await setupPlayers(1);

        // Test basic functions
        await game.click(player, 1);
        await game.resolve(player);
        await game.claimRewards(player)
        const playerData = await game.getPlayerData(player.publicKey);
        await game.unstake(player, playerData.staked.toNumber());
        const tokenBalance = await game.getTokenBalance(player.publicKey);
        await game.stake(player, tokenBalance)
        await game.withdraw();
    });

    it("Initialize with metadata token", async () => {
        const authority = Keypair.generate();
        await airdrop(connection, authority.publicKey, 1);
        await airdrop(connection, DEFAULT_RULES.dev1, 1);
        await airdrop(connection, DEFAULT_RULES.dev2, 1);
        await airdrop(connection, DEFAULT_RULES.dev3, 1);
        await initializeGameWithToken(program, authority, "TEST", "T", "http://TEST", new anchor.BN(300_000_000_000_000));

        const gameData = await getGameData(program, authority.publicKey);
        const mint = gameData.mint;

        const game = new Game(program, authority, mint);
        const [player] = await setupPlayers(1);

        // Test basic functions
        await game.click(player, 1);
        await game.resolve(player);
        await game.claimRewards(player)
        const playerData = await game.getPlayerData(player.publicKey);
        await game.unstake(player, playerData.staked.toNumber());
        const tokenBalance = await game.getTokenBalance(player.publicKey);
        await game.stake(player, tokenBalance)
        await game.withdraw();
    });

    it("Cannot initialize twice with same authority", async () => {
        const game = await setupGame();
        const mint = Keypair.generate();
        await expect(initializeGame(program, game.authority, mint)).to.eventually.be.rejected;
    });

    it("Cannot perform 0 clicks", async () => {
        const game = await setupGame();
        const [player] = await setupPlayers(1);
        await expect(game.click(player, 0)).to.eventually.be.rejected;
    });

    it("Test multiple players clicking and then withdrawing rewards and dev fees", async () => {
        const game = await setupGame();
        const players = await setupPlayers(5);

        // Click 2x for each player
        for (let i = 0; i < players.length; i++) {
            await game.click(players[i], 2);
        }

        // Claim rewards
        for (let i = 0; i < players.length; i++) {
            await game.claimRewards(players[i])
        }

        // Withdraw
        const expectedFees = players.length * DEFAULT_RULES.costPerClick.toNumber() * 2 / DEFAULT_RULES.devFee.toNumber();
        await game.withdrawAndExpectFees(expectedFees);

        // Game treasury should be (more or less) empty
        await game.expectBalance(0);
    });

    it("Test unstaking, transfering, staking and claiming", async () => {
        const game = await setupGame();
        const [bob, alice] = await setupPlayers(2);

        // Click 10x to farm some tokens
        await game.click(bob, 10);

        // Unstake all
        await game.unstakeAll(bob);

        // Transfer from player 1 to player 2
        const bobBalance = await game.getTokenBalance(bob.publicKey);
        expect(bobBalance).is.greaterThan(0);
        await game.sendTokensTo(bob, alice.publicKey, bobBalance);

        // Stake with alice
        await game.stake(alice, bobBalance);

        // Click some more times with bob
        await game.click(bob, 10);

        // Unstake all
        await game.unstakeAll(alice);
        await game.unstakeAll(bob);

        // Claim rewards
        await game.claimRewards(alice);
        await game.claimRewards(bob);

        // Withdraw
        await game.withdraw();

        // Game treasury should be (more or less) empty
        await game.expectBalance(0);
    });
});
