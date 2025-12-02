use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Addr, Order,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, ModelResponse, ModelsResponse, ValidatorResponse, ValidatorsResponse, ValidationResponse, ModelValidationsResponse, ValidatorStatsResponse, BridgeResponse, BridgesResponse, CrossRuntimeStateResponse};
use crate::state::{Config, AIModel, AIValidator, ValidationRecord, ValidationResult, CrossRuntimeBridge, BridgeType, ModelType, CONFIG, MODELS, VALIDATORS, VALIDATIONS, MODEL_COUNT, VALIDATOR_COUNT, CROSS_BRIDGES};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: info.sender.clone(),
        neo_token: deps.api.addr_validate(&msg.neo_token)?,
        min_stake_for_registration: msg.min_stake_for_registration,
        ai_validator_threshold: msg.ai_validator_threshold,
    };
    
    CONFIG.save(deps.storage, &config)?;
    MODEL_COUNT.save(deps.storage, &0u64)?;
    VALIDATOR_COUNT.save(deps.storage, &0u64)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("neo_token", msg.neo_token))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterModel { model_id, name, description, ipfs_hash, model_type, quantum_signature } => {
            execute_register_model(deps, env, info, model_id, name, description, ipfs_hash, model_type, quantum_signature)
        },
        ExecuteMsg::UpdateModel { model_id, name, description, ipfs_hash } => {
            execute_update_model(deps, env, info, model_id, name, description, ipfs_hash)
        },
        ExecuteMsg::DeactivateModel { model_id } => {
            execute_deactivate_model(deps, info, model_id)
        },
        ExecuteMsg::RegisterValidator { neo_address, quantum_key_hash } => {
            execute_register_validator(deps, env, info, neo_address, quantum_key_hash)
        },
        ExecuteMsg::ValidateModel { model_id, result, accuracy_score } => {
            execute_validate_model(deps, env, info, model_id, result, accuracy_score)
        },
        ExecuteMsg::ClaimRewards {} => {
            execute_claim_rewards(deps, info)
        },
        ExecuteMsg::UpdateValidatorStake {} => {
            execute_update_validator_stake(deps, info)
        },
        ExecuteMsg::RegisterCrossRuntimeBridge { bridge_id, evm_contract, bridge_type } => {
            execute_register_bridge(deps, env, info, bridge_id, evm_contract, bridge_type)
        },
        ExecuteMsg::CrossRuntimeCall { bridge_id, method, params } => {
            execute_cross_runtime_call(deps, env, info, bridge_id, method, params)
        },
        ExecuteMsg::UpdateConfig { min_stake_for_registration, ai_validator_threshold } => {
            execute_update_config(deps, info, min_stake_for_registration, ai_validator_threshold)
        },
    }
}

fn execute_register_model(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    model_id: String,
    name: String,
    description: String,
    ipfs_hash: String,
    model_type: ModelType,
    quantum_signature: Option<String>,
) -> Result<Response, ContractError> {
    if MODELS.has(deps.storage, &model_id) {
        return Err(ContractError::ModelAlreadyExists { model_id });
    }
    
    let model = AIModel {
        model_id: model_id.clone(),
        owner: info.sender.clone(),
        name: name.clone(),
        description,
        ipfs_hash,
        version: 1,
        accuracy_score: 0,
        total_validations: 0,
        successful_validations: 0,
        created_at: env.block.time.seconds(),
        updated_at: env.block.time.seconds(),
        is_active: true,
        model_type,
        quantum_signature,
    };
    
    MODELS.save(deps.storage, &model_id, &model)?;
    
    let count = MODEL_COUNT.load(deps.storage)?;
    MODEL_COUNT.save(deps.storage, &(count + 1))?;
    
    Ok(Response::new()
        .add_attribute("method", "register_model")
        .add_attribute("model_id", model_id)
        .add_attribute("name", name)
        .add_attribute("owner", info.sender))
}

