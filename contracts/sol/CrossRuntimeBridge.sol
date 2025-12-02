// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

interface INeoToken {
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

contract CrossRuntimeBridge is Ownable, ReentrancyGuard {
    
    INeoToken public neoToken;
    INeoToken public wasmxToken;
    INeoToken public cptToken;
    
    enum BridgeType { TokenBridge, DataBridge, CallBridge, StateBridge }
    enum TransferStatus { Pending, Completed, Failed, Refunded }
    
    struct BridgeTransfer {
        bytes32 transferId;
        address sender;
        string wasmRecipient;
        uint256 amount;
        address tokenAddress;
        uint256 timestamp;
        TransferStatus status;
        bytes32 wasmTxHash;
        bytes quantumSignature;
    }
    
    struct WasmBridge {
        string wasmContractId;
        BridgeType bridgeType;
        bool isActive;
        uint256 totalTransfers;
        uint256 totalVolume;
    }
    
    struct CrossRuntimeCall {
        bytes32 callId;
        address caller;
        string targetWasmContract;
        string method;
        bytes params;
        bytes result;
        bool executed;
        uint256 gasUsed;
    }
    
    mapping(bytes32 => BridgeTransfer) public transfers;
    mapping(string => WasmBridge) public wasmBridges;
    mapping(bytes32 => CrossRuntimeCall) public crossCalls;
    mapping(address => uint256) public lockedBalances;
    
    bytes32[] public transferIds;
    bytes32[] public callIds;
    string[] public registeredBridges;
    
    address[] public relayers;
    mapping(address => bool) public isRelayer;
    
    uint256 public bridgeFee = 10; // 0.1% fee (10 basis points)
    uint256 public minTransferAmount = 100 * 10**18;
    uint256 public transferNonce;
    
    event BridgeRegistered(string indexed bridgeId, string wasmContractId, BridgeType bridgeType);
    event TransferInitiated(bytes32 indexed transferId, address indexed sender, string wasmRecipient, uint256 amount, address token);
    event TransferCompleted(bytes32 indexed transferId, bytes32 wasmTxHash);
    event TransferFailed(bytes32 indexed transferId, string reason);
    event CrossRuntimeCallInitiated(bytes32 indexed callId, address caller, string targetContract, string method);
    event CrossRuntimeCallCompleted(bytes32 indexed callId, bytes result);
    event RelayerAdded(address indexed relayer);
    event RelayerRemoved(address indexed relayer);
    event TokensLocked(address indexed user, address token, uint256 amount);
    event TokensUnlocked(address indexed user, address token, uint256 amount);
    
    modifier onlyRelayer() {
        require(isRelayer[msg.sender] || msg.sender == owner(), "Not authorized relayer");
        _;
    }
    
    constructor(
        address _neoToken,
        address _wasmxToken,
        address _cptToken
    ) Ownable(msg.sender) {
        neoToken = INeoToken(_neoToken);
        wasmxToken = INeoToken(_wasmxToken);
        cptToken = INeoToken(_cptToken);
        
        isRelayer[msg.sender] = true;
        relayers.push(msg.sender);
    }
    
    function registerWasmBridge(
        string calldata bridgeId,
        string calldata wasmContractId,
        BridgeType bridgeType
    ) external onlyOwner {
        require(bytes(wasmBridges[bridgeId].wasmContractId).length == 0, "Bridge already exists");
        
        wasmBridges[bridgeId] = WasmBridge({
            wasmContractId: wasmContractId,
            bridgeType: bridgeType,
            isActive: true,
            totalTransfers: 0,
            totalVolume: 0
        });
        
        registeredBridges.push(bridgeId);
        
        emit BridgeRegistered(bridgeId, wasmContractId, bridgeType);
    }
    
    function initiateTransferToWasm(
        string calldata bridgeId,
        string calldata wasmRecipient,
        uint256 amount,
        address tokenAddress,
        bytes calldata quantumSignature
    ) external nonReentrant returns (bytes32) {
        require(wasmBridges[bridgeId].isActive, "Bridge not active");
        require(amount >= minTransferAmount, "Amount below minimum");
        
        INeoToken token = _getToken(tokenAddress);
        require(token.balanceOf(msg.sender) >= amount, "Insufficient balance");
        
        uint256 fee = (amount * bridgeFee) / 10000;
        uint256 netAmount = amount - fee;
        
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        lockedBalances[tokenAddress] += netAmount;
        
        transferNonce++;
        bytes32 transferId = keccak256(abi.encodePacked(
            msg.sender,
            wasmRecipient,
            amount,
            block.timestamp,
            transferNonce
        ));
        
        transfers[transferId] = BridgeTransfer({
            transferId: transferId,
            sender: msg.sender,
            wasmRecipient: wasmRecipient,
            amount: netAmount,
            tokenAddress: tokenAddress,
            timestamp: block.timestamp,
            status: TransferStatus.Pending,
            wasmTxHash: bytes32(0),
            quantumSignature: quantumSignature
        });
        
        transferIds.push(transferId);
        
        wasmBridges[bridgeId].totalTransfers++;
        wasmBridges[bridgeId].totalVolume += netAmount;
        
        emit TransferInitiated(transferId, msg.sender, wasmRecipient, netAmount, tokenAddress);
        emit TokensLocked(msg.sender, tokenAddress, netAmount);
        
        return transferId;
    }
    
    function completeTransfer(
        bytes32 transferId,
        bytes32 wasmTxHash
    ) external onlyRelayer {
        BridgeTransfer storage transfer = transfers[transferId];
        require(transfer.status == TransferStatus.Pending, "Transfer not pending");
        
        transfer.status = TransferStatus.Completed;
        transfer.wasmTxHash = wasmTxHash;
        
        lockedBalances[transfer.tokenAddress] -= transfer.amount;
        
        emit TransferCompleted(transferId, wasmTxHash);
    }
    
    function failTransfer(bytes32 transferId, string calldata reason) external onlyRelayer {
        BridgeTransfer storage transfer = transfers[transferId];
        require(transfer.status == TransferStatus.Pending, "Transfer not pending");
        
        transfer.status = TransferStatus.Failed;
        
        INeoToken token = _getToken(transfer.tokenAddress);
        lockedBalances[transfer.tokenAddress] -= transfer.amount;
        require(token.transfer(transfer.sender, transfer.amount), "Refund failed");
        
        emit TransferFailed(transferId, reason);
        emit TokensUnlocked(transfer.sender, transfer.tokenAddress, transfer.amount);
    }
    
    function receiveFromWasm(
        address evmRecipient,
        uint256 amount,
        address tokenAddress,
        bytes32 wasmTxHash,
        bytes calldata wasmProof
    ) external onlyRelayer nonReentrant {
        require(wasmProof.length > 0, "Invalid WASM proof");
        
        INeoToken token = _getToken(tokenAddress);
        require(token.balanceOf(address(this)) >= amount, "Insufficient bridge balance");
        
        require(token.transfer(evmRecipient, amount), "Transfer to recipient failed");
        
        emit TokensUnlocked(evmRecipient, tokenAddress, amount);
    }
    
    function initiateCrossRuntimeCall(
        string calldata targetWasmContract,
        string calldata method,
        bytes calldata params
    ) external returns (bytes32) {
        bytes32 callId = keccak256(abi.encodePacked(
            msg.sender,
            targetWasmContract,
            method,
            block.timestamp,
            transferNonce++
        ));
        
        crossCalls[callId] = CrossRuntimeCall({
            callId: callId,
            caller: msg.sender,
            targetWasmContract: targetWasmContract,
            method: method,
            params: params,
            result: "",
            executed: false,
            gasUsed: 0
        });
        
        callIds.push(callId);
        
        emit CrossRuntimeCallInitiated(callId, msg.sender, targetWasmContract, method);
        
        return callId;
    }
    
    function completeCrossRuntimeCall(
        bytes32 callId,
        bytes calldata result,
        uint256 gasUsed
    ) external onlyRelayer {
        CrossRuntimeCall storage call = crossCalls[callId];
        require(!call.executed, "Already executed");
        
        call.result = result;
        call.executed = true;
        call.gasUsed = gasUsed;
        
        emit CrossRuntimeCallCompleted(callId, result);
    }
    
    function addRelayer(address relayer) external onlyOwner {
        require(!isRelayer[relayer], "Already a relayer");
        isRelayer[relayer] = true;
        relayers.push(relayer);
        emit RelayerAdded(relayer);
    }
    
    function removeRelayer(address relayer) external onlyOwner {
        require(isRelayer[relayer], "Not a relayer");
        isRelayer[relayer] = false;
        emit RelayerRemoved(relayer);
    }
    
    function setBridgeFee(uint256 newFee) external onlyOwner {
        require(newFee <= 100, "Fee too high"); // Max 1%
        bridgeFee = newFee;
    }
    
    function setMinTransferAmount(uint256 newMin) external onlyOwner {
        minTransferAmount = newMin;
    }
    
    function toggleBridge(string calldata bridgeId, bool active) external onlyOwner {
        wasmBridges[bridgeId].isActive = active;
    }
    
    function _getToken(address tokenAddress) internal view returns (INeoToken) {
        if (tokenAddress == address(neoToken)) return neoToken;
        if (tokenAddress == address(wasmxToken)) return wasmxToken;
        if (tokenAddress == address(cptToken)) return cptToken;
        revert("Unknown token");
    }
    
    function getTransfer(bytes32 transferId) external view returns (
        address sender,
        string memory wasmRecipient,
        uint256 amount,
        address tokenAddress,
        uint256 timestamp,
        TransferStatus status
    ) {
        BridgeTransfer storage t = transfers[transferId];
        return (t.sender, t.wasmRecipient, t.amount, t.tokenAddress, t.timestamp, t.status);
    }
    
    function getBridgeStats(string calldata bridgeId) external view returns (
        string memory wasmContractId,
        bool isActive,
        uint256 totalTransfers,
        uint256 totalVolume
    ) {
        WasmBridge storage b = wasmBridges[bridgeId];
        return (b.wasmContractId, b.isActive, b.totalTransfers, b.totalVolume);
    }
    
    function getTransferCount() external view returns (uint256) {
        return transferIds.length;
    }
    
    function getCrossCallCount() external view returns (uint256) {
        return callIds.length;
    }
    
    function getRelayerCount() external view returns (uint256) {
        return relayers.length;
    }
}
