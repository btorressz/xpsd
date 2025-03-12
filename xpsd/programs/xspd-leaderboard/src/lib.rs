use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("6wwxv4ysXnAWFHVbzN6G8rJ3RiXqpg8q4L3dfcE1QgBT");

#[program]
pub mod xspd_leaderboard {
    use super::*;

    pub fn initialize<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Initialize<'info>>
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.admin = ctx.accounts.admin.key();
        global_state.last_reward_distribution = Clock::get()?.unix_timestamp;
        // Initialize the leaderboard with default (empty) entries.
        for i in 0..10 {
            global_state.leaderboard[i] = LeaderboardEntry::default();
        }
        Ok(())
    }

    pub fn register_trader<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, RegisterTrader<'info>>
    ) -> Result<()> {
        let trader_stats = &mut ctx.accounts.trader_stats;
        trader_stats.trader = ctx.accounts.trader.key();
        trader_stats.total_trades = 0;
        trader_stats.total_execution_time = 0;
        trader_stats.last_updated = Clock::get()?.unix_timestamp;
        trader_stats.last_reward_time = 0;
        trader_stats.failed_trades = 0;
        Ok(())
    }

    pub fn record_trade<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, RecordTrade<'info>>,
        execution_time: u64,
        trade_price: u64,
    ) -> Result<()> {
        let trader_stats = &mut ctx.accounts.trader_stats;
        let clock = Clock::get()?;
        // Enforce a cooldown period (30 seconds) between trades.
        if clock.unix_timestamp - trader_stats.last_updated < 30 {
            return Err(ErrorCode::CooldownPeriod.into());
        }
        // Validate the trade using an external oracle.
        let latest_price = get_latest_oracle_price(ctx.accounts.price_oracle.to_account_info())?;
        if (latest_price as i64 - trade_price as i64).abs() > (latest_price / 200) as i64 {
            return Err(ErrorCode::InvalidTrade.into());
        }
        trader_stats.total_trades = trader_stats
            .total_trades
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;
        trader_stats.total_execution_time = trader_stats
            .total_execution_time
            .checked_add(execution_time)
            .ok_or(ErrorCode::Overflow)?;
        trader_stats.last_updated = clock.unix_timestamp;
        // Update the on-chain leaderboard.
        update_leaderboard(&mut ctx.accounts.global_state.leaderboard, trader_stats);
        Ok(())
    }

    pub fn record_failed_trade<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, RecordTrade<'info>>
    ) -> Result<()> {
        let trader_stats = &mut ctx.accounts.trader_stats;
        trader_stats.failed_trades = trader_stats
            .failed_trades
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;
        if trader_stats.failed_trades >= 5 {
            return Err(ErrorCode::TooManyFailedTrades.into());
        }
        Ok(())
    }

    pub fn distribute_rewards<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DistributeRewards<'info>>
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        let clock = Clock::get()?;
        if clock.unix_timestamp - global_state.last_reward_distribution < 3600 {
            return Err(ErrorCode::TooSoon.into());
        }
        // Calculate total trades and determine the reward amount.
        let total_trades: u64 = global_state
            .leaderboard
            .iter()
            .map(|entry| entry.total_trades)
            .sum();
        let base_reward: u64 = 100 * 1_000_000_000; // 100 tokens (with 9 decimals)
        let dynamic_reward = if total_trades >= 500 {
            (total_trades / 500) * base_reward
        } else {
            base_reward
        };

        // Clone treasury, admin, and token_program AccountInfos to decouple their lifetimes.
        let treasury_info = ctx.accounts.treasury.to_account_info().clone();
        let admin_info = ctx.accounts.admin.to_account_info().clone();
        let token_program_info = ctx.accounts.token_program.to_account_info().clone();

        // For each leaderboard entry, find the matching trader token account and transfer rewards.
        for entry in global_state.leaderboard.iter() {
            if entry.trader != Pubkey::default() {
                if let Some(trader_token_account_info) = ctx
                    .remaining_accounts
                    .iter()
                    .find(|acc| acc.key == &entry.trader)
                {
                    let cpi_accounts = Transfer {
                        from: treasury_info.clone(),
                        to: trader_token_account_info.clone(),
                        authority: admin_info.clone(),
                    };
                    let cpi_ctx = CpiContext::new(token_program_info.clone(), cpi_accounts);
                    token::transfer(cpi_ctx, dynamic_reward)?;
                }
            }
        }
        global_state.last_reward_distribution = clock.unix_timestamp;
        Ok(())
    }

    pub fn stake_tokens<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, StakeTokens<'info>>,
        amount: u64,
    ) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;
        stake_info.staked_amount = stake_info
            .staked_amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        let cpi_accounts = Transfer {
            from: ctx.accounts.trader_token_account.to_account_info(),
            to: ctx.accounts.staking_pool.to_account_info(),
            authority: ctx.accounts.trader.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw_stake<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, WithdrawStake<'info>>,
        amount: u64,
    ) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;
        if stake_info.staked_amount < amount {
            return Err(ErrorCode::InsufficientStake.into());
        }
        stake_info.staked_amount = stake_info
            .staked_amount
            .checked_sub(amount)
            .ok_or(ErrorCode::Overflow)?;
        let cpi_accounts = Transfer {
            from: ctx.accounts.staking_pool.to_account_info(),
            to: ctx.accounts.trader_token_account.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn claim_rewards<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ClaimRewards<'info>>
    ) -> Result<()> {
        let trader_stats = &mut ctx.accounts.trader_stats;
        if trader_stats.total_trades == 0 {
            return Err(ErrorCode::NoEligibleRewards.into());
        }
        let reward_amount = trader_stats
            .total_trades
            .checked_mul(10_000_000)
            .ok_or(ErrorCode::Overflow)?;
        // Reset the trader's stats after claiming rewards.
        trader_stats.total_trades = 0;
        trader_stats.total_execution_time = 0;
        trader_stats.failed_trades = 0;
        let cpi_accounts = Transfer {
            from: ctx.accounts.treasury.to_account_info(),
            to: ctx.accounts.trader_token_account.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, reward_amount)?;
        Ok(())
    }
}

