// Dual Runtime Router - Smart routing between EVM and WASM
// Routes transactions based on signature type (ECDSA -> EVM, Dilithium -> WASM)

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::evm_adapter::EVMAdapter;
use crate::wasm_vm::WasmVM;
use crate::pqc::{HybridSignature, HybridPublicKey, verify_hybrid_signature};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    EVM,
    WASM,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    ECDSA,
    Dilithium,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub nonce: u64,
    pub signature_type: SignatureType,
    pub signature: Vec<u8>,
    pub runtime_hint: Option<RuntimeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub runtime_used: RuntimeType,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

pub struct DualRuntimeRouter {
    evm: EVMAdapter,
    wasm: WasmVM,
    routing_rules: HashMap<String, RuntimeType>,
    execution_stats: ExecutionStats,
}

#[derive(Default)]
struct ExecutionStats {
    evm_calls: u64,
    wasm_calls: u64,
    hybrid_calls: u64,
    total_gas_used: u64,
}

impl DualRuntimeRouter {
    pub fn new() -> Self {
        DualRuntimeRouter {
            evm: EVMAdapter::new(),
            wasm: WasmVM::new(10_000_000),
            routing_rules: HashMap::new(),
            execution_stats: ExecutionStats::default(),
        }
    }

    /// Route transaction to appropriate runtime based on signature type and hints
    pub fn route(&mut self, tx: &Transaction) -> Result<ExecutionResult> {
        let runtime = self.determine_runtime(tx);
        
        println!("Routing transaction to {:?} runtime", runtime);
        
        match runtime {
            RuntimeType::EVM => self.execute_evm(tx),
            RuntimeType::WASM => self.execute_wasm(tx),
            RuntimeType::Hybrid => self.execute_hybrid(tx),
        }
    }

    /// Determine which runtime to use based on transaction properties
    fn determine_runtime(&self, tx: &Transaction) -> RuntimeType {
        // Check explicit runtime hint first
        if let Some(ref hint) = tx.runtime_hint {
            return hint.clone();
        }

        // Check contract address format
        if tx.to.starts_with("neo1") {
            return RuntimeType::WASM;
        }
        if tx.to.starts_with("0x") {
            return RuntimeType::EVM;
        }

        // Route based on signature type
        match tx.signature_type {
            SignatureType::ECDSA => RuntimeType::EVM,
            SignatureType::Dilithium => RuntimeType::WASM,
            SignatureType::Hybrid => RuntimeType::Hybrid,
        }
    }

    /// Execute transaction in EVM runtime
    fn execute_evm(&mut self, tx: &Transaction) -> Result<ExecutionResult> {
        self.execution_stats.evm_calls += 1;

        // Verify signature for EVM (simplified)
        if !self.verify_ecdsa_signature(tx) {
            return Ok(ExecutionResult {
                success: false,
                runtime_used: RuntimeType::EVM,
                gas_used: 0,
                output: vec![],
                logs: vec![],
                error: Some("Invalid ECDSA signature".to_string()),
            });
        }

        // Execute in EVM
        let result = if tx.data.is_empty() {
            // Simple transfer
            self.evm.transfer(&tx.from, &tx.to, tx.value)
                .map(|_| vec![])
        } else {
            // Contract call
            self.evm.call_contract(&tx.from, &tx.to, tx.data.clone(), tx.value, tx.gas_limit)
        };

        match result {
            Ok(output) => {
                let gas_used = self.estimate_evm_gas(&tx.data);
                self.execution_stats.total_gas_used += gas_used;
                
                Ok(ExecutionResult {
                    success: true,
                    runtime_used: RuntimeType::EVM,
                    gas_used,
                    output,
                    logs: vec!["EVM execution successful".to_string()],
                    error: None,
                })
            }
            Err(e) => Ok(ExecutionResult {
                success: false,
                runtime_used: RuntimeType::EVM,
                gas_used: 21000, // Base gas for failed tx
                output: vec![],
                logs: vec![],
                error: Some(e.to_string()),
            })
        }
    }

    /// Execute transaction in WASM runtime
    fn execute_wasm(&mut self, tx: &Transaction) -> Result<ExecutionResult> {
        self.execution_stats.wasm_calls += 1;

        // Verify Dilithium signature (simplified)
        if !self.verify_dilithium_signature(tx) {
            return Ok(ExecutionResult {
                success: false,
                runtime_used: RuntimeType::WASM,
                gas_used: 0,
                output: vec![],
                logs: vec![],
                error: Some("Invalid Dilithium signature".to_string()),
            });
        }

        // Execute in WASM
        let contract_id = tx.to.clone();
        
        let result = if tx.data.len() > 4 {
            // Contract call with function selector
            self.wasm.execute_contract(&contract_id, &tx.data)
        } else {
            // Simple execution
            self.wasm.execute_contract(&contract_id, &[])
        };

        match result {
            Ok(output) => {
                let gas_used = self.wasm.get_gas_used();
                self.execution_stats.total_gas_used += gas_used;
                
                Ok(ExecutionResult {
                    success: true,
                    runtime_used: RuntimeType::WASM,
                    gas_used,
                    output,
                    logs: vec!["WASM execution successful".to_string()],
                    error: None,
                })
            }
            Err(e) => Ok(ExecutionResult {
                success: false,
                runtime_used: RuntimeType::WASM,
                gas_used: 10000,
                output: vec![],
                logs: vec![],
                error: Some(e.to_string()),
            })
        }
    }

