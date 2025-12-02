mod bridge;
mod wasm_vm;
mod pqc;
mod evm_adapter;

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::{fs, time::{SystemTime, UNIX_EPOCH}, collections::VecDeque, sync::{Arc, Mutex}};
use wasm_vm::WasmVM;
use pqc::{HybridKeyPair, verify_hybrid_signature};
use evm_adapter::EVMAdapter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tx {
    pub from: String,
    pub to: String,
    pub payload: String,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub prev_hash: String,
    pub timestamp: u128,
    pub txs: Vec<Tx>,
    pub nonce: u64,
    pub proposer: String,
    pub hash: String,
}

impl Block {
    pub fn compute_hash(&self) -> String {
        let s = serde_json::to_string(&(
            self.index,
            &self.prev_hash,
            self.timestamp,
            &self.txs,
            self.nonce,
            &self.proposer
        )).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(s);
        hex::encode(hasher.finalize())
    }
}

pub struct Chain {
    pub blocks: Vec<Block>,
    pub mempool: VecDeque<Tx>,
    pub validators: Vec<String>,
    pub next_proposer_idx: usize,
}

impl Chain {
    pub fn new(validators: Vec<String>) -> Self {
        let genesis = Block {
            index: 0,
            prev_hash: "0".repeat(64),
            timestamp: now_millis(),
            txs: vec![],
            nonce: 0,
            proposer: String::from("genesis"),
            hash: "0".repeat(64),
        };
        Chain { blocks: vec![genesis], mempool: VecDeque::new(), validators, next_proposer_idx: 0 }
    }

    pub fn add_tx(&mut self, tx: Tx) {
        self.mempool.push_back(tx);
    }

    pub fn rotate_proposer(&mut self) -> String {
        if self.validators.is_empty() { return "".into(); }
        let p = self.validators[self.next_proposer_idx % self.validators.len()].clone();
        self.next_proposer_idx = (self.next_proposer_idx + 1) % self.validators.len();
        p
    }

    pub fn mine_block(&mut self) -> Block {
        // deterministic proposer rotation
        let proposer = self.rotate_proposer();
        let txs: Vec<Tx> = self.mempool.drain(..).collect();
        let prev = self.blocks.last().unwrap();
        let mut block = Block {
            index: prev.index + 1,
            prev_hash: prev.hash.clone(),
            timestamp: now_millis(),
            txs,
            nonce: 0,
            proposer: proposer.clone(),
            hash: String::new(),
        };
        block.hash = block.compute_hash();
        self.blocks.push(block.clone());
        println!("Mined block {} by {}", block.index, proposer);
        block
    }

    pub fn validate(&self) -> bool {
        for i in 1..self.blocks.len() {
            let cur = &self.blocks[i];
            let prev = &self.blocks[i-1];
            if cur.prev_hash != prev.hash { return false; }
            if cur.compute_hash() != cur.hash { return false; }
        }
        true
    }
}

fn now_millis() -> u128 {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    d.as_millis()
}

fn main() {
    println!("=== NeoNet Blockchain Core Starting ===");
    println!("Version: 0.1.0 - Web4 AI-Powered Blockchain");
    
    println!("\n1. Initializing Post-Quantum Cryptography...");
    let keypair = HybridKeyPair::generate();
    let public_key = keypair.public_key();
    let test_msg = b"NeoNet Proof of Intelligence";
    let signature = keypair.sign(test_msg);
    let is_valid = verify_hybrid_signature(&public_key, test_msg, &signature).unwrap();
    println!("   PQC Test: Signature valid = {}", is_valid);
    
    println!("\n2. Initializing WASM Virtual Machine...");
    let mut wasm_vm = WasmVM::new(1000000);
    let contract_code = vec![0x00, 0x61, 0x73, 0x6d];
    wasm_vm.deploy_contract("wasm_contract_1".to_string(), contract_code).unwrap();
    println!("   WASM VM: Contract deployed, Gas used: {}", wasm_vm.get_gas_used());
    
    println!("\n3. Initializing EVM Adapter...");
    let mut evm = EVMAdapter::new();
    evm.create_account("0xalice".to_string(), 1000000).unwrap();
    evm.create_account("0xbob".to_string(), 500000).unwrap();
    evm.transfer("0xalice", "0xbob", 10000).unwrap();
    println!("   EVM: Accounts created, Transfer successful");
    println!("   Alice balance: {}, Bob balance: {}", 
        evm.get_balance("0xalice").unwrap(),
        evm.get_balance("0xbob").unwrap()
    );
    
    println!("\n4. Starting Blockchain...");
    bridge::start_bridge();
    
    let validators = vec!["validator1".into(), "validator2".into(), "validator3".into()];
    let mut chain = Chain::new(validators);
    
    println!("   Genesis block created");
    
    chain.add_tx(Tx{
        from: "alice".into(), 
        to: "bob".into(), 
        payload: "transfer 10 NEO".into(), 
        nonce: 0
    });
    
    let block1 = chain.mine_block();
    println!("   Block {} mined by {}", block1.index, block1.proposer);
    
    chain.add_tx(Tx{
        from: "bob".into(),
        to: "charlie".into(),
        payload: "transfer 5 NEO".into(),
        nonce: 1
    });
    
    let block2 = chain.mine_block();
    println!("   Block {} mined by {}", block2.index, block2.proposer);
    
    println!("   Chain validation: {}", chain.validate());
    println!("   Total blocks: {}", chain.blocks.len());
    
    println!("\n=== NeoNet Core Initialized Successfully ===");
    println!("Bridge running on port 6000");
    println!("Blockchain Core: {} blocks", chain.blocks.len());
    println!("WASM VM: Ready");
    println!("EVM Adapter: Ready");
    println!("PQC: Ready (Ed25519-Hybrid)");
    println!("\nPress Ctrl+C to shutdown");
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