fn get_latest_oracle_price(_oracle_account: AccountInfo) -> Result<u64> {
    // Stubbed oracle price – replace with a real oracle integration.
    Ok(100)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + GlobalState::SIZE)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterTrader<'info> {
    #[account(
        init,
        payer = trader,
        space = 8 + TraderStats::SIZE,
        seeds = [b"trader_stats", trader.key().as_ref()],
        bump
    )]
    pub trader_stats: Account<'info, TraderStats>,
    #[account(mut)]
    pub trader: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordTrade<'info> {
    #[account(mut, seeds = [b"trader_stats", trader.key().as_ref()], bump)]
    pub trader_stats: Account<'info, TraderStats>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub trader: Signer<'info>,
    /// CHECK: This oracle account is unchecked; its data is validated within the program.
    pub price_oracle: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut, has_one = admin)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,
    // Trader token accounts for reward transfers are provided as remaining_accounts.
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub stake_info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub trader: Signer<'info>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub trader_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    pub stake_info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub trader: Signer<'info>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub trader_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut, seeds = [b"trader_stats", trader.key().as_ref()], bump)]
    pub trader_stats: Account<'info, TraderStats>,
    #[account(mut)]
    pub trader: Signer<'info>,
    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,
    #[account(mut)]
    pub trader_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub last_reward_distribution: i64,
    pub leaderboard: [LeaderboardEntry; 10],
}

impl GlobalState {
    const SIZE: usize = 32 + 8 + (LeaderboardEntry::SIZE * 10);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LeaderboardEntry {
    pub trader: Pubkey,
    pub total_trades: u64,
    pub total_execution_time: u64,
}

impl LeaderboardEntry {
    const SIZE: usize = 32 + 8 + 8;
}

#[account]
pub struct TraderStats {
    pub trader: Pubkey,
    pub total_trades: u64,
    pub total_execution_time: u64,
    pub last_updated: i64,
    pub last_reward_time: i64,
    pub failed_trades: u64,
}

impl TraderStats {
    const SIZE: usize = 32 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct StakeInfo {
    pub trader: Pubkey,
    pub staked_amount: u64,
}

impl StakeInfo {
    const SIZE: usize = 32 + 8;
}

/// Updates the leaderboard by inserting or updating the trader's stats.
fn update_leaderboard(leaderboard: &mut [LeaderboardEntry; 10], trader_stats: &TraderStats) {
    if trader_stats.total_trades == 0 {
        return;
    }
    let avg_time = trader_stats.total_execution_time as f64 / trader_stats.total_trades as f64;
    if let Some(pos) = leaderboard.iter().position(|entry| entry.trader == trader_stats.trader) {
        leaderboard[pos].total_trades = trader_stats.total_trades;
        leaderboard[pos].total_execution_time = trader_stats.total_execution_time;
    } else {
        if let Some(pos) = leaderboard.iter().position(|entry| entry.trader == Pubkey::default()) {
            leaderboard[pos] = LeaderboardEntry {
                trader: trader_stats.trader,
                total_trades: trader_stats.total_trades,
                total_execution_time: trader_stats.total_execution_time,
            };
        } else {
            let mut worst_index = 0;
            let mut worst_avg = 0.0;
            for (i, entry) in leaderboard.iter().enumerate() {
                if entry.total_trades > 0 {
                    let entry_avg = entry.total_execution_time as f64 / entry.total_trades as f64;
                    if entry_avg > worst_avg {
                        worst_avg = entry_avg;
                        worst_index = i;
                    }
                }
            }
            if avg_time < worst_avg {
                leaderboard[worst_index] = LeaderboardEntry {
                    trader: trader_stats.trader,
                    total_trades: trader_stats.total_trades,
                    total_execution_time: trader_stats.total_execution_time,
                };
            }
        }
    }
    leaderboard.sort_by(|a, b| {
        let a_avg = if a.total_trades > 0 {
            a.total_execution_time as f64 / a.total_trades as f64
        } else {
            f64::MAX
        };
        let b_avg = if b.total_trades > 0 {
            b.total_execution_time as f64 / b.total_trades as f64
        } else {
            f64::MAX
        };
        a_avg.partial_cmp(&b_avg).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow occurred.")]
    Overflow,
    #[msg("Not enough time has passed since the last reward distribution.")]
    TooSoon,
    #[msg("Insufficient staked amount.")]
    InsufficientStake,
    #[msg("Cooldown period has not passed yet.")]
    CooldownPeriod,
    #[msg("Trade price does not match oracle.")]
    InvalidTrade,
    #[msg("Too many failed trades. Please slow down.")]
    TooManyFailedTrades,
    #[msg("No eligible rewards to claim.")]
    NoEligibleRewards,
}
