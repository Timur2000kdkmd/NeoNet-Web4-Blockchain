// Full CosmWasm Staking Contract for NeoNet
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128
};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StakeInfo {
    pub amount: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub reward_rate: Uint128,
}

#[derive(Serialize, Deserialize)]
pub enum ExecuteMsg {
    Stake {},
    Unstake { amount: Uint128 },
    ClaimRewards {},
}

#[derive(Serialize, Deserialize)]
pub enum QueryMsg {
    GetStake { address: String },
    GetTotalStaked {},
    GetRewards { address: String },
}

#[derive(Serialize, Deserialize)]
pub struct StakeResponse {
    pub amount: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TotalStakedResponse {
    pub total: Uint128,
}

#[derive(Serialize, Deserialize)]
pub struct RewardsResponse {
    pub rewards: Uint128,
}

const STAKES: Map<&str, StakeInfo> = Map::new("stakes");
const TOTAL_STAKED: Item<Uint128> = Item::new("total_staked");
const REWARD_RATE: Item<Uint128> = Item::new("reward_rate");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    TOTAL_STAKED.save(deps.storage, &Uint128::zero())?;
    REWARD_RATE.save(deps.storage, &msg.reward_rate)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Stake {} => execute_stake(deps, env, info),
        ExecuteMsg::Unstake { amount } => execute_unstake(deps, env, info, amount),
        ExecuteMsg::ClaimRewards {} => execute_claim_rewards(deps, env, info),
    }
}

fn execute_stake(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uneo")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if amount.is_zero() {
        return Err(StdError::generic_err("Must send NEO tokens to stake"));
    }

    let address = info.sender.as_str();
    let stake = STAKES.may_load(deps.storage, address)?;

    let new_stake = if let Some(mut existing) = stake {
        existing.amount += amount;
        existing.timestamp = env.block.time.seconds();
        existing
    } else {
        StakeInfo {
            amount,
            timestamp: env.block.time.seconds(),
        }
    };

    STAKES.save(deps.storage, address, &new_stake)?;

    let mut total = TOTAL_STAKED.load(deps.storage)?;
    total += amount;
    TOTAL_STAKED.save(deps.storage, &total)?;

    Ok(Response::new()
        .add_attribute("action", "stake")
        .add_attribute("staker", address)
        .add_attribute("amount", amount))
}

fn execute_unstake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let address = info.sender.as_str();
    let mut stake = STAKES.load(deps.storage, address)?;

    if stake.amount < amount {
        return Err(StdError::generic_err("Insufficient staked amount"));
    }

    stake.amount -= amount;

    if stake.amount.is_zero() {
        STAKES.remove(deps.storage, address);
    } else {
        STAKES.save(deps.storage, address, &stake)?;
    }

    let mut total = TOTAL_STAKED.load(deps.storage)?;
    total -= amount;
    TOTAL_STAKED.save(deps.storage, &total)?;

    Ok(Response::new()
        .add_attribute("action", "unstake")
        .add_attribute("staker", address)
        .add_attribute("amount", amount))
}

fn execute_claim_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let address = info.sender.as_str();
    let mut stake = STAKES.load(deps.storage, address)?;
    
    let reward_rate = REWARD_RATE.load(deps.storage)?;
    let time_staked = env.block.time.seconds() - stake.timestamp;
    
    let rewards = stake.amount
        .checked_mul(reward_rate)?
        .checked_mul(Uint128::from(time_staked))?
        .checked_div(Uint128::from(31536000u128))?;

    stake.timestamp = env.block.time.seconds();
    STAKES.save(deps.storage, address, &stake)?;

    Ok(Response::new()
        .add_attribute("action", "claim_rewards")
        .add_attribute("staker", address)
        .add_attribute("rewards", rewards))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStake { address } => to_binary(&query_stake(deps, address)?),
        QueryMsg::GetTotalStaked {} => to_binary(&query_total_staked(deps)?),
        QueryMsg::GetRewards { address } => to_binary(&query_rewards(deps, env, address)?),
    }
}

fn query_stake(deps: Deps, address: String) -> StdResult<StakeResponse> {
    let stake = STAKES.load(deps.storage, &address)?;
    Ok(StakeResponse {
        amount: stake.amount,
        timestamp: stake.timestamp,
    })
}

fn query_total_staked(deps: Deps) -> StdResult<TotalStakedResponse> {
    let total = TOTAL_STAKED.load(deps.storage)?;
    Ok(TotalStakedResponse { total })
}

fn query_rewards(deps: Deps, env: Env, address: String) -> StdResult<RewardsResponse> {
    let stake = STAKES.load(deps.storage, &address)?;
    let reward_rate = REWARD_RATE.load(deps.storage)?;
    let time_staked = env.block.time.seconds() - stake.timestamp;
    
    let rewards = stake.amount
        .checked_mul(reward_rate)?
        .checked_mul(Uint128::from(time_staked))?
        .checked_div(Uint128::from(31536000u128))?;

    Ok(RewardsResponse { rewards })
}
