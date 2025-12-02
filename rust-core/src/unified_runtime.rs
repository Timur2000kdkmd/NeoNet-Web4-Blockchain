use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeType {
    EVM,
    WASM,
    Hybrid,
    AIOptimized,
}

#[derive(Debug, Clone)]
pub struct DualAddress {
    pub evm_address: [u8; 20],
    pub neo_address: String,
    pub account_id: [u8; 32],
}

impl DualAddress {
    pub fn from_evm(evm_addr: [u8; 20]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&evm_addr);
        let hash = hasher.finalize();
        let neo_addr = format!("neo1{}", hex::encode(&hash[..19]));
        
        let mut account_id = [0u8; 32];
        account_id.copy_from_slice(&hash);
        
        Self {
            evm_address: evm_addr,
            neo_address: neo_addr,
            account_id,
        }
    }
    
    pub fn from_neo(neo_addr: &str) -> Option<Self> {
        if !neo_addr.starts_with("neo1") || neo_addr.len() < 42 {
            return None;
        }
        
        let hex_part = &neo_addr[4..];
        let bytes = hex::decode(hex_part).ok()?;
        
        let mut evm_address = [0u8; 20];
        evm_address.copy_from_slice(&bytes[..20.min(bytes.len())]);
        
        let mut hasher = Sha256::new();
        hasher.update(&evm_address);
        let hash = hasher.finalize();
        let mut account_id = [0u8; 32];
        account_id.copy_from_slice(&hash);
        
        Some(Self {
            evm_address,
            neo_address: neo_addr.to_string(),
            account_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DualSignature {
    pub ecdsa_sig: Option<Vec<u8>>,
    pub dilithium_sig: Option<Vec<u8>>,
    pub signature_mode: SignatureMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignatureMode {
    EVMOnly,
    QuantumOnly,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct UnifiedTransaction {
    pub tx_hash: [u8; 32],
    pub from: DualAddress,
    pub to: Option<DualAddress>,
    pub value: u128,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
    pub signature: DualSignature,
    pub runtime_hint: Option<RuntimeType>,
    pub cross_runtime_calls: Vec<CrossRuntimeCall>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct CrossRuntimeCall {
    pub source_runtime: RuntimeType,
    pub target_runtime: RuntimeType,
    pub target_contract: DualAddress,
    pub method: String,
    pub params: Vec<u8>,
    pub gas_budget: u64,
}

#[derive(Debug, Clone)]
pub struct UnifiedAccount {
    pub account_id: [u8; 32],
    pub dual_address: DualAddress,
    pub balance: u128,
    pub nonce: u64,
    pub evm_code_hash: Option<[u8; 32]>,
    pub wasm_code_id: Option<u64>,
    pub storage_root: [u8; 32],
    pub ai_reputation_score: f64,
    pub quantum_key_registered: bool,
}

#[derive(Debug, Clone)]
pub struct DualStateNode {
    pub hash: [u8; 32],
    pub evm_state_root: [u8; 32],
    pub wasm_state_root: [u8; 32],
    pub shared_state_root: [u8; 32],
    pub height: u64,
}

pub struct DualStateEngine {
    accounts: Arc<RwLock<HashMap<[u8; 32], UnifiedAccount>>>,
    evm_storage: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,
    wasm_storage: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,
    shared_storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    state_root: Arc<RwLock<DualStateNode>>,
    pending_changes: Arc<RwLock<Vec<StateChange>>>,
}

#[derive(Debug, Clone)]
pub enum StateChange {
    AccountUpdate(UnifiedAccount),
    EVMStorageWrite { key: [u8; 32], value: Vec<u8> },
    WASMStorageWrite { key: [u8; 32], value: Vec<u8> },
    SharedStorageWrite { key: String, value: Vec<u8> },
}

impl DualStateEngine {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            evm_storage: Arc::new(RwLock::new(HashMap::new())),
            wasm_storage: Arc::new(RwLock::new(HashMap::new())),
            shared_storage: Arc::new(RwLock::new(HashMap::new())),
            state_root: Arc::new(RwLock::new(DualStateNode {
                hash: [0u8; 32],
                evm_state_root: [0u8; 32],
                wasm_state_root: [0u8; 32],
                shared_state_root: [0u8; 32],
                height: 0,
            })),
            pending_changes: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn get_account(&self, account_id: &[u8; 32]) -> Option<UnifiedAccount> {
        self.accounts.read().unwrap().get(account_id).cloned()
    }
    
    pub fn create_account(&self, dual_address: DualAddress) -> UnifiedAccount {
        let account = UnifiedAccount {
            account_id: dual_address.account_id,
            dual_address: dual_address.clone(),
            balance: 0,
            nonce: 0,
            evm_code_hash: None,
            wasm_code_id: None,
            storage_root: [0u8; 32],
            ai_reputation_score: 0.5,
            quantum_key_registered: false,
        };
        
        self.accounts.write().unwrap().insert(dual_address.account_id, account.clone());
        self.pending_changes.write().unwrap().push(StateChange::AccountUpdate(account.clone()));
        
        account
    }
    
    pub fn update_balance(&self, account_id: &[u8; 32], new_balance: u128) -> bool {
        let mut accounts = self.accounts.write().unwrap();
        if let Some(account) = accounts.get_mut(account_id) {
            account.balance = new_balance;
            self.pending_changes.write().unwrap().push(StateChange::AccountUpdate(account.clone()));
            true
        } else {
            false
        }
    }
    
    pub fn write_evm_storage(&self, key: [u8; 32], value: Vec<u8>) {
        self.evm_storage.write().unwrap().insert(key, value.clone());
        self.pending_changes.write().unwrap().push(StateChange::EVMStorageWrite { key, value });
    }
    
    pub fn write_wasm_storage(&self, key: [u8; 32], value: Vec<u8>) {
        self.wasm_storage.write().unwrap().insert(key, value.clone());
        self.pending_changes.write().unwrap().push(StateChange::WASMStorageWrite { key, value });
    }
    
    pub fn write_shared_storage(&self, key: String, value: Vec<u8>) {
        self.shared_storage.write().unwrap().insert(key.clone(), value.clone());
        self.pending_changes.write().unwrap().push(StateChange::SharedStorageWrite { key, value });
    }
    
    pub fn read_evm_storage(&self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.evm_storage.read().unwrap().get(key).cloned()
    }
    
    pub fn read_wasm_storage(&self, key: &[u8; 32]) -> Option<Vec<u8>> {
        self.wasm_storage.read().unwrap().get(key).cloned()
    }
    
    pub fn read_shared_storage(&self, key: &str) -> Option<Vec<u8>> {
        self.shared_storage.read().unwrap().get(key).cloned()
    }
    
    pub fn compute_state_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        let accounts = self.accounts.read().unwrap();
        for (id, account) in accounts.iter() {
            hasher.update(id);
            hasher.update(&account.balance.to_le_bytes());
            hasher.update(&account.nonce.to_le_bytes());
        }
        
        let evm = self.evm_storage.read().unwrap();
        for (k, v) in evm.iter() {
            hasher.update(k);
            hasher.update(v);
        }
        
        let wasm = self.wasm_storage.read().unwrap();
        for (k, v) in wasm.iter() {
            hasher.update(k);
            hasher.update(v);
        }
        
        let hash = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&hash);
        root
    }
    
    pub fn commit(&self, height: u64) -> DualStateNode {
        let root = self.compute_state_root();
        
        let node = DualStateNode {
            hash: root,
            evm_state_root: self.compute_evm_root(),
            wasm_state_root: self.compute_wasm_root(),
            shared_state_root: self.compute_shared_root(),
            height,
        };
        
        *self.state_root.write().unwrap() = node.clone();
        self.pending_changes.write().unwrap().clear();
        
        node
    }
    
    pub fn rollback(&self) {
        self.pending_changes.write().unwrap().clear();
    }
    
    fn compute_evm_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"EVM_STATE");
        let storage = self.evm_storage.read().unwrap();
        for (k, v) in storage.iter() {
            hasher.update(k);
            hasher.update(v);
        }
        let hash = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&hash);
        root
    }
    
    fn compute_wasm_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"WASM_STATE");
        let storage = self.wasm_storage.read().unwrap();
        for (k, v) in storage.iter() {
            hasher.update(k);
            hasher.update(v);
        }
        let hash = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&hash);
        root
    }
    
    fn compute_shared_root(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"SHARED_STATE");
        let storage = self.shared_storage.read().unwrap();
        for (k, v) in storage.iter() {
            hasher.update(k.as_bytes());
            hasher.update(v);
        }
        let hash = hasher.finalize();
        let mut root = [0u8; 32];
        root.copy_from_slice(&hash);
        root
    }
}

