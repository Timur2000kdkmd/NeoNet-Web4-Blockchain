pub mod stake;
pub mod governance;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModelInfo {
    pub owner: String,
    pub name: String,
    pub ipfs_hash: String,
    pub version: String,
    pub metadata: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize)]
pub enum ExecuteMsg {
    RegisterModel { name: String, ipfs_hash: String, version: String, metadata: Option<String> },
    UpdateModel { id: u64, ipfs_hash: String, version: String },
}

#[derive(Serialize, Deserialize)]
pub enum QueryMsg {
    GetModel { id: u64 },
    ListModels {},
}

static MODELS: Map<u64, ModelInfo> = Map::new("models");
static NEXT_ID: Item<u64> = Item::new("next_id");

pub fn instantiate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InstantiateMsg) -> StdResult<Response> {
    NEXT_ID.save(_deps.storage, &1)?;
    Ok(Response::default())
}

pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::RegisterModel { name, ipfs_hash, version, metadata } => {
            let mut id = NEXT_ID.load(deps.storage)?;
            let model = ModelInfo {
                owner: info.sender.to_string(),
                name,
                ipfs_hash,
                version,
                metadata,
            };
            MODELS.save(deps.storage, id, &model)?;
            id += 1;
            NEXT_ID.save(deps.storage, &id)?;
            Ok(Response::new().set_data(to_binary(&id)?))
        },
        ExecuteMsg::UpdateModel { id, ipfs_hash, version } => {
            let mut model = MODELS.load(deps.storage, id)?;
            model.ipfs_hash = ipfs_hash;
            model.version = version;
            MODELS.save(deps.storage, id, &model)?;
            Ok(Response::default())
        }
    }
}

pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetModel { id } => to_binary(&MODELS.load(_deps.storage, id)?),
        QueryMsg::ListModels {} => {
            let mut models: Vec<ModelInfo> = vec![];
            let all_keys: Vec<u64> = MODELS
                .keys(_deps.storage, None, None, cosmwasm_std::Order::Ascending)
                .collect::<Result<Vec<_>, _>>()?;
            
            for key in all_keys {
                if let Ok(model) = MODELS.load(_deps.storage, key) {
                    models.push(model);
                }
            }
            to_binary(&models)
        }
    }
}