fn execute_update_model(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    model_id: String,
    name: Option<String>,
    description: Option<String>,
    ipfs_hash: Option<String>,
) -> Result<Response, ContractError> {
    let mut model = MODELS.load(deps.storage, &model_id)
        .map_err(|_| ContractError::ModelNotFound { model_id: model_id.clone() })?;
    
    if model.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    
    if let Some(n) = name {
        model.name = n;
    }
    if let Some(d) = description {
        model.description = d;
    }
    if let Some(h) = ipfs_hash {
        model.ipfs_hash = h;
        model.version += 1;
    }
    model.updated_at = env.block.time.seconds();
    
    MODELS.save(deps.storage, &model_id, &model)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_model")
        .add_attribute("model_id", model_id)
        .add_attribute("version", model.version.to_string()))
}

fn execute_deactivate_model(
    deps: DepsMut,
    info: MessageInfo,
    model_id: String,
) -> Result<Response, ContractError> {
    let mut model = MODELS.load(deps.storage, &model_id)
        .map_err(|_| ContractError::ModelNotFound { model_id: model_id.clone() })?;
    
    let config = CONFIG.load(deps.storage)?;
    if model.owner != info.sender && config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    
    model.is_active = false;
    MODELS.save(deps.storage, &model_id, &model)?;
    
    Ok(Response::new()
        .add_attribute("method", "deactivate_model")
        .add_attribute("model_id", model_id))
}

fn execute_register_validator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    neo_address: String,
    quantum_key_hash: Option<String>,
) -> Result<Response, ContractError> {
    if VALIDATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }
    
    let validator = AIValidator {
        address: info.sender.clone(),
        neo_address: neo_address.clone(),
        stake_amount: 0,
        reputation_score: 50,
        validations_performed: 0,
        successful_validations: 0,
        is_active: true,
        registered_at: env.block.time.seconds(),
        last_validation_at: 0,
        quantum_key_hash,
    };
    
    VALIDATORS.save(deps.storage, &info.sender, &validator)?;
    
    let count = VALIDATOR_COUNT.load(deps.storage)?;
    VALIDATOR_COUNT.save(deps.storage, &(count + 1))?;
    
    Ok(Response::new()
        .add_attribute("method", "register_validator")
        .add_attribute("validator", info.sender)
        .add_attribute("neo_address", neo_address))
}

fn execute_validate_model(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    model_id: String,
    result: ValidationResult,
    accuracy_score: u64,
) -> Result<Response, ContractError> {
    let mut validator = VALIDATORS.load(deps.storage, &info.sender)
        .map_err(|_| ContractError::ValidatorNotRegistered { address: info.sender.to_string() })?;
    
    if !validator.is_active {
        return Err(ContractError::Unauthorized {});
    }
    
    let mut model = MODELS.load(deps.storage, &model_id)
        .map_err(|_| ContractError::ModelNotFound { model_id: model_id.clone() })?;
    
    let validation_id = format!("{}_{}", model_id, env.block.time.seconds());
    
    let is_success = matches!(result, ValidationResult::Approved);
    
    let validation = ValidationRecord {
        validation_id: validation_id.clone(),
        model_id: model_id.clone(),
        validator: info.sender.clone(),
        result: result.clone(),
        accuracy_score,
        gas_used: 0,
        timestamp: env.block.time.seconds(),
        quantum_verified: validator.quantum_key_hash.is_some(),
    };
    
    VALIDATIONS.save(deps.storage, &validation_id, &validation)?;
    
    model.total_validations += 1;
    if is_success {
        model.successful_validations += 1;
    }
    model.accuracy_score = (model.accuracy_score * (model.total_validations - 1) + accuracy_score) / model.total_validations;
    MODELS.save(deps.storage, &model_id, &model)?;
    
    validator.validations_performed += 1;
    if is_success {
        validator.successful_validations += 1;
    }
    validator.last_validation_at = env.block.time.seconds();
    validator.reputation_score = (validator.successful_validations * 100) / validator.validations_performed.max(1);
    VALIDATORS.save(deps.storage, &info.sender, &validator)?;
    
    Ok(Response::new()
        .add_attribute("method", "validate_model")
        .add_attribute("validation_id", validation_id)
        .add_attribute("model_id", model_id)
        .add_attribute("result", format!("{:?}", result)))
}

