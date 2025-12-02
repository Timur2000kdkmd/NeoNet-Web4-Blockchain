// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract WasmXToken is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 500_000_000 * 10**18;
    
    mapping(address => bool) public wasmContractDeployers;
    mapping(address => uint256) public deployerRewards;
    
    uint256 public rewardPerDeploy = 100 * 10**18;
    
    event WasmContractDeployed(address indexed deployer, address indexed contractAddr, uint256 reward);
    event DeployerRegistered(address indexed deployer);
    
    constructor() ERC20("WASM-X Token", "WASMX") Ownable(msg.sender) {
        _mint(msg.sender, 100_000_000 * 10**18);
    }
    
    function registerDeployer(address deployer) external onlyOwner {
        wasmContractDeployers[deployer] = true;
        emit DeployerRegistered(deployer);
    }
    
    function rewardWasmDeploy(address deployer, address contractAddr) external onlyOwner {
        require(totalSupply() + rewardPerDeploy <= MAX_SUPPLY, "Max supply reached");
        
        _mint(deployer, rewardPerDeploy);
        deployerRewards[deployer] += rewardPerDeploy;
        
        emit WasmContractDeployed(deployer, contractAddr, rewardPerDeploy);
    }
    
    function setRewardPerDeploy(uint256 newReward) external onlyOwner {
        rewardPerDeploy = newReward;
    }
    
    function burn(uint256 amount) external {
        _burn(msg.sender, amount);
    }
}
