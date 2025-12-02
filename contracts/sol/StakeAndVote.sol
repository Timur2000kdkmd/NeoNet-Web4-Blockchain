// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./NeoToken.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract StakeAndVote is ReentrancyGuard, Ownable {
    address public oracleContract;
    function setOracle(address _oracle) external onlyOwner {
        oracleContract = _oracle;
    }
    IERC20 public token; // supports snapshot/votes if ERC20Votes implemented
    uint256 public totalStaked;
    uint256 public quorumBps = 1000; // 10% default (basis points)


    struct StakeInfo {
        uint256 amount;
        uint256 timestamp;
    }
    mapping(address => StakeInfo) public stakes;

    struct Proposal {
        uint256 id;
        address proposer;
        string ipfsHash;
        uint256 start;
        uint256 end;
        uint256 forVotes;
        uint256 againstVotes;
        bool executed;
    }
    uint256 public nextProposalId;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public hasVoted;

    event Staked(address indexed who, uint256 amount);
    event Unstaked(address indexed who, uint256 amount);
    event ProposalCreated(uint256 id, address proposer, string ipfsHash, uint256 start, uint256 end);
    event Voted(uint256 proposalId, address voter, bool support, uint256 weight);

    constructor(address tokenAddr) { 
        // owner set in Ownable constructor
 
        // set deployer as owner
        _transferOwnership(msg.sender);
        token = NeoToken(tokenAddr);
    }

    function stake(uint256 amount) external nonReentrant {
        require(amount > 0, "Zero");
        token.transferFrom(msg.sender, address(this), amount);
        stakes[msg.sender].amount += amount;
        stakes[msg.sender].timestamp = block.timestamp;
        totalStaked += amount;
        emit Staked(msg.sender, amount);
    }

    function unstake(uint256 amount) external nonReentrant {
        require(amount > 0 && stakes[msg.sender].amount >= amount, "Bad amount");
        stakes[msg.sender].amount -= amount;
        totalStaked -= amount;
        token.transfer(msg.sender, amount);
        emit Unstaked(msg.sender, amount);
    }

    function createProposal(string calldata ipfsHash, uint256 durationSeconds) external returns (uint256) {
        require(durationSeconds >= 60, "Too short");
        uint256 id = ++nextProposalId;
        proposals[id] = Proposal({
            id: id,
            proposer: msg.sender,
            ipfsHash: ipfsHash,
            start: block.timestamp,
            end: block.timestamp + durationSeconds,
            forVotes: 0,
            againstVotes: 0,
            executed: false
        });
        emit ProposalCreated(id, msg.sender, ipfsHash, block.timestamp, block.timestamp+durationSeconds);
        return id;
    }

    function vote(uint256 proposalId, bool support) external {
        Proposal storage p = proposals[proposalId];
        require(block.timestamp >= p.start && block.timestamp <= p.end, "Voting closed");
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        uint256 weight = stakes[msg.sender].amount;
        require(weight > 0, "No stake");
        hasVoted[proposalId][msg.sender] = true;
        if (support) {
            p.forVotes += weight;
        } else {
            p.againstVotes += weight;
        }
        emit Voted(proposalId, msg.sender, support, weight);
    }

    event ProposalExecuted(uint256 indexed proposalId, bytes32 reportId, bytes32 resultHash);

    function executeByOracle(uint256 proposalId, bytes32 reportId, bytes32 resultHash) external {
        require(msg.sender == oracleContract, "only oracle");
        Proposal storage p = proposals[proposalId];
        require(block.timestamp > p.end, "Not finished");
        require(!p.executed, "Executed");
        
        p.executed = true;
        emit ProposalExecuted(proposalId, reportId, resultHash);
    }

    function executeProposal(uint256 proposalId) external {
        Proposal storage p = proposals[proposalId];
        require(block.timestamp > p.end, "Not finished");
        require(!p.executed, "Executed");
        uint256 totalVotes = p.forVotes + p.againstVotes;
        require(totalVotes * 10000 / (totalStaked==0?1:totalStaked) >= quorumBps, "quorum not met");
        require(p.forVotes > p.againstVotes, "Proposal rejected");
        
        p.executed = true;
        emit ProposalExecuted(proposalId, bytes32(0), bytes32(0));
    }

    function setQuorumBps(uint256 bps) external onlyOwner { 
        require(bps<=10000, "Invalid BPS"); 
        quorumBps = bps; 
    }
}
