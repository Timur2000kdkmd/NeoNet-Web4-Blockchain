// Staking Contract для NeoNet WASM
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StakeInfo {
    pub staker: String,
    pub amount: u128,
    pub timestamp: u64,
    pub reward: u128,
}

#[derive(Serialize, Deserialize)]
pub struct StakeMsg {
    pub amount: u128,
}

#[derive(Serialize, Deserialize)]
pub struct UnstakeMsg {
    pub amount: u128,
}

#[derive(Serialize, Deserialize)]
pub struct ClaimRewardsMsg {}

pub struct StakingContract {
    pub stakes: Vec<StakeInfo>,
    pub total_staked: u128,
    pub reward_rate: u128,
}

impl StakingContract {
    pub fn new() -> Self {
        StakingContract {
            stakes: Vec::new(),
            total_staked: 0,
            reward_rate: 100,
        }
    }

    pub fn stake(&mut self, staker: String, amount: u128, timestamp: u64) -> Result<String, String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let existing_stake = self.stakes.iter_mut().find(|s| s.staker == staker);

        if let Some(stake) = existing_stake {
            stake.amount += amount;
        } else {
            self.stakes.push(StakeInfo {
                staker: staker.clone(),
                amount,
                timestamp,
                reward: 0,
            });
        }

        self.total_staked += amount;
        Ok(format!("Staked {} from {}", amount, staker))
    }

    pub fn unstake(&mut self, staker: String, amount: u128) -> Result<String, String> {
        let stake = self.stakes.iter_mut().find(|s| s.staker == staker)
            .ok_or_else(|| "Stake not found".to_string())?;

        if stake.amount < amount {
            return Err("Insufficient staked amount".to_string());
        }

        stake.amount -= amount;
        self.total_staked -= amount;

        if stake.amount == 0 {
            self.stakes.retain(|s| s.staker != staker);
        }

        Ok(format!("Unstaked {} from {}", amount, staker))
    }

    pub fn calculate_rewards(&self, staker: &str, current_time: u64) -> u128 {
        if let Some(stake) = self.stakes.iter().find(|s| s.staker == staker) {
            let time_staked = current_time.saturating_sub(stake.timestamp);
            let reward = (stake.amount * self.reward_rate * time_staked as u128) / (86400 * 365 * 100);
            stake.reward + reward
        } else {
            0
        }
    }

    pub fn claim_rewards(&mut self, staker: String, current_time: u64) -> Result<u128, String> {
        let stake = self.stakes.iter_mut().find(|s| s.staker == staker)
            .ok_or_else(|| "Stake not found".to_string())?;

        let rewards = self.calculate_rewards(&staker, current_time);
        stake.reward = 0;
        stake.timestamp = current_time;

        Ok(rewards)
    }

    pub fn get_stake(&self, staker: &str) -> Option<&StakeInfo> {
        self.stakes.iter().find(|s| s.staker == staker)
    }

    pub fn get_total_staked(&self) -> u128 {
        self.total_staked
    }
}

impl Default for StakingContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake() {
        let mut contract = StakingContract::new();
        let result = contract.stake("alice".to_string(), 1000, 0);
        assert!(result.is_ok());
        assert_eq!(contract.get_total_staked(), 1000);
    }

    #[test]
    fn test_unstake() {
        let mut contract = StakingContract::new();
        contract.stake("alice".to_string(), 1000, 0).unwrap();
        let result = contract.unstake("alice".to_string(), 500);
        assert!(result.is_ok());
        assert_eq!(contract.get_total_staked(), 500);
    }

    #[test]
    fn test_rewards() {
        let contract = StakingContract::new();
        let mut contract = contract;
        contract.stake("alice".to_string(), 1000, 0).unwrap();
        
        let rewards = contract.calculate_rewards("alice", 86400);
        assert!(rewards > 0);
    }
}
