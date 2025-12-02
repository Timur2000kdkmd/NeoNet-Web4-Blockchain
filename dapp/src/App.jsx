import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { WalletProvider, useWallet } from './components/WalletProvider';
import WalletModal from './components/WalletModal';

const API = '/api';

function DeveloperTab({ wallet, setShowWallet }) {
  const [contractCode, setContractCode] = useState('');
  const [runtime, setRuntime] = useState('hybrid');
  const [deployResult, setDeployResult] = useState(null);
  const [contracts, setContracts] = useState([]);
  const [loading, setLoading] = useState(false);
  const [proposals, setProposals] = useState([]);
  const [newProposal, setNewProposal] = useState({ title: '', description: '' });
  const [aiStatus, setAiStatus] = useState(null);
  const [showExamples, setShowExamples] = useState('connect');

  useEffect(() => {
    loadContracts();
    loadProposals();
    loadAiStatus();
    const i = setInterval(loadAiStatus, 10000);
    return () => clearInterval(i);
  }, []);

  const loadContracts = async () => {
    try {
      const res = await axios.get(`${API}/contracts`);
      if (res.data?.contracts) setContracts(res.data.contracts);
    } catch (e) {}
  };

  const loadProposals = async () => {
    try {
      const res = await axios.get(`${API}/governance/proposals`);
      if (res.data?.proposals) setProposals(res.data.proposals);
    } catch (e) {}
  };

  const loadAiStatus = async () => {
    try {
      const res = await axios.get(`${API}/ai/status`);
      if (res.data) setAiStatus(res.data);
    } catch (e) {}
  };

  const deployContract = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    if (!contractCode) return alert('Enter contract code');
    
    setLoading(true);
    try {
      const res = await axios.post(`${API}/contracts/deploy`, {
        code: contractCode,
        runtime: runtime,
        deployer: wallet.evmAddress
      });
      setDeployResult(res.data);
      loadContracts();
      alert(`Contract deployed! Address: ${res.data.contract_address}`);
    } catch (e) {
      alert('Error: ' + e.message);
    }
    setLoading(false);
  };

  const createProposal = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    if (!newProposal.title || !newProposal.description) return alert('Fill all fields');
    
    try {
      const res = await axios.post(`${API}/governance/proposals`, {
        title: newProposal.title,
        description: newProposal.description,
        proposer: wallet.evmAddress
      });
      alert(`Proposal created! AI recommendation: ${res.data.ai_recommendation} (${(res.data.ai_confidence * 100).toFixed(0)}% confidence)`);
      setNewProposal({ title: '', description: '' });
      loadProposals();
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  const voteOnProposal = async (proposalId, voteFor) => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    
    try {
      await axios.post(`${API}/governance/vote`, {
        proposal_id: proposalId,
        voter: wallet.evmAddress,
        vote_for: voteFor,
        stake_weight: 100
      });
      alert('Vote submitted!');
      loadProposals();
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  return (
    <div>
      <h1 style={styles.title}>Developer - Deploy Contracts</h1>
      
      {/* AI Status Card */}
      {aiStatus && (
        <div style={{...styles.card, background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)', border: '1px solid #4CAF50'}}>
          <h3 style={{color: '#4CAF50'}}>AI Autonomous Training (Active)</h3>
          <p style={{color: '#888'}}>AI trains itself on network data without any user input.</p>
          <div style={{display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '15px', marginTop: '15px'}}>
            <div><span style={{color: '#888'}}>Accuracy:</span> <strong>{aiStatus.accuracy}%</strong></div>
            <div><span style={{color: '#888'}}>Rounds:</span> <strong>{aiStatus.training_rounds}</strong></div>
            <div><span style={{color: '#888'}}>Fraud Detected:</span> <strong>{aiStatus.fraud_detected}</strong></div>
            <div><span style={{color: '#888'}}>Mode:</span> <strong style={{color: '#4CAF50'}}>{aiStatus.mode}</strong></div>
          </div>
        </div>
      )}

      {/* Examples Tabs */}
      <div style={{display: 'flex', gap: '10px', marginBottom: '20px'}}>
        {['connect', 'evm', 'wasm', 'hybrid'].map(ex => (
          <button key={ex} onClick={() => setShowExamples(ex)} style={showExamples === ex ? styles.tabActive : styles.tab}>
            {ex === 'connect' ? 'Connect to NeoNet' : ex.toUpperCase() + ' Contract'}
          </button>
        ))}
      </div>

      {showExamples === 'connect' && (
        <div style={styles.card}>
          <h3>Connect to NeoNet Network</h3>
          <p>NeoNet is NOT Ethereum or Cosmos. It's a unified Web4 blockchain with EVM+WASM integrated runtime.</p>
          <pre style={styles.code}>{`// JavaScript - Connect to NeoNet
import { ethers } from 'ethers';

// NeoNet Network Config
const NEONET_CONFIG = {
  chainId: 8888,
  chainName: 'NeoNet Mainnet',
  rpcUrls: ['https://rpc.neonet.ai'],
  nativeCurrency: {
    name: 'NEO',
    symbol: 'NEO',
    decimals: 18
  },
  blockExplorerUrls: ['https://explorer.neonet.ai']
};

// Connect with ethers.js
const provider = new ethers.JsonRpcProvider(NEONET_CONFIG.rpcUrls[0]);

// Or add to MetaMask
await window.ethereum.request({
  method: 'wallet_addEthereumChain',
  params: [NEONET_CONFIG]
});

// Create wallet with dual keys (EVM + Quantum)
import { NeoNetWallet } from '@neonet/wallet';

const wallet = await NeoNetWallet.create();
console.log('EVM Address:', wallet.evmAddress);     // 0x...
console.log('Quantum Address:', wallet.neoAddress); // neo1...`}</pre>
        </div>
      )}

      {showExamples === 'evm' && (
        <div style={styles.card}>
          <h3>EVM Contract (Solidity) - Deploy to NeoNet</h3>
          <p>Write standard Solidity contracts, but they run on NeoNet's unified runtime.</p>
          <pre style={styles.code}>{`// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

// NeoNet ERC20 Token - uses native NEO for gas
contract MyNeoNetToken is ERC20 {
    constructor() ERC20("My Token", "MTK") {
        _mint(msg.sender, 1000000 * 10**18);
    }
    
    // Cross-runtime call to WASM contract
    function callWasmContract(address wasmAddr) external {
        // NeoNet bridges EVM <-> WASM automatically
        (bool success,) = wasmAddr.call(
            abi.encodeWithSignature("execute()")
        );
        require(success, "WASM call failed");
    }
}

// Deploy using Hardhat
// hardhat.config.js:
module.exports = {
  networks: {
    neonet: {
      url: "https://rpc.neonet.ai",
      chainId: 8888,
      accounts: [process.env.PRIVATE_KEY]
    }
  }
};

// npx hardhat run scripts/deploy.js --network neonet`}</pre>
        </div>
      )}

      {showExamples === 'wasm' && (
        <div style={styles.card}>
          <h3>WASM Contract (Rust/CosmWasm) - Deploy to NeoNet</h3>
          <p>Write Rust contracts for advanced AI/ML integration on NeoNet.</p>
          <pre style={styles.code}>{`// lib.rs - CosmWasm contract for NeoNet
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, 
    Env, MessageInfo, Response, StdResult
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RegisterAiModel { model_id: String, ipfs_hash: String },
    UpdateScore { model_id: String, score: u64 },
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Initialize AI model registry
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterAiModel { model_id, ipfs_hash } => {
            // Register AI model for PoI consensus
            Ok(Response::new()
                .add_attribute("action", "register_model")
                .add_attribute("model_id", model_id))
        }
        ExecuteMsg::UpdateScore { model_id, score } => {
            Ok(Response::new())
        }
    }
}

// Build: cargo build --target wasm32-unknown-unknown --release
// Deploy: neonet-cli tx wasm store ./target/wasm32.wasm`}</pre>
        </div>
      )}

      {showExamples === 'hybrid' && (
        <div style={styles.card}>
          <h3>Hybrid Contract - EVM + WASM Together</h3>
          <p>NeoNet's unique feature: contracts can call across runtimes seamlessly.</p>
          <pre style={styles.code}>{`// CrossRuntimeBridge.sol - EVM side
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface INeoNetBridge {
    function callWasm(bytes32 wasmAddr, bytes calldata data) 
        external returns (bytes memory);
    function getAiScore(address validator) 
        external view returns (uint256);
}

contract HybridAiVoting {
    INeoNetBridge public bridge;
    
    struct Proposal {
        string title;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 aiRecommendation; // 0-100
    }
    
    mapping(uint256 => Proposal) public proposals;
    
    function createProposal(
        string memory title,
        bytes32 wasmAnalyzer
    ) external returns (uint256) {
        // Call WASM AI contract to analyze proposal
        bytes memory result = bridge.callWasm(
            wasmAnalyzer,
            abi.encode("analyze", title)
        );
        
        uint256 aiScore = abi.decode(result, (uint256));
        
        proposals[block.number] = Proposal({
            title: title,
            forVotes: 0,
            againstVotes: 0,
            aiRecommendation: aiScore
        });
        
        return block.number;
    }
    
    function vote(uint256 proposalId, bool support) external {
        uint256 voterAiScore = bridge.getAiScore(msg.sender);
        uint256 weight = 1 + (voterAiScore / 10);
        
        if (support) {
            proposals[proposalId].forVotes += weight;
        } else {
            proposals[proposalId].againstVotes += weight;
        }
    }
}`}</pre>
        </div>
      )}

      {/* Deploy Contract Section */}
      {!wallet.isConnected ? (
        <div style={styles.card}>
          <p>Connect wallet to deploy contracts.</p>
          <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
        </div>
      ) : (
        <>
          <div style={styles.card}>
            <h3>Deploy Smart Contract to NeoNet</h3>
            <p>Deploy using NeoNet's unified EVM+WASM runtime (not Ethereum or Cosmos separately).</p>
            <div style={styles.formGroup}>
              <label>Runtime</label>
              <select value={runtime} onChange={e => setRuntime(e.target.value)} style={styles.input}>
                <option value="hybrid">Hybrid (EVM + WASM)</option>
                <option value="evm">EVM Only (Solidity)</option>
                <option value="wasm">WASM Only (Rust/CosmWasm)</option>
              </select>
            </div>
            <div style={styles.formGroup}>
              <label>Contract Code</label>
              <textarea 
                placeholder={runtime === 'evm' ? 
                  '// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\ncontract MyToken {\n    // ...\n}' : 
                  runtime === 'wasm' ?
                  'use cosmwasm_std::*;\n\n#[entry_point]\npub fn instantiate(...) {\n    // ...\n}' :
                  '// Hybrid contract - uses both EVM and WASM\n// Cross-runtime calls supported'
                }
                value={contractCode}
                onChange={e => setContractCode(e.target.value)}
                style={{...styles.input, minHeight: '150px', fontFamily: 'monospace'}}
              />
            </div>
            <button onClick={deployContract} style={styles.btn} disabled={loading}>
              {loading ? 'Deploying...' : `Deploy to NeoNet (${runtime.toUpperCase()})`}
            </button>
            {deployResult && (
              <div style={{marginTop: '15px', padding: '10px', background: '#1a1a2e', borderRadius: '8px'}}>
                <p><strong>Contract Address:</strong> {deployResult.contract_address}</p>
                <p><strong>Runtime:</strong> {deployResult.runtime}</p>
                <p><strong>TX Hash:</strong> {deployResult.tx_hash}</p>
              </div>
            )}
          </div>

          <div style={styles.card}>
            <h3>Deployed Contracts ({contracts.length})</h3>
            {contracts.length === 0 ? <p>No contracts deployed yet</p> : (
              <table style={styles.table}>
                <thead>
                  <tr><th>Address</th><th>Runtime</th><th>Deployer</th><th>Status</th></tr>
                </thead>
                <tbody>
                  {contracts.map(c => (
                    <tr key={c.address}>
                      <td>{c.address?.slice(0,20)}...</td>
                      <td><span style={c.runtime === 'hybrid' ? styles.online : styles.pending}>{c.runtime}</span></td>
                      <td>{c.deployer?.slice(0,15)}...</td>
                      <td>{c.status}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>

          <h2 style={styles.subtitle}>DualGov - AI + DAO Governance</h2>
          
          <div style={styles.card}>
            <h3>Create Proposal</h3>
            <p>AI analyzes proposals and provides recommendations (30% weight). DAO votes have 70% weight.</p>
            <div style={styles.formGroup}>
              <label>Title</label>
              <input 
                placeholder="Upgrade network security" 
                value={newProposal.title}
                onChange={e => setNewProposal({...newProposal, title: e.target.value})}
                style={styles.input}
              />
            </div>
            <div style={styles.formGroup}>
              <label>Description</label>
              <textarea 
                placeholder="Detailed description of the proposal..."
                value={newProposal.description}
                onChange={e => setNewProposal({...newProposal, description: e.target.value})}
                style={{...styles.input, minHeight: '80px'}}
              />
            </div>
            <button onClick={createProposal} style={styles.btn}>Submit Proposal</button>
          </div>

          <div style={styles.card}>
            <h3>Active Proposals ({proposals.length})</h3>
            {proposals.length === 0 ? <p>No proposals yet</p> : (
              <table style={styles.table}>
                <thead>
                  <tr><th>Title</th><th>Status</th><th>AI Rec.</th><th>For</th><th>Against</th><th>Vote</th></tr>
                </thead>
                <tbody>
                  {proposals.map(p => (
                    <tr key={p.proposal_id}>
                      <td>{p.title}</td>
                      <td><span style={p.status === 'active' ? styles.online : styles.pending}>{p.status}</span></td>
                      <td><span style={p.ai_recommendation === 'for' ? styles.online : p.ai_recommendation === 'against' ? {color: '#e74c3c'} : {color: '#888'}}>{p.ai_recommendation}</span></td>
                      <td>{p.for_votes}</td>
                      <td>{p.against_votes}</td>
                      <td>
                        <button onClick={() => voteOnProposal(p.proposal_id, true)} style={{...styles.btn, padding: '5px 10px', marginRight: 5}}>For</button>
                        <button onClick={() => voteOnProposal(p.proposal_id, false)} style={{...styles.btn, padding: '5px 10px', background: '#e74c3c'}}>Against</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      )}
    </div>
  );
}

function AppContent() {
  const wallet = useWallet();
  const [tab, setTab] = useState('dashboard');
  const [showWallet, setShowWallet] = useState(false);
  const [network, setNetwork] = useState(null);
  const [loading, setLoading] = useState(true);
  const [tasks, setTasks] = useState([]);
  const [miners, setMiners] = useState([]);
  const [sendForm, setSendForm] = useState({ to: '', amount: '', token: 'NEO' });
  const [stakeAmount, setStakeAmount] = useState('');
  const [staked, setStaked] = useState(0);
  const [gpuForm, setGpuForm] = useState({ cores: 8, memory: 8192, endpoint: '' });
  const [flStats, setFlStats] = useState(null);

  useEffect(() => {
    loadNetworkData();
    const i = setInterval(loadNetworkData, 5000);
    return () => clearInterval(i);
  }, []);

  const loadNetworkData = async () => {
    try {
      const [health, poi, taskRes, minerRes] = await Promise.all([
        axios.get(`${API}/health`).catch(() => null),
        axios.get(`${API}/poi/network/stats`).catch(() => null),
        axios.get(`${API}/tasks`).catch(() => null),
        axios.get(`${API}/miners`).catch(() => null)
      ]);
      
      if (health?.data || poi?.data) {
        setNetwork({
          status: health?.data?.status || 'offline',
          miners: health?.data?.miners_count || 0,
          tasks: health?.data?.tasks_count || 0,
          validators: poi?.data?.total_validators || 0,
          stake: poi?.data?.total_stake || 0,
          round: poi?.data?.current_round || 0,
          compute: poi?.data?.total_compute_power || 0,
          quantumSigs: poi?.data?.quantum_signatures_verified || 0,
          hybridSigs: poi?.data?.hybrid_signatures_verified || 0
        });
      }
      if (taskRes?.data?.tasks) setTasks(taskRes.data.tasks);
      if (minerRes?.data?.miners) setMiners(minerRes.data.miners);
      setLoading(false);
    } catch (e) {
      setLoading(false);
    }
  };

  const sendTokens = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    if (!sendForm.to || !sendForm.amount) return alert('Fill all fields');
    
    try {
      const tx = {
        from: wallet.evmAddress,
        to: sendForm.to,
        amount: sendForm.amount,
        token: sendForm.token,
        timestamp: Date.now()
      };
      const signed = await wallet.signTransaction(tx);
      console.log('Signed TX:', signed);
      alert(`Transaction signed!\nFrom: ${wallet.evmAddress}\nTo: ${sendForm.to}\nAmount: ${sendForm.amount} ${sendForm.token}`);
      setSendForm({ to: '', amount: '', token: 'NEO' });
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  const stake = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    if (!stakeAmount) return alert('Enter amount');
    
    try {
      await axios.post(`${API}/poi/validator/register`, {
        validator_id: wallet.evmAddress,
        stake: parseFloat(stakeAmount),
        compute_power: 100
      });
      setStaked(prev => prev + parseFloat(stakeAmount));
      setStakeAmount('');
      alert('Staked successfully!');
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  const registerGPU = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    
    try {
      const res = await axios.post(`${API}/register_miner`, {
        miner_id: wallet.evmAddress,
        cpu_cores: gpuForm.cores,
        gpu_memory_mb: gpuForm.memory,
        endpoint: gpuForm.endpoint || `http://miner-${wallet.evmAddress.slice(0,8)}.local`
      });
      alert('GPU registered! ID: ' + res.data.miner_uid);
      loadNetworkData();
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  const submitAITask = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    
    try {
      const res = await axios.post(`${API}/submit_task`, {
        model_id: 'fraud_detection_v1',
        payload_ref: `ipfs://task_${Date.now()}`,
        priority: 1
      });
      alert('AI Task submitted! ID: ' + res.data.task_id);
      loadNetworkData();
    } catch (e) {
      alert('Error: ' + e.message);
    }
  };

  const trainFL = async () => {
    if (!wallet.isConnected) return alert('Connect wallet first');
    
    try {
      await axios.post(`${API}/fl/register?node_id=${wallet.evmAddress}`);
      const statsRes = await axios.get(`${API}/fl/stats`).catch(() => null);
      if (statsRes?.data) setFlStats(statsRes.data);
      alert('Registered for Federated Learning!');
    } catch (e) {
      if (e.response?.status === 503) {
        alert('Federated Learning engine starting... Try again.');
      } else {
        alert('Error: ' + e.message);
      }
    }
  };

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <div style={styles.logo}>NeoNet</div>
        <nav style={styles.nav}>
          {['dashboard', 'wallet', 'send', 'staking', 'gpu', 'ai', 'developer', 'docs'].map(t => (
            <button key={t} onClick={() => setTab(t)} style={tab === t ? styles.tabActive : styles.tab}>
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </nav>
        <button onClick={() => setShowWallet(true)} style={styles.connectBtn}>
          {wallet.isConnected ? `${wallet.evmAddress?.slice(0,6)}...${wallet.evmAddress?.slice(-4)}` : 'Connect Wallet'}
        </button>
      </header>

      <main style={styles.main}>
        {tab === 'dashboard' && (
          <div>
            <h1 style={styles.title}>Network Dashboard</h1>
            {loading ? <p>Loading network data...</p> : !network ? (
              <div style={styles.error}>
                <h3>Network Offline</h3>
                <p>Backend API not responding. Make sure the server is running on port 8000.</p>
              </div>
            ) : (
              <>
                <div style={styles.grid}>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Status</div>
                    <div style={network.status === 'healthy' ? styles.online : styles.offline}>
                      {network.status}
                    </div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Current Round</div>
                    <div style={styles.cardValue}>{network.round.toLocaleString()}</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Validators</div>
                    <div style={styles.cardValue}>{network.validators}</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Total Stake</div>
                    <div style={styles.cardValue}>{(network.stake / 1000000).toFixed(1)}M NEO</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Miners</div>
                    <div style={styles.cardValue}>{network.miners}</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>AI Tasks</div>
                    <div style={styles.cardValue}>{network.tasks}</div>
                  </div>
                </div>

                <h2 style={styles.subtitle}>Recent Tasks</h2>
                {tasks.length === 0 ? <p>No tasks yet</p> : (
                  <table style={styles.table}>
                    <thead>
                      <tr><th>ID</th><th>Model</th><th>State</th><th>Created</th></tr>
                    </thead>
                    <tbody>
                      {tasks.map(t => (
                        <tr key={t.id}>
                          <td>{t.id.slice(0,8)}...</td>
                          <td>{t.model_id}</td>
                          <td><span style={t.state === 'queued' ? styles.pending : styles.done}>{t.state}</span></td>
                          <td>{new Date(t.created_at * 1000).toLocaleString()}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}

                <h2 style={styles.subtitle}>Active Miners</h2>
                {miners.length === 0 ? <p>No miners registered</p> : (
                  <table style={styles.table}>
                    <thead>
                      <tr><th>ID</th><th>CPU Cores</th><th>GPU Memory</th><th>Endpoint</th></tr>
                    </thead>
                    <tbody>
                      {miners.map(m => (
                        <tr key={m.id}>
                          <td>{m.id.slice(0,8)}...</td>
                          <td>{m.cpu_cores}</td>
                          <td>{m.gpu_memory_mb} MB</td>
                          <td>{m.endpoint}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}

                <div style={{...styles.card, background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)', border: '1px solid #4CAF50', marginTop: 20}}>
                  <h3 style={{color: '#4CAF50', marginBottom: 10}}>Quantum-Safe Network</h3>
                  <p style={{color: '#ccc'}}>All transactions protected with Hybrid Ed25519+Dilithium3 signatures</p>
                  <div style={{display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 15, marginTop: 15}}>
                    <div><span style={{color: '#888'}}>Signature:</span> <strong style={{color: '#4CAF50'}}>Hybrid PQC</strong></div>
                    <div><span style={{color: '#888'}}>Algorithm:</span> <strong>Ed25519+Dilithium3</strong></div>
                    <div><span style={{color: '#888'}}>Protection:</span> <strong style={{color: '#4CAF50'}}>Quantum-Safe</strong></div>
                  </div>
                </div>
              </>
            )}
          </div>
        )}

        {tab === 'wallet' && (
          <div>
            <h1 style={styles.title}>Web4 Wallet</h1>
            {!wallet.isConnected ? (
              <div style={styles.card}>
                <h3>No Wallet Connected</h3>
                <p>Create or import a wallet to access all features.</p>
                <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
              </div>
            ) : (
              <>
                <div style={styles.card}>
                  <div style={styles.cardLabel}>EVM Address (Ethereum Compatible)</div>
                  <code style={styles.address}>{wallet.evmAddress}</code>
                  <button onClick={() => navigator.clipboard.writeText(wallet.evmAddress)} style={styles.copyBtn}>Copy</button>
                </div>
                <div style={styles.card}>
                  <div style={styles.cardLabel}>Quantum Address (Post-Quantum Safe)</div>
                  <code style={styles.address}>{wallet.quantumAddress}</code>
                  <button onClick={() => navigator.clipboard.writeText(wallet.quantumAddress)} style={styles.copyBtn}>Copy</button>
                </div>
                <div style={styles.card}>
                  <div style={styles.cardLabel}>NEO Balance</div>
                  <div style={styles.cardValue}>{(wallet.balances?.NEO || 0).toLocaleString()} NEO</div>
                  <p style={{color: '#888', marginTop: 8, fontSize: 14}}>
                    Earn NEO by: Mining (GPU) | Staking | AI Training | Buy
                  </p>
                </div>
              </>
            )}
          </div>
        )}

        {tab === 'send' && (
          <div>
            <h1 style={styles.title}>Send Tokens</h1>
            {!wallet.isConnected ? (
              <div style={styles.card}>
                <p>Please connect your wallet first.</p>
                <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
              </div>
            ) : (
              <div style={styles.card}>
                <div style={styles.formGroup}>
                  <label>Token</label>
                  <div style={{...styles.input, background: '#333', cursor: 'default'}}>NEO</div>
                </div>
                <div style={styles.formGroup}>
                  <label>Recipient Address</label>
                  <input 
                    placeholder="0x... or neo1..." 
                    value={sendForm.to} 
                    onChange={e => setSendForm({...sendForm, to: e.target.value})} 
                    style={styles.input}
                  />
                </div>
                <div style={styles.formGroup}>
                  <label>Amount</label>
                  <input 
                    type="number" 
                    placeholder="0.00" 
                    value={sendForm.amount} 
                    onChange={e => setSendForm({...sendForm, amount: e.target.value})} 
                    style={styles.input}
                  />
                </div>
                <button onClick={sendTokens} style={styles.btn}>Send Transaction</button>
              </div>
            )}
          </div>
        )}

        {tab === 'staking' && (
          <div>
            <h1 style={styles.title}>Staking</h1>
            {!wallet.isConnected ? (
              <div style={styles.card}>
                <p>Please connect your wallet first.</p>
                <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
              </div>
            ) : (
              <>
                <div style={styles.grid}>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Your Stake</div>
                    <div style={styles.cardValue}>{staked.toLocaleString()} NEO</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>APR</div>
                    <div style={styles.cardValue}>12.5%</div>
                  </div>
                  <div style={styles.card}>
                    <div style={styles.cardLabel}>Status</div>
                    <div style={staked > 0 ? styles.online : styles.offline}>{staked > 0 ? 'Active Validator' : 'Not Staking'}</div>
                  </div>
                </div>
                <div style={styles.card}>
                  <h3>Stake NEO</h3>
                  <div style={styles.formGroup}>
                    <input 
                      type="number" 
                      placeholder="Amount to stake" 
                      value={stakeAmount} 
                      onChange={e => setStakeAmount(e.target.value)} 
                      style={styles.input}
                    />
                  </div>
                  <button onClick={stake} style={styles.btn}>Stake NEO</button>
                </div>
              </>
            )}
          </div>
        )}

        {tab === 'gpu' && (
          <div>
            <h1 style={styles.title}>GPU Mining</h1>
            {!wallet.isConnected ? (
              <div style={styles.card}>
                <p>Please connect your wallet first.</p>
                <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
              </div>
            ) : (
              <>
                <div style={styles.card}>
                  <h3>Register Your GPU</h3>
                  <p>Provide compute power to the network and earn NEO tokens.</p>
                  <div style={styles.formGroup}>
                    <label>CPU Cores</label>
                    <input type="number" value={gpuForm.cores} onChange={e => setGpuForm({...gpuForm, cores: parseInt(e.target.value)})} style={styles.input} />
                  </div>
                  <div style={styles.formGroup}>
                    <label>GPU Memory (MB)</label>
                    <input type="number" value={gpuForm.memory} onChange={e => setGpuForm({...gpuForm, memory: parseInt(e.target.value)})} style={styles.input} />
                  </div>
                  <div style={styles.formGroup}>
                    <label>Endpoint (optional)</label>
                    <input placeholder="http://your-miner.local" value={gpuForm.endpoint} onChange={e => setGpuForm({...gpuForm, endpoint: e.target.value})} style={styles.input} />
                  </div>
                  <button onClick={registerGPU} style={styles.btn}>Register GPU Miner</button>
                </div>

                <div style={styles.card}>
                  <h3>Submit AI Task</h3>
                  <p>Submit a fraud detection task to the network.</p>
                  <button onClick={submitAITask} style={styles.btn}>Submit AI Task</button>
                </div>
              </>
            )}
          </div>
        )}

        {tab === 'ai' && (
          <div>
            <h1 style={styles.title}>AI Federated Learning</h1>
            {!wallet.isConnected ? (
              <div style={styles.card}>
                <p>Please connect your wallet first.</p>
                <button onClick={() => setShowWallet(true)} style={styles.btn}>Connect Wallet</button>
              </div>
            ) : (
              <>
                <div style={styles.card}>
                  <h3>How It Works</h3>
                  <ol style={styles.list}>
                    <li>Register your node for federated learning</li>
                    <li>Download global model weights</li>
                    <li>Train locally on your data</li>
                    <li>Submit gradients to the network</li>
                    <li>AI aggregates all updates (FedAvg)</li>
                    <li>Earn rewards for valid contributions</li>
                  </ol>
                </div>
                
                <div style={styles.card}>
                  <h3>Join Federated Learning</h3>
                  <button onClick={trainFL} style={styles.btn}>Register Node</button>
                  {flStats && (
                    <div style={{marginTop: '20px'}}>
                      <p>Nodes: {flStats.total_nodes}</p>
                      <p>Rounds: {flStats.training_rounds}</p>
                    </div>
                  )}
                </div>

                <div style={styles.card}>
                  <h3>AI Consensus (Proof of Intelligence)</h3>
                  <p>Validators prove their intelligence by training ML models. The network:</p>
                  <ul style={styles.list}>
                    <li>Verifies gradient computations</li>
                    <li>Checks model improvement</li>
                    <li>Aggregates updates with FedAvg</li>
                    <li>Rewards honest validators</li>
                  </ul>
                </div>
              </>
            )}
          </div>
        )}

        {tab === 'developer' && (
          <DeveloperTab wallet={wallet} setShowWallet={setShowWallet} />
        )}

        {tab === 'docs' && (
          <div>
            <h1 style={styles.title}>Documentation</h1>
            
            <div style={styles.card}>
              <h3>Download Documentation</h3>
              <p>Get the complete NeoNet documentation:</p>
              <a href="/docs/neonet-whitepaper.pdf" download style={styles.downloadBtn}>
                Download Whitepaper (PDF)
              </a>
              <a href="/docs/api-reference.pdf" download style={{...styles.downloadBtn, marginLeft: '10px'}}>
                Download API Reference (PDF)
              </a>
            </div>

            <div style={styles.card}>
              <h3>Architecture Overview</h3>
              <ul style={styles.list}>
                <li><strong>Proof of Intelligence (PoI)</strong> - AI validates blocks by training ML models</li>
                <li><strong>Unified Dual Runtime</strong> - EVM + WASM as integrated fabric, not separate chains</li>
                <li><strong>Post-Quantum Security</strong> - Ed25519 + Dilithium3 hybrid signatures</li>
                <li><strong>Single Token Economy</strong> - NEO (50M supply) for all operations</li>
                <li><strong>DualGov</strong> - 30% AI + 70% DAO voting on proposals</li>
                <li><strong>Federated Learning</strong> - Train on real network transactions and attacks</li>
              </ul>
            </div>

            <div style={styles.card}>
              <h3>NEO Token</h3>
              <p>Single native token powering the entire NeoNet ecosystem.</p>
              <table style={styles.table}>
                <thead>
                  <tr><th>Use Case</th><th>Description</th></tr>
                </thead>
                <tbody>
                  <tr><td>Governance</td><td>Vote on proposals with AI assistance</td></tr>
                  <tr><td>Staking</td><td>Stake to become validator, earn 12.5% APR</td></tr>
                  <tr><td>Gas</td><td>Pay for transactions and contract execution</td></tr>
                  <tr><td>Mining Rewards</td><td>Earn for providing GPU compute</td></tr>
                  <tr><td>AI Training</td><td>Rewards for federated learning contributions</td></tr>
                </tbody>
              </table>
              <p style={{marginTop: 10}}><strong>Total Supply:</strong> 50,000,000 NEO</p>
            </div>

            <div style={styles.card}>
              <h3>Quick Links</h3>
              <ul style={styles.list}>
                <li><a href="https://github.com/neonet" target="_blank">GitHub Repository</a></li>
                <li><a href="https://discord.gg/neonet" target="_blank">Discord Community</a></li>
                <li><a href="https://twitter.com/NeoNetAI" target="_blank">Twitter</a></li>
              </ul>
            </div>
          </div>
        )}
      </main>

      <WalletModal isOpen={showWallet} onClose={() => setShowWallet(false)} />
    </div>
  );
}

const styles = {
  app: { 
    minHeight: '100vh', 
    background: '#0d1117', 
    color: '#c9d1d9', 
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif' 
  },
  header: { 
    display: 'flex', 
    alignItems: 'center', 
    padding: '16px 24px', 
    borderBottom: '1px solid #21262d', 
    background: '#161b22' 
  },
  logo: { 
    fontSize: '24px', 
    fontWeight: 'bold', 
    color: '#58a6ff', 
    marginRight: '40px' 
  },
  nav: { 
    display: 'flex', 
    gap: '8px', 
    flex: 1 
  },
  tab: { 
    padding: '8px 16px', 
    background: 'transparent', 
    border: '1px solid #30363d', 
    borderRadius: '6px', 
    color: '#8b949e', 
    cursor: 'pointer' 
  },
  tabActive: { 
    padding: '8px 16px', 
    background: '#238636', 
    border: '1px solid #238636', 
    borderRadius: '6px', 
    color: '#fff', 
    cursor: 'pointer' 
  },
  connectBtn: { 
    padding: '8px 16px', 
    background: '#238636', 
    border: 'none', 
    borderRadius: '6px', 
    color: '#fff', 
    cursor: 'pointer',
    fontWeight: '600'
  },
  main: { 
    padding: '24px', 
    maxWidth: '1200px', 
    margin: '0 auto' 
  },
  title: { 
    fontSize: '32px', 
    marginBottom: '24px', 
    color: '#fff' 
  },
  subtitle: { 
    fontSize: '20px', 
    marginTop: '32px', 
    marginBottom: '16px', 
    color: '#fff' 
  },
  grid: { 
    display: 'grid', 
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', 
    gap: '16px', 
    marginBottom: '24px' 
  },
  card: { 
    background: '#161b22', 
    border: '1px solid #30363d', 
    borderRadius: '8px', 
    padding: '20px' 
  },
  cardLabel: { 
    fontSize: '14px', 
    color: '#8b949e', 
    marginBottom: '8px' 
  },
  cardValue: { 
    fontSize: '28px', 
    fontWeight: 'bold', 
    color: '#fff' 
  },
  online: { 
    color: '#3fb950', 
    fontWeight: 'bold', 
    fontSize: '18px' 
  },
  offline: { 
    color: '#f85149', 
    fontWeight: 'bold', 
    fontSize: '18px' 
  },
  error: { 
    background: '#21262d', 
    border: '1px solid #f85149', 
    borderRadius: '8px', 
    padding: '24px', 
    textAlign: 'center' 
  },
  table: { 
    width: '100%', 
    borderCollapse: 'collapse', 
    background: '#161b22', 
    borderRadius: '8px', 
    overflow: 'hidden' 
  },
  pending: { 
    background: '#1f6feb', 
    color: '#fff', 
    padding: '2px 8px', 
    borderRadius: '12px', 
    fontSize: '12px' 
  },
  done: { 
    background: '#238636', 
    color: '#fff', 
    padding: '2px 8px', 
    borderRadius: '12px', 
    fontSize: '12px' 
  },
  address: { 
    display: 'block', 
    background: '#0d1117', 
    padding: '12px', 
    borderRadius: '6px', 
    wordBreak: 'break-all', 
    marginBottom: '8px', 
    fontSize: '14px' 
  },
  copyBtn: { 
    padding: '6px 12px', 
    background: '#21262d', 
    border: '1px solid #30363d', 
    borderRadius: '6px', 
    color: '#8b949e', 
    cursor: 'pointer' 
  },
  formGroup: { 
    marginBottom: '16px' 
  },
  input: { 
    width: '100%', 
    padding: '12px', 
    background: '#0d1117', 
    border: '1px solid #30363d', 
    borderRadius: '6px', 
    color: '#c9d1d9', 
    fontSize: '14px' 
  },
  btn: { 
    padding: '12px 24px', 
    background: '#238636', 
    border: 'none', 
    borderRadius: '6px', 
    color: '#fff', 
    cursor: 'pointer', 
    fontWeight: '600' 
  },
  list: { 
    paddingLeft: '20px', 
    lineHeight: '2' 
  },
  code: { 
    background: '#0d1117', 
    padding: '16px', 
    borderRadius: '6px', 
    overflow: 'auto', 
    fontSize: '14px', 
    lineHeight: '1.6' 
  },
  downloadBtn: { 
    display: 'inline-block', 
    padding: '12px 24px', 
    background: '#1f6feb', 
    color: '#fff', 
    borderRadius: '6px', 
    textDecoration: 'none', 
    fontWeight: '600' 
  }
};

export default function App() {
  return (
    <WalletProvider>
      <AppContent />
    </WalletProvider>
  );
}
