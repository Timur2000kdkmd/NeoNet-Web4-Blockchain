// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";

interface INeoToken {
    function balanceOf(address account) external view returns (uint256);
}

contract DualGov is Ownable {
    INeoToken public neoToken;
    
    uint256 public constant DAO_WEIGHT = 70;
    uint256 public constant AI_WEIGHT = 30;
    uint256 public constant QUORUM_PERCENTAGE = 10;
    uint256 public constant VOTING_PERIOD = 3 days;
    
    struct AIAgent {
        bool isVerified;
        uint256 confidenceScore;
        uint256 proposalsValidated;
        string modelHash;
    }
    
    struct Proposal {
        uint256 id;
        address proposer;
        string title;
        string description;
        string ipfsHash;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 aiScore;
        uint256 startTime;
        uint256 endTime;
        bool executed;
        bool cancelled;
        ProposalType proposalType;
        bytes callData;
        address targetContract;
    }
    
    enum ProposalType {
        ParameterChange,
        ContractUpgrade,
        TreasurySpend,
        AIModelUpdate,
        NetworkUpgrade
    }
    
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public hasVoted;
    mapping(address => AIAgent) public aiAgents;
    
    uint256 public proposalCount;
    uint256 public minProposalThreshold = 10000 * 10**18;
    
    address[] public verifiedAIAgents;
    
    event ProposalCreated(uint256 indexed proposalId, address indexed proposer, string title, ProposalType proposalType);
    event Voted(uint256 indexed proposalId, address indexed voter, bool support, uint256 weight);
    event AIScoreSubmitted(uint256 indexed proposalId, address indexed aiAgent, uint256 score);
    event ProposalExecuted(uint256 indexed proposalId, bool success);
    event AIAgentVerified(address indexed agent, string modelHash);
    event ProposalCancelled(uint256 indexed proposalId);
    
    constructor(address _neoToken) Ownable(msg.sender) {
        neoToken = INeoToken(_neoToken);
    }
    
    function verifyAIAgent(address agent, string calldata modelHash, uint256 initialConfidence) external onlyOwner {
        require(!aiAgents[agent].isVerified, "Already verified");
        require(initialConfidence <= 100, "Confidence must be <= 100");
        
        aiAgents[agent] = AIAgent({
            isVerified: true,
            confidenceScore: initialConfidence,
            proposalsValidated: 0,
            modelHash: modelHash
        });
        
        verifiedAIAgents.push(agent);
        emit AIAgentVerified(agent, modelHash);
    }
    
    function createProposal(
        string calldata title,
        string calldata description,
        string calldata ipfsHash,
        ProposalType proposalType,
        address targetContract,
        bytes calldata callData
    ) external returns (uint256) {
        require(neoToken.balanceOf(msg.sender) >= minProposalThreshold, "Insufficient NEO balance");
        
        proposalCount++;
        
        proposals[proposalCount] = Proposal({
            id: proposalCount,
            proposer: msg.sender,
            title: title,
            description: description,
            ipfsHash: ipfsHash,
            forVotes: 0,
            againstVotes: 0,
            aiScore: 0,
            startTime: block.timestamp,
            endTime: block.timestamp + VOTING_PERIOD,
            executed: false,
            cancelled: false,
            proposalType: proposalType,
            callData: callData,
            targetContract: targetContract
        });
        
        emit ProposalCreated(proposalCount, msg.sender, title, proposalType);
        return proposalCount;
    }
    
    function vote(uint256 proposalId, bool support) external {
        Proposal storage proposal = proposals[proposalId];
        
        require(proposal.id != 0, "Proposal does not exist");
        require(block.timestamp < proposal.endTime, "Voting ended");
        require(!proposal.cancelled, "Proposal cancelled");
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        
        uint256 weight = neoToken.balanceOf(msg.sender);
        require(weight > 0, "No voting power");
        
        hasVoted[proposalId][msg.sender] = true;
        
        if (support) {
            proposal.forVotes += weight;
        } else {
            proposal.againstVotes += weight;
        }
        
        emit Voted(proposalId, msg.sender, support, weight);
    }
    
    function submitAIScore(uint256 proposalId, uint256 score) external {
        require(aiAgents[msg.sender].isVerified, "Not a verified AI agent");
        require(score <= 100, "Score must be <= 100");
        
        Proposal storage proposal = proposals[proposalId];
        require(proposal.id != 0, "Proposal does not exist");
        require(!proposal.executed, "Already executed");
        
        proposal.aiScore = score;
        aiAgents[msg.sender].proposalsValidated++;
        
        emit AIScoreSubmitted(proposalId, msg.sender, score);
    }
    
    function calculateFinalScore(uint256 proposalId) public view returns (uint256 daoScore, uint256 aiScore, uint256 finalScore, bool passed) {
        Proposal storage proposal = proposals[proposalId];
        
        uint256 totalVotes = proposal.forVotes + proposal.againstVotes;
        
        if (totalVotes == 0) {
            return (0, proposal.aiScore, 0, false);
        }
        
        daoScore = (proposal.forVotes * 100) / totalVotes;
        aiScore = proposal.aiScore;
        
        finalScore = (daoScore * DAO_WEIGHT + aiScore * AI_WEIGHT) / 100;
        
        passed = finalScore >= 50;
        
        return (daoScore, aiScore, finalScore, passed);
    }
    
    function executeProposal(uint256 proposalId) external {
        Proposal storage proposal = proposals[proposalId];
        
        require(proposal.id != 0, "Proposal does not exist");
        require(block.timestamp >= proposal.endTime, "Voting not ended");
        require(!proposal.executed, "Already executed");
        require(!proposal.cancelled, "Proposal cancelled");
        
        (,, uint256 finalScore, bool passed) = calculateFinalScore(proposalId);
        require(passed, "Proposal did not pass");
        
        proposal.executed = true;
        
        if (proposal.targetContract != address(0) && proposal.callData.length > 0) {
            (bool success,) = proposal.targetContract.call(proposal.callData);
            emit ProposalExecuted(proposalId, success);
        } else {
            emit ProposalExecuted(proposalId, true);
        }
    }
    
    function cancelProposal(uint256 proposalId) external {
        Proposal storage proposal = proposals[proposalId];
        require(msg.sender == proposal.proposer || msg.sender == owner(), "Not authorized");
        require(!proposal.executed, "Already executed");
        
        proposal.cancelled = true;
        emit ProposalCancelled(proposalId);
    }
    
    function getProposal(uint256 proposalId) external view returns (
        address proposer,
        string memory title,
        string memory description,
        uint256 forVotes,
        uint256 againstVotes,
        uint256 aiScore,
        uint256 endTime,
        bool executed,
        ProposalType proposalType
    ) {
        Proposal storage p = proposals[proposalId];
        return (
            p.proposer,
            p.title,
            p.description,
            p.forVotes,
            p.againstVotes,
            p.aiScore,
            p.endTime,
            p.executed,
            p.proposalType
        );
    }
    
    function getVerifiedAgentCount() external view returns (uint256) {
        return verifiedAIAgents.length;
    }
    
    function setMinProposalThreshold(uint256 newThreshold) external onlyOwner {
        minProposalThreshold = newThreshold;
    }
}
