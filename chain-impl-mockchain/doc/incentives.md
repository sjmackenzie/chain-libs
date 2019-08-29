# Incentives

## Overview

The goal is to provide incentive to people to participate in the system : stake pools for their operating cost and their delegators for their individual stake contribution.

The `Reward Escrow` is an amount defined in the genesis block and monotonically decreases till empty.

A `Reward Escrow Deduction` is an amount deducted from the `Reward Escrow` at the end of each epoch and is used to fund `Epoch Reward`.  

The `Epoch Reward` is accumulated during, then emptied at the end of every epoch. The `Epoch Reward` is composed of two parts:
1. Transaction fees
2. A `Reward Escrow Deduction` described in the above paragraph

Once the epoch reward is known, the `Treasury` deducts a portion and the remaining amount is divided between pools proportional to successful slot creation rate. Each pool's share is further divided between pool owners and delegators proportional to their stake.

    ┏━━━━━━━━━━━━━┓
    ┃Reward Escrow┃
    ┗━━━━━━━━━━━━━┛                    ╭ Block        ╭
               │   ┏━━━━━━━━━━━━━┓     │ Creators     │  Stake
               ╞══>┃Epoch Rewards┃═╤═> │ For      ═╤═>│ Delegators
               │   ┗━━━━━━━━━━━━━┛ │   │ This      │  ╰
            Σ Fees                 │   ╰ Epoch     │  ╭
                                   │               ╰─>│ Pool Owners
                                   │   ┏━━━━━━━━┓     ╰
                                   ╰─>─┃Treasury┃
                                       ┗━━━━━━━━┛

## Reward collection

Rewards follow a tried and tested model of high rewards and low fees at the start of the system, then as the `Reward Escrow` depletes, fees take over the rewarding of stakepools.

### Reward Calculation

Genesis creators have full control over the initial `Reward Escrow` amount, block rates and whether they wish an exponential or linear formula:

* Linear formula: `calc = f - rratio * (#epoch / erate)`
* Halving formula: `calc = f * rratio ^ (#epoch / erate)`

where

* `f` is a constant representing the starting amount of the `Reward Escrow`. In the linear formula `f` will always be the value at block0.
* `rratio` is the reducing ratio and is bounded between 0.0 and 1.0.
  A further requirement is that this ratio is expressed in fractional form (e.g. 1/2), which allows calculation in integer form (see implementation details). It represents the `Reward Escrow` reducing factor.
* `erate` is the rate at which the `Reward Escrow` is reduced. e.g. erate=100 means that
  every 100 epochs, the calculation is reduce further.

The `Reward Escrow Deduction` is:

    reward_escrow_deduction = MIN(reward_escrow, MAX(0, calc))

The `Reward Escrow` amount is adjusted as such:

    reward_escrow -= reward_escrow_deduction

### Epoch Fees

This one is simply of the sum of all the usage fees usage collected since the
previous reward distribution. Typically all the block that are not empty will
contains fees related to certificates and transactions, that are just added
to the total fees collected so far at each block application.

## Reward distribution

Once the reward pot is composed, the treasury takes a cut on the total,
and then each pool get reward related by their stake in the system

    UPSCALE(x) = x * 10^9
    DOWNSCALE(x) = x / 10^9

    treasury_contribution = TREASURY_CONTRIBUTION(reward_total)
    pool_contribution = reward_total - treasury_contribution

    TREASURY += treasury_contribution

    unit_reward = UPSCALE(pool_contribution) / #blocks_epoch
    remaining = UPSCALE(pool_contribution) % #blocks_epoch

    for each pool
        pool.account = DOWNSCALE(unit_reward * #pool.blocks_created)

Any non null amount could be arbitrarily gifted further to the treasury, or
could be considered a bootstrap contribution toward the next epoch reward pot.

### Pool distribution

For each pool, we split each `pool.account` into a owner part and the stake reward part. Further:

    UPSCALE_STAKE(x) = x * 10^18
    DOWNSCALE_STAKE(x) = x / 10^18

    UPSCALE_OWNER(x) = x * 10^9
    DOWNSCALE_OWNER(x) = x / 10^9

    owners_contribution += OWNER_CONTRIBUTION(pool.account)
    stake_contribution = pool.account - owner_contribution

    stake_unit_reward = UPSCALE_STAKE(stake_contribution) / pool.stake
    stake_remainder = UPSCALE_STAKE(stake_contribution) % pool.stake

    owner_unit_reward = UPSCALE_OWNER(owner_contribution) / pool.owners
    owner_remainder = UPSCALE_OWNER(owner_contribution) % pool.owners

    for each owner
        owner.account += owner_unit_reward
    owner[BLOCK_DEPTH % #owners].account += owner_remainder
    for each contributor
        contributor.account += (contributor.stake * stake_unit_reward)
    contributor.

Note: The stake scaling is stronger here as the precision required is also more
important here and the values can be much smaller than the previous algorithm.

Note: We rewards an arbitrary owner of the pool with the

## General implementation details

Every arithmetic operations are conducted on ℕ (natural numbers).

All due care has been taken so that algorithm related to coins are lossless and
implemented using fixed size unsigned integer. Overflow or underflow are
designed to not happens, and if they occur should be a fatal error and the
result of using the wrong fixed size types.

Typically for a 64 bits total value/stake, all division/modulus/scaling operation
should be done pre-casted to 128 bits. It's possible to also sidestep this issue
by using multi precision arithmetic, although all the scaling operations
should remain the same to prevent any differences in the computed values.

Every time the integer division `/` is used, precaution should be taken to
not forget the remainder (operator `%`).

Both `OWNER_CONTRIBUTION` and `TREASURY_CONTRIBUTION` are calculation algorithm
that should return a number inferior or equal to the input. Both algorithms
can take other unspecified inputs from parameter as well if necessary, provided
that the constraint hold.

    OWNER_CONTRIBUTION : n ∈ ℕ → { r ∈ ℕ | r ≤ n }
    TREASURY_CONTRIBUTION : n ∈ ℕ → { r ∈ ℕ | r ≤ n }

For ratio scaling, we expect that the numerator is multiplied first with no overflow, then the integer division of denumerator occurs. effectively:

        A
    V * - = (V * A) / B
        B
