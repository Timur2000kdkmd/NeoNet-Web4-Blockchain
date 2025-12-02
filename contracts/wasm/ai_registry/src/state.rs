use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub neo_token: Addr,
    pub min_stake_for_registration: u128,
    pub ai_validator_threshold: u32,
}

#[cw_serde]
pub struct AIModel {
    pub model_id: String,
    pub owner: Addr,
    pub name: String,
    pub description: String,
    pub ipfs_hash: String,
    pub version: u32,
    pub accuracy_score: u64,
    pub total_validations: u64,
    pub successful_validations: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
    pub model_type: ModelType,
    pub quantum_signature: Option<String>,
}

#[cw_serde]
pub enum ModelType {
    FraudDetection,
    GasOptimization,
    AnomalyDetection,
    ContractAudit,
    GovernanceAI,
    GeneralPurpose,
}

#[cw_serde]
pub struct AIValidator {
    pub address: Addr,
    pub neo_address: String,
    pub stake_amount: u128,
    pub reputation_score: u64,
    pub validations_performed: u64,
    pub successful_validations: u64,
    pub is_active: bool,
    pub registered_at: u64,
    pub last_validation_at: u64,
    pub quantum_key_hash: Option<String>,
}

#[cw_serde]
pub struct ValidationRecord {
    pub validation_id: String,
    pub model_id: String,
    pub validator: Addr,
    pub result: ValidationResult,
    pub accuracy_score: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub quantum_verified: bool,
}

#[cw_serde]
pub enum ValidationResult {
    Approved,
    Rejected,
    NeedsReview,
    Pending,
}

#[cw_serde]
pub struct CrossRuntimeBridge {
    pub evm_contract: String,
    pub wasm_contract: Addr,
    pub bridge_type: BridgeType,
    pub is_active: bool,
}

#[cw_serde]
pub enum BridgeType {
    TokenBridge,
    DataBridge,
    CallBridge,
    StateBridge,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const MODELS: Map<&str, AIModel> = Map::new("models");
pub const VALIDATORS: Map<&Addr, AIValidator> = Map::new("validators");
pub const VALIDATIONS: Map<&str, ValidationRecord> = Map::new("validations");
pub const MODEL_COUNT: Item<u64> = Item::new("model_count");
pub const VALIDATOR_COUNT: Item<u64> = Item::new("validator_count");
pub const CROSS_BRIDGES: Map<&str, CrossRuntimeBridge> = Map::new("bridges");