fn execute_claim_rewards(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let validator = VALIDATORS.load(deps.storage, &info.sender)
        .map_err(|_| ContractError::ValidatorNotRegistered { address: info.sender.to_string() })?;
    
    let rewards = validator.successful_validations as u128 * 100;
    
    Ok(Response::new()
        .add_attribute("method", "claim_rewards")
        .add_attribute("validator", info.sender)
        .add_attribute("rewards", rewards.to_string()))
}

fn execute_update_validator_stake(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut validator = VALIDATORS.load(deps.storage, &info.sender)
        .map_err(|_| ContractError::ValidatorNotRegistered { address: info.sender.to_string() })?;
    
    let stake = info.funds.iter()
        .find(|c| c.denom == "neo")
        .map(|c| c.amount.u128())
        .unwrap_or(0);
    
    validator.stake_amount += stake;
    VALIDATORS.save(deps.storage, &info.sender, &validator)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_stake")
        .add_attribute("validator", info.sender)
        .add_attribute("total_stake", validator.stake_amount.to_string()))
}

fn execute_register_bridge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bridge_id: String,
    evm_contract: String,
    bridge_type: BridgeType,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    
    let bridge = CrossRuntimeBridge {
        evm_contract: evm_contract.clone(),
        wasm_contract: env.contract.address.clone(),
        bridge_type: bridge_type.clone(),
        is_active: true,
    };
    
    CROSS_BRIDGES.save(deps.storage, &bridge_id, &bridge)?;
    
    Ok(Response::new()
        .add_attribute("method", "register_bridge")
        .add_attribute("bridge_id", bridge_id)
        .add_attribute("evm_contract", evm_contract)
        .add_attribute("bridge_type", format!("{:?}", bridge_type)))
}

fn execute_cross_runtime_call(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bridge_id: String,
    method: String,
    _params: Vec<u8>,
) -> Result<Response, ContractError> {
    let bridge = CROSS_BRIDGES.load(deps.storage, &bridge_id)
        .map_err(|_| ContractError::Unauthorized {})?;
    
    if !bridge.is_active {
        return Err(ContractError::Unauthorized {});
    }
    
    Ok(Response::new()
        .add_attribute("method", "cross_runtime_call")
        .add_attribute("bridge_id", bridge_id)
        .add_attribute("target_method", method)
        .add_attribute("caller", info.sender))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    min_stake_for_registration: Option<u128>,
    ai_validator_threshold: Option<u32>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    
    if let Some(stake) = min_stake_for_registration {
        config.min_stake_for_registration = stake;
    }
    if let Some(threshold) = ai_validator_threshold {
        config.ai_validator_threshold = threshold;
    }
    
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_config"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Model { model_id } => to_json_binary(&query_model(deps, model_id)?),
        QueryMsg::Models { start_after, limit } => to_json_binary(&query_models(deps, start_after, limit)?),
        QueryMsg::Validator { address } => to_json_binary(&query_validator(deps, address)?),
        QueryMsg::Validators { start_after, limit } => to_json_binary(&query_validators(deps, start_after, limit)?),
        QueryMsg::Validation { validation_id } => to_json_binary(&query_validation(deps, validation_id)?),
        QueryMsg::ModelValidations { model_id, limit } => to_json_binary(&query_model_validations(deps, model_id, limit)?),
        QueryMsg::ValidatorStats { address } => to_json_binary(&query_validator_stats(deps, address)?),
        QueryMsg::Bridge { bridge_id } => to_json_binary(&query_bridge(deps, bridge_id)?),
        QueryMsg::Bridges {} => to_json_binary(&query_bridges(deps)?),
        QueryMsg::CrossRuntimeState { bridge_id, key } => to_json_binary(&query_cross_runtime_state(deps, bridge_id, key)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let model_count = MODEL_COUNT.load(deps.storage)?;
    let validator_count = VALIDATOR_COUNT.load(deps.storage)?;
    
    Ok(ConfigResponse {
        owner: config.owner,
        neo_token: config.neo_token,
        min_stake_for_registration: config.min_stake_for_registration,
        ai_validator_threshold: config.ai_validator_threshold,
        total_models: model_count,
        total_validators: validator_count,
    })
}

fn query_model(deps: Deps, model_id: String) -> StdResult<ModelResponse> {
    let model = MODELS.load(deps.storage, &model_id)?;
    Ok(ModelResponse { model })
}

fn query_models(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<ModelsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.as_deref();
    
    let models: Vec<AIModel> = MODELS
        .range(deps.storage, start.map(cosmwasm_std::Bound::exclusive), None, Order::Ascending)
        .take(limit)
        .map(|r| r.map(|(_, m)| m))
        .collect::<StdResult<_>>()?;
    
    Ok(ModelsResponse { models })
}

fn query_validator(deps: Deps, address: String) -> StdResult<ValidatorResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let validator = VALIDATORS.load(deps.storage, &addr)?;
    Ok(ValidatorResponse { validator })
}

