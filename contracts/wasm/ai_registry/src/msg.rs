use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::{AIModel, AIValidator, ModelType, ValidationResult, BridgeType};

#[cw_serde]
pub struct InstantiateMsg {
    pub neo_token: String,
    pub min_stake_for_registration: u128,
    pub ai_validator_threshold: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterModel {
        model_id: String,
        name: String,
        description: String,
        ipfs_hash: String,
        model_type: ModelType,
        quantum_signature: Option<String>,
    },
    UpdateModel {
        model_id: String,
        name: Option<String>,
        description: Option<String>,
        ipfs_hash: Option<String>,
    },
    DeactivateModel {
        model_id: String,
    },
    RegisterValidator {
        neo_address: String,
        quantum_key_hash: Option<String>,
    },
    ValidateModel {
        model_id: String,
        result: ValidationResult,
        accuracy_score: u64,
    },
    ClaimRewards {},
    UpdateValidatorStake {},
    RegisterCrossRuntimeBridge {
        bridge_id: String,
        evm_contract: String,
        bridge_type: BridgeType,
    },
    CrossRuntimeCall {
        bridge_id: String,
        method: String,
        params: Vec<u8>,
    },
    UpdateConfig {
        min_stake_for_registration: Option<u128>,
        ai_validator_threshold: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    
    #[returns(ModelResponse)]
    Model { model_id: String },
    
    #[returns(ModelsResponse)]
    Models { start_after: Option<String>, limit: Option<u32> },
    
    #[returns(ValidatorResponse)]
    Validator { address: String },
    
    #[returns(ValidatorsResponse)]
    Validators { start_after: Option<String>, limit: Option<u32> },
    
    #[returns(ValidationResponse)]
    Validation { validation_id: String },
    
    #[returns(ModelValidationsResponse)]
    ModelValidations { model_id: String, limit: Option<u32> },
    
    #[returns(ValidatorStatsResponse)]
    ValidatorStats { address: String },
    
    #[returns(BridgeResponse)]
    Bridge { bridge_id: String },
    
    #[returns(BridgesResponse)]
    Bridges {},
    
    #[returns(CrossRuntimeStateResponse)]
    CrossRuntimeState { bridge_id: String, key: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub neo_token: Addr,
    pub min_stake_for_registration: u128,
    pub ai_validator_threshold: u32,
    pub total_models: u64,
    pub total_validators: u64,
}

#[cw_serde]
pub struct ModelResponse {
    pub model: AIModel,
}

#[cw_serde]
pub struct ModelsResponse {
    pub models: Vec<AIModel>,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub validator: AIValidator,
}

#[cw_serde]
pub struct ValidatorsResponse {
    pub validators: Vec<AIValidator>,
}

#[cw_serde]
pub struct ValidationResponse {
    pub validation_id: String,
    pub model_id: String,
    pub validator: Addr,
    pub result: ValidationResult,
    pub accuracy_score: u64,
    pub timestamp: u64,
}

#[cw_serde]
pub struct ModelValidationsResponse {
    pub validations: Vec<ValidationResponse>,
}

#[cw_serde]
pub struct ValidatorStatsResponse {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub reputation_score: u64,
    pub pending_rewards: u128,
}

#[cw_serde]
pub struct BridgeResponse {
    pub bridge_id: String,
    pub evm_contract: String,
    pub wasm_contract: Addr,
    pub bridge_type: BridgeType,
    pub is_active: bool,
}

#[cw_serde]
pub struct BridgesResponse {
    pub bridges: Vec<BridgeResponse>,
}

#[cw_serde]
pub struct CrossRuntimeStateResponse {
    pub key: String,
    pub evm_value: Option<Vec<u8>>,
    pub wasm_value: Option<Vec<u8>>,
    pub synced: bool,
}
