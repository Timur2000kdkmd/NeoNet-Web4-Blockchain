// Governance Contract для NeoNet WASM - DualGov (AI + DAO)
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub ipfs_hash: String,
    pub start_time: u64,
    pub end_time: u64,
    pub for_votes: u128,
    pub against_votes: u128,
    pub ai_score: f64,
    pub executed: bool,
    pub passed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: u64,
    pub support: bool,
    pub weight: u128,
}

pub struct GovernanceContract {
    pub proposals: Vec<Proposal>,
    pub votes: Vec<Vote>,
    pub next_proposal_id: u64,
    pub quorum_percentage: u8,
    pub ai_weight: u8,
    pub dao_weight: u8,
}

impl GovernanceContract {
    pub fn new() -> Self {
        GovernanceContract {
            proposals: Vec::new(),
            votes: Vec::new(),
            next_proposal_id: 1,
            quorum_percentage: 10,
            ai_weight: 30,
            dao_weight: 70,
        }
    }

    pub fn create_proposal(
        &mut self,
        proposer: String,
        title: String,
        description: String,
        ipfs_hash: String,
        start_time: u64,
        duration: u64,
    ) -> Result<u64, String> {
        let proposal = Proposal {
            id: self.next_proposal_id,
            proposer,
            title,
            description,
            ipfs_hash,
            start_time,
            end_time: start_time + duration,
            for_votes: 0,
            against_votes: 0,
            ai_score: 0.0,
            executed: false,
            passed: false,
        };

        self.proposals.push(proposal);
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        Ok(id)
    }

    pub fn vote(
        &mut self,
        voter: String,
        proposal_id: u64,
        support: bool,
        weight: u128,
        current_time: u64,
    ) -> Result<String, String> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| "Proposal not found".to_string())?;

        if current_time < proposal.start_time {
            return Err("Voting not started yet".to_string());
        }

        if current_time > proposal.end_time {
            return Err("Voting period ended".to_string());
        }

        if self.votes.iter().any(|v| v.voter == voter && v.proposal_id == proposal_id) {
            return Err("Already voted".to_string());
        }

        if support {
            proposal.for_votes += weight;
        } else {
            proposal.against_votes += weight;
        }

        self.votes.push(Vote {
            voter: voter.clone(),
            proposal_id,
            support,
            weight,
        });

        Ok(format!("Vote recorded for {} on proposal {}", voter, proposal_id))
    }

    pub fn set_ai_score(&mut self, proposal_id: u64, ai_score: f64) -> Result<(), String> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| "Proposal not found".to_string())?;

        proposal.ai_score = ai_score.clamp(0.0, 1.0);
        Ok(())
    }

    pub fn execute_proposal(
        &mut self,
        proposal_id: u64,
        total_supply: u128,
        current_time: u64,
    ) -> Result<bool, String> {
        let proposal = self.proposals.iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| "Proposal not found".to_string())?;

        if current_time <= proposal.end_time {
            return Err("Voting period not ended".to_string());
        }

        if proposal.executed {
            return Err("Proposal already executed".to_string());
        }

        let total_votes = proposal.for_votes + proposal.against_votes;
        let quorum = (total_supply * self.quorum_percentage as u128) / 100;

        if total_votes < quorum {
            return Err("Quorum not reached".to_string());
        }

        let dao_score = if total_votes > 0 {
            proposal.for_votes as f64 / total_votes as f64
        } else {
            0.0
        };

        let hybrid_score = (dao_score * self.dao_weight as f64 + proposal.ai_score * self.ai_weight as f64) / 100.0;

        proposal.passed = hybrid_score > 0.5;
        proposal.executed = true;

        Ok(proposal.passed)
    }

    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.id == proposal_id)
    }

    pub fn get_all_proposals(&self) -> &Vec<Proposal> {
        &self.proposals
    }

    pub fn set_governance_params(&mut self, quorum: u8, ai_weight: u8, dao_weight: u8) -> Result<(), String> {
        if ai_weight + dao_weight != 100 {
            return Err("AI weight + DAO weight must equal 100".to_string());
        }

        self.quorum_percentage = quorum;
        self.ai_weight = ai_weight;
        self.dao_weight = dao_weight;
        Ok(())
    }
}

impl Default for GovernanceContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let mut contract = GovernanceContract::new();
        let result = contract.create_proposal(
            "alice".to_string(),
            "Test Proposal".to_string(),
            "Description".to_string(),
            "QmHash".to_string(),
            0,
            86400,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_voting() {
        let mut contract = GovernanceContract::new();
        contract.create_proposal(
            "alice".to_string(),
            "Test".to_string(),
            "Desc".to_string(),
            "Hash".to_string(),
            0,
            86400,
        ).unwrap();

        let vote_result = contract.vote("bob".to_string(), 1, true, 1000, 100);
        assert!(vote_result.is_ok());

        let proposal = contract.get_proposal(1).unwrap();
        assert_eq!(proposal.for_votes, 1000);
    }

    #[test]
    fn test_dual_gov() {
        let mut contract = GovernanceContract::new();
        contract.create_proposal(
            "alice".to_string(),
            "Test".to_string(),
            "Desc".to_string(),
            "Hash".to_string(),
            0,
            100,
        ).unwrap();

        contract.vote("bob".to_string(), 1, true, 7000, 10).unwrap();
        contract.vote("charlie".to_string(), 1, false, 3000, 10).unwrap();

        contract.set_ai_score(1, 0.8).unwrap();

        let result = contract.execute_proposal(1, 100000, 200);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