    /// Execute transaction in both runtimes (Hybrid mode)
    fn execute_hybrid(&mut self, tx: &Transaction) -> Result<ExecutionResult> {
        self.execution_stats.hybrid_calls += 1;

        // Execute in EVM first
        let evm_result = self.execute_evm(tx)?;
        
        // Then execute in WASM for verification
        let wasm_result = self.execute_wasm(tx)?;

        // Both must succeed for hybrid execution
        let success = evm_result.success && wasm_result.success;
        let total_gas = evm_result.gas_used + wasm_result.gas_used;

        Ok(ExecutionResult {
            success,
            runtime_used: RuntimeType::Hybrid,
            gas_used: total_gas,
            output: if success { evm_result.output } else { vec![] },
            logs: vec![
                format!("EVM: {}", if evm_result.success { "OK" } else { "FAIL" }),
                format!("WASM: {}", if wasm_result.success { "OK" } else { "FAIL" }),
            ],
            error: if !success {
                Some("Hybrid execution failed in one or both runtimes".to_string())
            } else {
                None
            },
        })
    }

    /// Deploy contract to appropriate runtime
    pub fn deploy_contract(&mut self, tx: &Transaction) -> Result<String> {
        let runtime = self.determine_runtime(tx);

        match runtime {
            RuntimeType::EVM => {
                self.evm.deploy_contract(&tx.from, tx.data.clone())
            }
            RuntimeType::WASM => {
                let contract_id = format!("neo1{}", hex::encode(&tx.data[..20.min(tx.data.len())]));
                self.wasm.deploy_contract(contract_id.clone(), tx.data.clone())?;
                Ok(contract_id)
            }
            RuntimeType::Hybrid => {
                // Deploy to both runtimes
                let evm_addr = self.evm.deploy_contract(&tx.from, tx.data.clone())?;
                let wasm_id = format!("neo1{}", &evm_addr[2..]);
                self.wasm.deploy_contract(wasm_id.clone(), tx.data.clone())?;
                Ok(format!("{}|{}", evm_addr, wasm_id))
            }
        }
    }

    /// Add routing rule for specific contract
    pub fn add_routing_rule(&mut self, contract_addr: String, runtime: RuntimeType) {
        self.routing_rules.insert(contract_addr, runtime);
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("evm_calls".to_string(), self.execution_stats.evm_calls);
        stats.insert("wasm_calls".to_string(), self.execution_stats.wasm_calls);
        stats.insert("hybrid_calls".to_string(), self.execution_stats.hybrid_calls);
        stats.insert("total_gas_used".to_string(), self.execution_stats.total_gas_used);
        stats
    }

    // Simplified signature verification (in production, use real crypto)
    fn verify_ecdsa_signature(&self, _tx: &Transaction) -> bool {
        true // Simplified for demo
    }

    fn verify_dilithium_signature(&self, _tx: &Transaction) -> bool {
        true // Simplified for demo
    }

    fn estimate_evm_gas(&self, data: &[u8]) -> u64 {
        21000 + (data.len() as u64 * 16) // Base + calldata cost
    }
}

impl Default for DualRuntimeRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_by_address() {
        let router = DualRuntimeRouter::new();
        
        let evm_tx = Transaction {
            from: "0xalice".to_string(),
            to: "0xcontract".to_string(),
            value: 0,
            data: vec![],
            gas_limit: 100000,
            nonce: 0,
            signature_type: SignatureType::ECDSA,
            signature: vec![],
            runtime_hint: None,
        };
        
        let runtime = router.determine_runtime(&evm_tx);
        assert!(matches!(runtime, RuntimeType::EVM));
        
        let wasm_tx = Transaction {
            from: "neo1alice".to_string(),
            to: "neo1contract".to_string(),
            value: 0,
            data: vec![],
            gas_limit: 100000,
            nonce: 0,
            signature_type: SignatureType::Dilithium,
            signature: vec![],
            runtime_hint: None,
        };
        
        let runtime = router.determine_runtime(&wasm_tx);
        assert!(matches!(runtime, RuntimeType::WASM));
    }

    #[test]
    fn test_routing_by_signature_type() {
        let router = DualRuntimeRouter::new();
        
        let tx = Transaction {
            from: "unknown".to_string(),
            to: "unknown".to_string(),
            value: 0,
            data: vec![],
            gas_limit: 100000,
            nonce: 0,
            signature_type: SignatureType::Hybrid,
            signature: vec![],
            runtime_hint: None,
        };
        
        let runtime = router.determine_runtime(&tx);
        assert!(matches!(runtime, RuntimeType::Hybrid));
    }
}