pub struct CrossVMCallManager {
    state: Arc<DualStateEngine>,
    call_stack: Arc<RwLock<Vec<CrossRuntimeCall>>>,
    results: Arc<RwLock<HashMap<[u8; 32], CrossVMResult>>>,
}

#[derive(Debug, Clone)]
pub struct CrossVMResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub source_runtime: RuntimeType,
    pub target_runtime: RuntimeType,
    pub logs: Vec<CrossVMLog>,
}

#[derive(Debug, Clone)]
pub struct CrossVMLog {
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
    pub runtime: RuntimeType,
}

impl CrossVMCallManager {
    pub fn new(state: Arc<DualStateEngine>) -> Self {
        Self {
            state,
            call_stack: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn execute_cross_call(&self, call: CrossRuntimeCall) -> CrossVMResult {
        self.call_stack.write().unwrap().push(call.clone());
        
        let result = match (&call.source_runtime, &call.target_runtime) {
            (RuntimeType::EVM, RuntimeType::WASM) => self.evm_to_wasm_call(&call),
            (RuntimeType::WASM, RuntimeType::EVM) => self.wasm_to_evm_call(&call),
            (RuntimeType::EVM, RuntimeType::EVM) => self.evm_internal_call(&call),
            (RuntimeType::WASM, RuntimeType::WASM) => self.wasm_internal_call(&call),
            _ => CrossVMResult {
                success: false,
                return_data: b"Unsupported cross-runtime call".to_vec(),
                gas_used: 0,
                source_runtime: call.source_runtime.clone(),
                target_runtime: call.target_runtime.clone(),
                logs: vec![],
            },
        };
        
        self.call_stack.write().unwrap().pop();
        result
    }
    
    fn evm_to_wasm_call(&self, call: &CrossRuntimeCall) -> CrossVMResult {
        let abi_translated = self.translate_evm_to_wasm_abi(&call.params);
        
        let wasm_result = self.execute_wasm_method(
            &call.target_contract,
            &call.method,
            &abi_translated,
            call.gas_budget,
        );
        
        let return_data = self.translate_wasm_to_evm_abi(&wasm_result.return_data);
        
        CrossVMResult {
            success: wasm_result.success,
            return_data,
            gas_used: wasm_result.gas_used,
            source_runtime: RuntimeType::EVM,
            target_runtime: RuntimeType::WASM,
            logs: wasm_result.logs,
        }
    }
    
    fn wasm_to_evm_call(&self, call: &CrossRuntimeCall) -> CrossVMResult {
        let abi_translated = self.translate_wasm_to_evm_abi(&call.params);
        
        let evm_result = self.execute_evm_method(
            &call.target_contract,
            &call.method,
            &abi_translated,
            call.gas_budget,
        );
        
        let return_data = self.translate_evm_to_wasm_abi(&evm_result.return_data);
        
        CrossVMResult {
            success: evm_result.success,
            return_data,
            gas_used: evm_result.gas_used,
            source_runtime: RuntimeType::WASM,
            target_runtime: RuntimeType::EVM,
            logs: evm_result.logs,
        }
    }
    
    fn evm_internal_call(&self, call: &CrossRuntimeCall) -> CrossVMResult {
        self.execute_evm_method(&call.target_contract, &call.method, &call.params, call.gas_budget)
    }
    
    fn wasm_internal_call(&self, call: &CrossRuntimeCall) -> CrossVMResult {
        self.execute_wasm_method(&call.target_contract, &call.method, &call.params, call.gas_budget)
    }
    
    fn translate_evm_to_wasm_abi(&self, evm_data: &[u8]) -> Vec<u8> {
        evm_data.to_vec()
    }
    
    fn translate_wasm_to_evm_abi(&self, wasm_data: &[u8]) -> Vec<u8> {
        wasm_data.to_vec()
    }
    
    fn execute_evm_method(&self, _target: &DualAddress, method: &str, _params: &[u8], gas_budget: u64) -> CrossVMResult {
        CrossVMResult {
            success: true,
            return_data: format!("EVM:{}", method).into_bytes(),
            gas_used: gas_budget / 10,
            source_runtime: RuntimeType::EVM,
            target_runtime: RuntimeType::EVM,
            logs: vec![],
        }
    }
    
    fn execute_wasm_method(&self, _target: &DualAddress, method: &str, _params: &[u8], gas_budget: u64) -> CrossVMResult {
        CrossVMResult {
            success: true,
            return_data: format!("WASM:{}", method).into_bytes(),
            gas_used: gas_budget / 10,
            source_runtime: RuntimeType::WASM,
            target_runtime: RuntimeType::WASM,
            logs: vec![],
        }
    }
}

pub struct AIRuntimePlanner {
    historical_metrics: Arc<RwLock<Vec<RuntimeMetrics>>>,
    routing_policy: Arc<RwLock<RoutingPolicy>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    pub runtime: RuntimeType,
    pub avg_gas_cost: f64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RoutingPolicy {
    pub prefer_wasm_for_ai: bool,
    pub prefer_evm_for_defi: bool,
    pub gas_optimization_enabled: bool,
    pub quantum_security_required: bool,
    pub max_cross_runtime_depth: u8,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            prefer_wasm_for_ai: true,
            prefer_evm_for_defi: true,
            gas_optimization_enabled: true,
            quantum_security_required: false,
            max_cross_runtime_depth: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub recommended_runtime: RuntimeType,
    pub confidence: f64,
    pub reasoning: String,
    pub fallback_runtime: RuntimeType,
    pub estimated_gas: u64,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Standard,
    Enhanced,
    QuantumSafe,
    Maximum,
}

impl AIRuntimePlanner {
    pub fn new() -> Self {
        Self {
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            routing_policy: Arc::new(RwLock::new(RoutingPolicy::default())),
        }
    }
    
    pub fn plan_execution(&self, tx: &UnifiedTransaction) -> RoutingDecision {
        let policy = self.routing_policy.read().unwrap();
        
        if let Some(hint) = &tx.runtime_hint {
            return RoutingDecision {
                recommended_runtime: hint.clone(),
                confidence: 1.0,
                reasoning: "User specified runtime hint".to_string(),
                fallback_runtime: RuntimeType::EVM,
                estimated_gas: tx.gas_limit,
                security_level: self.determine_security_level(&tx.signature),
            };
        }
        
        let tx_type = self.analyze_transaction_type(tx);
        let metrics = self.historical_metrics.read().unwrap();
        
        let (runtime, confidence, reasoning) = match tx_type.as_str() {
            "ai_computation" | "model_inference" | "ml_training" => {
                if policy.prefer_wasm_for_ai {
                    (RuntimeType::WASM, 0.92, "AI computation optimized for WASM runtime".to_string())
                } else {
                    (RuntimeType::EVM, 0.75, "Using EVM for AI (policy override)".to_string())
                }
            },
            "token_transfer" | "defi_swap" | "liquidity" => {
                if policy.prefer_evm_for_defi {
                    (RuntimeType::EVM, 0.95, "DeFi operation optimized for EVM compatibility".to_string())
                } else {
                    (RuntimeType::WASM, 0.80, "Using WASM for DeFi (policy override)".to_string())
                }
            },
            "cross_runtime" | "hybrid" => {
                (RuntimeType::Hybrid, 0.88, "Cross-runtime operation requires hybrid execution".to_string())
            },
            "governance" | "voting" => {
                (RuntimeType::AIOptimized, 0.90, "Governance optimized with AI verification".to_string())
            },
            _ => {
                let avg_evm_gas = self.get_avg_metric(&metrics, RuntimeType::EVM);
                let avg_wasm_gas = self.get_avg_metric(&metrics, RuntimeType::WASM);
                
                if avg_wasm_gas < avg_evm_gas && policy.gas_optimization_enabled {
                    (RuntimeType::WASM, 0.70, "WASM selected for gas optimization".to_string())
                } else {
                    (RuntimeType::EVM, 0.75, "EVM selected as default".to_string())
                }
            }
        };
        
        let security_level = if tx.signature.signature_mode == SignatureMode::Hybrid {
            SecurityLevel::QuantumSafe
        } else if tx.signature.dilithium_sig.is_some() {
            SecurityLevel::Enhanced
        } else {
            SecurityLevel::Standard
        };
        
        RoutingDecision {
            recommended_runtime: runtime,
            confidence,
            reasoning,
            fallback_runtime: RuntimeType::EVM,
            estimated_gas: self.estimate_gas(tx, &metrics),
            security_level,
        }
    }
    
    fn analyze_transaction_type(&self, tx: &UnifiedTransaction) -> String {
        if tx.data.len() < 4 {
            return "simple_transfer".to_string();
        }
        
        let selector = &tx.data[0..4];
        
        match selector {
            [0xa9, 0x05, 0x9c, 0xbb] => "token_transfer".to_string(),
            [0x38, 0xed, 0x17, 0x39] => "defi_swap".to_string(),
            [0xe8, 0xe3, 0x37, 0x00] => "liquidity".to_string(),
            [0x01, 0x02, 0x03, 0x04] => "ai_computation".to_string(),
            [0x05, 0x06, 0x07, 0x08] => "model_inference".to_string(),
            [0x09, 0x0a, 0x0b, 0x0c] => "ml_training".to_string(),
            [0x0d, 0x0e, 0x0f, 0x10] => "governance".to_string(),
            _ => {
                if !tx.cross_runtime_calls.is_empty() {
                    "cross_runtime".to_string()
                } else {
                    "general".to_string()
                }
            }
        }
    }
    
    fn determine_security_level(&self, sig: &DualSignature) -> SecurityLevel {
        match sig.signature_mode {
            SignatureMode::Hybrid => SecurityLevel::Maximum,
            SignatureMode::QuantumOnly => SecurityLevel::QuantumSafe,
            SignatureMode::EVMOnly => SecurityLevel::Standard,
        }
    }
    
    fn get_avg_metric(&self, metrics: &[RuntimeMetrics], runtime: RuntimeType) -> f64 {
        let filtered: Vec<_> = metrics.iter().filter(|m| m.runtime == runtime).collect();
        if filtered.is_empty() {
            return 100000.0;
        }
        filtered.iter().map(|m| m.avg_gas_cost).sum::<f64>() / filtered.len() as f64
    }
    
    fn estimate_gas(&self, tx: &UnifiedTransaction, metrics: &[RuntimeMetrics]) -> u64 {
        let base_gas = 21000u64;
        let data_gas = tx.data.len() as u64 * 16;
        let cross_call_gas = tx.cross_runtime_calls.len() as u64 * 50000;
        
        base_gas + data_gas + cross_call_gas
    }
    
    pub fn record_metrics(&self, metrics: RuntimeMetrics) {
        let mut hist = self.historical_metrics.write().unwrap();
        hist.push(metrics);
        if hist.len() > 10000 {
            hist.remove(0);
        }
    }
    
    pub fn update_policy(&self, policy: RoutingPolicy) {
        *self.routing_policy.write().unwrap() = policy;
    }
}

pub struct UnifiedGasModel {
    base_evm_multiplier: f64,
    base_wasm_multiplier: f64,
    cross_runtime_overhead: u64,
    ai_discount_rate: f64,
}

impl UnifiedGasModel {
    pub fn new() -> Self {
        Self {
            base_evm_multiplier: 1.0,
            base_wasm_multiplier: 0.8,
            cross_runtime_overhead: 25000,
            ai_discount_rate: 0.15,
        }
    }
    
    pub fn calculate_gas(&self, tx: &UnifiedTransaction, runtime: &RuntimeType) -> u64 {
        let base_gas = 21000u64;
        let data_gas = tx.data.len() as u64 * 16;
        
        let multiplier = match runtime {
            RuntimeType::EVM => self.base_evm_multiplier,
            RuntimeType::WASM => self.base_wasm_multiplier,
            RuntimeType::Hybrid => (self.base_evm_multiplier + self.base_wasm_multiplier) / 2.0,
            RuntimeType::AIOptimized => self.base_wasm_multiplier * (1.0 - self.ai_discount_rate),
        };
        
        let cross_runtime_gas = tx.cross_runtime_calls.len() as u64 * self.cross_runtime_overhead;
        
        let total = (base_gas + data_gas) as f64 * multiplier + cross_runtime_gas as f64;
        
        total as u64
    }
    
    pub fn estimate_cost(&self, gas: u64, gas_price: u64) -> u128 {
        gas as u128 * gas_price as u128
    }
}

pub struct NeoNetUnifiedFabric {
    pub state_engine: Arc<DualStateEngine>,
    pub cross_vm_manager: Arc<CrossVMCallManager>,
    pub ai_planner: Arc<AIRuntimePlanner>,
    pub gas_model: Arc<UnifiedGasModel>,
}

impl NeoNetUnifiedFabric {
    pub fn new() -> Self {
        let state = Arc::new(DualStateEngine::new());
        let cross_vm = Arc::new(CrossVMCallManager::new(state.clone()));
        
        Self {
            state_engine: state,
            cross_vm_manager: cross_vm,
            ai_planner: Arc::new(AIRuntimePlanner::new()),
            gas_model: Arc::new(UnifiedGasModel::new()),
        }
    }
    
    pub fn execute_transaction(&self, tx: UnifiedTransaction) -> ExecutionResult {
        let routing = self.ai_planner.plan_execution(&tx);
        let gas_estimate = self.gas_model.calculate_gas(&tx, &routing.recommended_runtime);
        
        if gas_estimate > tx.gas_limit {
            return ExecutionResult {
                success: false,
                gas_used: 0,
                return_data: b"Insufficient gas".to_vec(),
                logs: vec![],
                state_changes: vec![],
                runtime_used: routing.recommended_runtime,
                cross_runtime_results: vec![],
            };
        }
        
        let result = match routing.recommended_runtime {
            RuntimeType::EVM => self.execute_evm(tx.clone()),
            RuntimeType::WASM => self.execute_wasm(tx.clone()),
            RuntimeType::Hybrid => self.execute_hybrid(tx.clone()),
            RuntimeType::AIOptimized => self.execute_ai_optimized(tx.clone()),
        };
        
        self.ai_planner.record_metrics(RuntimeMetrics {
            runtime: routing.recommended_runtime.clone(),
            avg_gas_cost: result.gas_used as f64,
            avg_execution_time_ms: 10.0,
            success_rate: if result.success { 1.0 } else { 0.0 },
            timestamp: tx.timestamp,
        });
        
        result
    }
    
    fn execute_evm(&self, tx: UnifiedTransaction) -> ExecutionResult {
        let gas_used = self.gas_model.calculate_gas(&tx, &RuntimeType::EVM);
        
        if let Some(to) = &tx.to {
            self.state_engine.update_balance(&tx.from.account_id, 0);
        }
        
        ExecutionResult {
            success: true,
            gas_used,
            return_data: vec![],
            logs: vec![],
            state_changes: vec![],
            runtime_used: RuntimeType::EVM,
            cross_runtime_results: vec![],
        }
    }
    
    fn execute_wasm(&self, tx: UnifiedTransaction) -> ExecutionResult {
        let gas_used = self.gas_model.calculate_gas(&tx, &RuntimeType::WASM);
        
        ExecutionResult {
            success: true,
            gas_used,
            return_data: vec![],
            logs: vec![],
            state_changes: vec![],
            runtime_used: RuntimeType::WASM,
            cross_runtime_results: vec![],
        }
    }
    
    fn execute_hybrid(&self, tx: UnifiedTransaction) -> ExecutionResult {
        let mut cross_results = vec![];
        
        for call in &tx.cross_runtime_calls {
            let result = self.cross_vm_manager.execute_cross_call(call.clone());
            cross_results.push(result);
        }
        
        let gas_used = self.gas_model.calculate_gas(&tx, &RuntimeType::Hybrid);
        
        ExecutionResult {
            success: true,
            gas_used,
            return_data: vec![],
            logs: vec![],
            state_changes: vec![],
            runtime_used: RuntimeType::Hybrid,
            cross_runtime_results: cross_results,
        }
    }
    
    fn execute_ai_optimized(&self, tx: UnifiedTransaction) -> ExecutionResult {
        let gas_used = self.gas_model.calculate_gas(&tx, &RuntimeType::AIOptimized);
        
        ExecutionResult {
            success: true,
            gas_used,
            return_data: b"AI_OPTIMIZED".to_vec(),
            logs: vec![],
            state_changes: vec![],
            runtime_used: RuntimeType::AIOptimized,
            cross_runtime_results: vec![],
        }
    }
    
    pub fn commit_block(&self, height: u64) -> DualStateNode {
        self.state_engine.commit(height)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub logs: Vec<CrossVMLog>,
    pub state_changes: Vec<StateChange>,
    pub runtime_used: RuntimeType,
    pub cross_runtime_results: Vec<CrossVMResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dual_address_creation() {
        let evm_addr = [1u8; 20];
        let dual = DualAddress::from_evm(evm_addr);
        
        assert!(dual.neo_address.starts_with("neo1"));
        assert_eq!(dual.evm_address, evm_addr);
    }
    
    #[test]
    fn test_unified_fabric_execution() {
        let fabric = NeoNetUnifiedFabric::new();
        
        let tx = UnifiedTransaction {
            tx_hash: [0u8; 32],
            from: DualAddress::from_evm([1u8; 20]),
            to: Some(DualAddress::from_evm([2u8; 20])),
            value: 1000,
            gas_limit: 100000,
            gas_price: 1,
            nonce: 0,
            data: vec![],
            signature: DualSignature {
                ecdsa_sig: Some(vec![1, 2, 3]),
                dilithium_sig: None,
                signature_mode: SignatureMode::EVMOnly,
            },
            runtime_hint: None,
            cross_runtime_calls: vec![],
            timestamp: 12345,
        };
        
        let result = fabric.execute_transaction(tx);
        assert!(result.success);
    }
}