fn query_validators(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<ValidatorsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    
    let validators: Vec<AIValidator> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
        .map(|r| r.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;
    
    Ok(ValidatorsResponse { validators })
}

fn query_validation(deps: Deps, validation_id: String) -> StdResult<ValidationResponse> {
    let validation = VALIDATIONS.load(deps.storage, &validation_id)?;
    Ok(ValidationResponse {
        validation_id: validation.validation_id,
        model_id: validation.model_id,
        validator: validation.validator,
        result: validation.result,
        accuracy_score: validation.accuracy_score,
        timestamp: validation.timestamp,
    })
}

fn query_model_validations(deps: Deps, model_id: String, limit: Option<u32>) -> StdResult<ModelValidationsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    
    let validations: Vec<ValidationResponse> = VALIDATIONS
        .range(deps.storage, None, None, Order::Descending)
        .filter_map(|r| r.ok())
        .filter(|(_, v)| v.model_id == model_id)
        .take(limit)
        .map(|(_, v)| ValidationResponse {
            validation_id: v.validation_id,
            model_id: v.model_id,
            validator: v.validator,
            result: v.result,
            accuracy_score: v.accuracy_score,
            timestamp: v.timestamp,
        })
        .collect();
    
    Ok(ModelValidationsResponse { validations })
}

fn query_validator_stats(deps: Deps, address: String) -> StdResult<ValidatorStatsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let validator = VALIDATORS.load(deps.storage, &addr)?;
    
    Ok(ValidatorStatsResponse {
        total_validations: validator.validations_performed,
        successful_validations: validator.successful_validations,
        reputation_score: validator.reputation_score,
        pending_rewards: validator.successful_validations as u128 * 100,
    })
}

fn query_bridge(deps: Deps, bridge_id: String) -> StdResult<BridgeResponse> {
    let bridge = CROSS_BRIDGES.load(deps.storage, &bridge_id)?;
    Ok(BridgeResponse {
        bridge_id,
        evm_contract: bridge.evm_contract,
        wasm_contract: bridge.wasm_contract,
        bridge_type: bridge.bridge_type,
        is_active: bridge.is_active,
    })
}

fn query_bridges(deps: Deps) -> StdResult<BridgesResponse> {
    let bridges: Vec<BridgeResponse> = CROSS_BRIDGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| r.map(|(id, b)| BridgeResponse {
            bridge_id: id,
            evm_contract: b.evm_contract,
            wasm_contract: b.wasm_contract,
            bridge_type: b.bridge_type,
            is_active: b.is_active,
        }))
        .collect::<StdResult<_>>()?;
    
    Ok(BridgesResponse { bridges })
}

fn query_cross_runtime_state(deps: Deps, bridge_id: String, key: String) -> StdResult<CrossRuntimeStateResponse> {
    let _bridge = CROSS_BRIDGES.load(deps.storage, &bridge_id)?;
    
    Ok(CrossRuntimeStateResponse {
        key,
        evm_value: None,
        wasm_value: None,
        synced: true,
    })
}
