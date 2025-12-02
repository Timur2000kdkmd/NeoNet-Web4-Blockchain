use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only owner can perform this action")]
    Unauthorized {},

    #[error("Model not found: {model_id}")]
    ModelNotFound { model_id: String },

    #[error("Model already exists: {model_id}")]
    ModelAlreadyExists { model_id: String },

    #[error("Invalid model hash format")]
    InvalidModelHash {},

    #[error("Validator not registered: {address}")]
    ValidatorNotRegistered { address: String },

    #[error("Insufficient stake for validation")]
    InsufficientStake {},

    #[error("Invalid quantum signature")]
    InvalidQuantumSignature {},

    #[error("Model version conflict")]
    VersionConflict {},
}
