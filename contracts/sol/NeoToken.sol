// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title NEO Token
 * @dev Main gas and governance token for NeoNet Web4 blockchain
 * Features:
 * - ERC20 with voting extensions for DualGov
 * - Burnable for gas consumption
 * - Role-based minting for validators
 * - Staking integration
 */
contract NeoToken is ERC20, ERC20Burnable, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant VALIDATOR_ROLE = keccak256("VALIDATOR_ROLE");
    
    uint256 public constant MAX_SUPPLY = 1_000_000_000 * 10**18;
    uint256 public constant INITIAL_SUPPLY = 100_000_000 * 10**18;
    
    mapping(address => uint256) public stakedBalance;
    mapping(address => uint256) public stakeTimestamp;
    
    uint256 public totalStaked;
    uint256 public stakingAPY = 1250;
    uint256 public minStakePeriod = 7 days;
    
    event Staked(address indexed user, uint256 amount);
    event Unstaked(address indexed user, uint256 amount, uint256 rewards);
    event RewardsDistributed(address indexed validator, uint256 amount);
    
    constructor() ERC20("NeoNet", "NEO") {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(VALIDATOR_ROLE, msg.sender);
        
        _mint(msg.sender, INITIAL_SUPPLY);
    }
    
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(to, amount);
    }
    
    function stake(uint256 amount) external {
        require(amount > 0, "Cannot stake 0");
        require(balanceOf(msg.sender) >= amount, "Insufficient balance");
        
        if (stakedBalance[msg.sender] > 0) {
            _claimRewards();
        }
        
        _transfer(msg.sender, address(this), amount);
        stakedBalance[msg.sender] += amount;
        stakeTimestamp[msg.sender] = block.timestamp;
        totalStaked += amount;
        
        emit Staked(msg.sender, amount);
    }
    
    function unstake() external {
        uint256 staked = stakedBalance[msg.sender];
        require(staked > 0, "No stake found");
        require(block.timestamp >= stakeTimestamp[msg.sender] + minStakePeriod, "Stake period not ended");
        
        uint256 rewards = calculateRewards(msg.sender);
        
        stakedBalance[msg.sender] = 0;
        totalStaked -= staked;
        
        _transfer(address(this), msg.sender, staked);
        if (rewards > 0 && totalSupply() + rewards <= MAX_SUPPLY) {
            _mint(msg.sender, rewards);
        }
        
        emit Unstaked(msg.sender, staked, rewards);
    }
    
    function _claimRewards() internal {
        uint256 rewards = calculateRewards(msg.sender);
        if (rewards > 0 && totalSupply() + rewards <= MAX_SUPPLY) {
            _mint(msg.sender, rewards);
            stakeTimestamp[msg.sender] = block.timestamp;
        }
    }
    
    function calculateRewards(address user) public view returns (uint256) {
        if (stakedBalance[user] == 0) return 0;
        
        uint256 duration = block.timestamp - stakeTimestamp[user];
        return (stakedBalance[user] * stakingAPY * duration) / (365 days * 10000);
    }
    
    function distributeValidatorRewards(address validator, uint256 amount) external onlyRole(VALIDATOR_ROLE) {
        require(totalSupply() + amount <= MAX_SUPPLY, "Exceeds max supply");
        _mint(validator, amount);
        emit RewardsDistributed(validator, amount);
    }
    
    function setStakingAPY(uint256 newAPY) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newAPY <= 5000, "APY too high");
        stakingAPY = newAPY;
    }
    
    function getStakeInfo(address user) external view returns (uint256 staked, uint256 rewards, uint256 timestamp) {
        return (stakedBalance[user], calculateRewards(user), stakeTimestamp[user]);
    }
}
