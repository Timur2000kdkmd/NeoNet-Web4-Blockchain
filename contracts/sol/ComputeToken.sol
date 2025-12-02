// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ComputeToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 1_000_000_000 * 10**18;
    
    struct ComputeProvider {
        bool isActive;
        uint256 totalCompute;
        uint256 pendingRewards;
        uint256 lastClaimBlock;
    }
    
    mapping(address => ComputeProvider) public providers;
    address[] public providerList;
    
    uint256 public rewardPerComputeUnit = 10 * 10**18;
    uint256 public minComputeForReward = 100;
    
    event ComputeProvided(address indexed provider, uint256 computeUnits, uint256 reward);
    event ProviderRegistered(address indexed provider);
    event RewardsClaimed(address indexed provider, uint256 amount);
    
    constructor() ERC20("Compute Token", "CPT") Ownable(msg.sender) {
        _mint(msg.sender, 200_000_000 * 10**18);
    }
    
    function registerProvider(address provider) external onlyOwner {
        require(!providers[provider].isActive, "Already registered");
        
        providers[provider] = ComputeProvider({
            isActive: true,
            totalCompute: 0,
            pendingRewards: 0,
            lastClaimBlock: block.number
        });
        
        providerList.push(provider);
        emit ProviderRegistered(provider);
    }
    
    function recordCompute(address provider, uint256 computeUnits) external onlyOwner {
        require(providers[provider].isActive, "Provider not registered");
        require(computeUnits >= minComputeForReward, "Compute below minimum");
        require(totalSupply() + (computeUnits * rewardPerComputeUnit / 100) <= MAX_SUPPLY, "Max supply reached");
        
        uint256 reward = (computeUnits * rewardPerComputeUnit) / 100;
        
        providers[provider].totalCompute += computeUnits;
        providers[provider].pendingRewards += reward;
        
        emit ComputeProvided(provider, computeUnits, reward);
    }
    
    function claimRewards() external {
        ComputeProvider storage provider = providers[msg.sender];
        require(provider.isActive, "Not a registered provider");
        require(provider.pendingRewards > 0, "No pending rewards");
        
        uint256 rewards = provider.pendingRewards;
        provider.pendingRewards = 0;
        provider.lastClaimBlock = block.number;
        
        _mint(msg.sender, rewards);
        emit RewardsClaimed(msg.sender, rewards);
    }
    
    function getProviderInfo(address provider) external view returns (
        bool isActive,
        uint256 totalCompute,
        uint256 pendingRewards,
        uint256 lastClaimBlock
    ) {
        ComputeProvider storage p = providers[provider];
        return (p.isActive, p.totalCompute, p.pendingRewards, p.lastClaimBlock);
    }
    
    function getProviderCount() external view returns (uint256) {
        return providerList.length;
    }
    
    function setRewardPerComputeUnit(uint256 newReward) external onlyOwner {
        rewardPerComputeUnit = newReward;
    }
    
    function burn(uint256 amount) external {
        _burn(msg.sender, amount);
    }
}
