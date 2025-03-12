//File was exported from Solana Playground and converted for VSCde
//TODO: view and edit test file
import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { XspdLeaderboard } from "../target/types/xspd_leaderboard";

describe("xspd_leaderboard", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.XspdLeaderboard as anchor.Program<XspdLeaderboard>;

  it("initialize", async () => {
    const admin = anchor.web3.Keypair.generate();
    const globalState = anchor.web3.Keypair.generate();

    await program.methods
      .initialize()
      .accounts({
        globalState: globalState.publicKey,
        admin: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([admin, globalState])
      .rpc();

    const state = await program.account.globalState.fetch(globalState.publicKey);
    assert.ok(state.admin.equals(admin.publicKey));
  });

  it("register_trader", async () => {
    const trader = anchor.web3.Keypair.generate();
    const traderStats = anchor.web3.Keypair.generate();

    await program.methods
      .registerTrader()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader, traderStats])
      .rpc();

    const stats = await program.account.traderStats.fetch(traderStats.publicKey);
    assert.ok(stats.trader.equals(trader.publicKey));
  });

  it("record_trade", async () => {
    const trader = anchor.web3.Keypair.generate();
    const traderStats = anchor.web3.Keypair.generate();
    const globalState = anchor.web3.Keypair.generate();
    const priceOracle = anchor.web3.Keypair.generate();

    await program.methods
      .registerTrader()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader, traderStats])
      .rpc();

    await program.methods
      .recordTrade(new BN(100), new BN(100))
      .accounts({
        traderStats: traderStats.publicKey,
        globalState: globalState.publicKey,
        trader: trader.publicKey,
        priceOracle: priceOracle.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader])
      .rpc();

    const stats = await program.account.traderStats.fetch(traderStats.publicKey);
    assert.ok(stats.totalTrades.eq(new BN(1)));
  });

  it("record_failed_trade", async () => {
    const trader = anchor.web3.Keypair.generate();
    const traderStats = anchor.web3.Keypair.generate();

    await program.methods
      .registerTrader()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader, traderStats])
      .rpc();

    await program.methods
      .recordFailedTrade()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader])
      .rpc();

    const stats = await program.account.traderStats.fetch(traderStats.publicKey);
    assert.ok(stats.failedTrades.eq(new BN(1)));
  });

  it("distribute_rewards", async () => {
    const admin = anchor.web3.Keypair.generate();
    const globalState = anchor.web3.Keypair.generate();
    const treasury = anchor.web3.Keypair.generate();
    const trader = anchor.web3.Keypair.generate();
    const traderTokenAccount = anchor.web3.Keypair.generate();

    await program.methods
      .initialize()
      .accounts({
        globalState: globalState.publicKey,
        admin: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([admin, globalState])
      .rpc();

    await program.methods
      .distributeRewards()
      .accounts({
        globalState: globalState.publicKey,
        admin: admin.publicKey,
        tokenProgram: anchor.web3.Token.programId,
        treasury: treasury.publicKey,
      })
      .remainingAccounts([
        {
          pubkey: traderTokenAccount.publicKey,
          isWritable: true,
          isSigner: false,
        },
      ])
      .signers([admin])
      .rpc();

    const state = await program.account.globalState.fetch(globalState.publicKey);
    assert.ok(state.lastRewardDistribution > 0);
  });

  it("stake_tokens", async () => {
    const trader = anchor.web3.Keypair.generate();
    const stakeInfo = anchor.web3.Keypair.generate();
    const traderTokenAccount = anchor.web3.Keypair.generate();
    const stakingPool = anchor.web3.Keypair.generate();

    await program.methods
      .stakeTokens(new BN(100))
      .accounts({
        stakeInfo: stakeInfo.publicKey,
        trader: trader.publicKey,
        tokenProgram: anchor.web3.Token.programId,
        traderTokenAccount: traderTokenAccount.publicKey,
        stakingPool: stakingPool.publicKey,
      })
      .signers([trader, stakeInfo])
      .rpc();

    const stake = await program.account.stakeInfo.fetch(stakeInfo.publicKey);
    assert.ok(stake.stakedAmount.eq(new BN(100)));
  });

  it("withdraw_stake", async () => {
    const trader = anchor.web3.Keypair.generate();
    const stakeInfo = anchor.web3.Keypair.generate();
    const traderTokenAccount = anchor.web3.Keypair.generate();
    const stakingPool = anchor.web3.Keypair.generate();
    const admin = anchor.web3.Keypair.generate();

    await program.methods
      .stakeTokens(new BN(100))
      .accounts({
        stakeInfo: stakeInfo.publicKey,
        trader: trader.publicKey,
        tokenProgram: anchor.web3.Token.programId,
        traderTokenAccount: traderTokenAccount.publicKey,
        stakingPool: stakingPool.publicKey,
      })
      .signers([trader, stakeInfo])
      .rpc();

    await program.methods
      .withdrawStake(new BN(50))
      .accounts({
        stakeInfo: stakeInfo.publicKey,
        trader: trader.publicKey,
        tokenProgram: anchor.web3.Token.programId,
        stakingPool: stakingPool.publicKey,
        traderTokenAccount: traderTokenAccount.publicKey,
        admin: admin.publicKey,
      })
      .signers([trader, admin])
      .rpc();

    const stake = await program.account.stakeInfo.fetch(stakeInfo.publicKey);
    assert.ok(stake.stakedAmount.eq(new BN(50)));
  });

  it("claim_rewards", async () => {
    const trader = anchor.web3.Keypair.generate();
    const traderStats = anchor.web3.Keypair.generate();
    const treasury = anchor.web3.Keypair.generate();
    const traderTokenAccount = anchor.web3.Keypair.generate();
    const admin = anchor.web3.Keypair.generate();

    await program.methods
      .registerTrader()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([trader, traderStats])
      .rpc();

    await program.methods
      .claimRewards()
      .accounts({
        traderStats: traderStats.publicKey,
        trader: trader.publicKey,
        treasury: treasury.publicKey,
        traderTokenAccount: traderTokenAccount.publicKey,
        admin: admin.publicKey,
        tokenProgram: anchor.web3.Token.programId,
      })
      .signers([trader, admin])
      .rpc();

    const stats = await program.account.traderStats.fetch(traderStats.publicKey);
    assert.ok(stats.totalTrades.eq(new BN(0)));
  });
});
