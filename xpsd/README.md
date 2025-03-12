# XSPD Leaderboard

## Overview

The XSPD Leaderboard program is a Solana-based decentralized application that tracks and rewards traders based on their trading performance. The program maintains a leaderboard of the top traders and distributes rewards based on their trading activity.

## Features

- **Initialize**: Sets up the global state of the program, including the admin and the initial leaderboard.
- **Register Trader**: Registers a new trader and initializes their trading statistics.
- **Record Trade**: Records a successful trade for a trader, updating their statistics and the leaderboard.
- **Record Failed Trade**: Records a failed trade for a trader, incrementing their failed trade count.
- **Distribute Rewards**: Distributes rewards to the top traders based on their trading performance.
- **Stake Tokens**: Allows traders to stake tokens into a staking pool.
- **Withdraw Stake**: Allows traders to withdraw their staked tokens from the staking pool.
- **Claim Rewards**: Allows traders to claim their accumulated rewards based on their trading activity.

## Accounts

- **GlobalState**: Stores the global state of the program, including the admin, last reward distribution timestamp, and the leaderboard.
- **TraderStats**: Stores the trading statistics for a trader, including total trades, total execution time, last updated timestamp, last reward time, and failed trades.
- **StakeInfo**: Stores the staking information for a trader, including the staked amount.

## Error Codes

- **Overflow**: Indicates an arithmetic overflow occurred.
- **TooSoon**: Indicates that not enough time has passed since the last reward distribution.
- **InsufficientStake**: Indicates that the trader does not have enough staked tokens to perform the action.
- **CooldownPeriod**: Indicates that the cooldown period between trades has not passed yet.
- **InvalidTrade**: Indicates that the trade price does not match the oracle price.
- **TooManyFailedTrades**: Indicates that the trader has too many failed trades.
- **NoEligibleRewards**: Indicates that the trader has no eligible rewards to claim.

## Usage

1. **Initialize**: Call the `initialize` function to set up the global state.
2. **Register Trader**: Call the `register_trader` function to register a new trader.
3. **Record Trade**: Call the `record_trade` function to record a successful trade.
4. **Record Failed Trade**: Call the `record_failed_trade` function to record a failed trade.
5. **Distribute Rewards**: Call the `distribute_rewards` function to distribute rewards to the top traders.
6. **Stake Tokens**: Call the `stake_tokens` function to stake tokens into the staking pool.
7. **Withdraw Stake**: Call the `withdraw_stake` function to withdraw staked tokens from the staking pool.
8. **Claim Rewards**: Call the `claim_rewards` function to claim accumulated rewards.

## Testing

The program includes a set of tests to verify its functionality. The tests cover the initialization, registration, trade recording, reward distribution, staking, and reward claiming functionalities.

## Conclusion

The XSPD Leaderboard program provides a decentralized way to track and reward traders based on their performance. By leveraging the Solana blockchain, it ensures transparency and security in the reward distribution process.
