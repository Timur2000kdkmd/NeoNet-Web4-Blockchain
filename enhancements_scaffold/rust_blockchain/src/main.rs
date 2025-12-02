use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use warp::Filter;
use chrono::Utc;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub payload: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub hash: String,
    pub validator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending: Vec<Transaction>,
    pub validators: Vec<String>,
}

impl Blockchain {
    pub fn new(validators: Vec<String>) -> Self {
        let mut bc = Blockchain {
            chain: vec![],
            pending: vec![],
            validators,
        };
        bc.chain.push(bc.genesis());
        bc
    }

    pub fn genesis(&self) -> Block {
        Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: Utc::now().timestamp(),
            transactions: vec![],
            nonce: 0,
            hash: String::from("genesis_hash"),
            validator: String::from("genesis"),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn mine_block(&mut self, validator: String) -> Option<Block> {
        if !self.validators.contains(&validator) {
            return None;
        }
        let index = (self.chain.len()) as u64;
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let timestamp = Utc::now().timestamp();
        let transactions = self.pending.drain(..).collect::<Vec<_>>();
        // simple nonce and hash (NOT cryptographically secure) for scaffold
        let nonce = 0u64;
        let hash = format!("hash:{}:{}:{}", index, previous_hash, timestamp);
        let block = Block {
            index,
            previous_hash,
            timestamp,
            transactions,
            nonce,
            hash: hash.clone(),
            validator,
        };
        self.chain.push(block.clone());
        Some(block)
    }

    pub fn to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let s = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, s)
    }

    pub fn from_file(path: &str) -> Option<Blockchain> {
        if !Path::new(path).exists() {
            return None;
        }
        let s = fs::read_to_string(path).ok()?;
        serde_json::from_str(&s).ok()
    }
}

#[tokio::main]
async fn main() {
    // config: validators and persistence file
    let validators = vec![String::from("validator-1")];
    let persist_file = "neonet_chain.json";

    // load existing or create new
    let bc = if let Some(loaded) = Blockchain::from_file(persist_file) {
        loaded
    } else {
        Blockchain::new(validators.clone())
    };
    let state = Arc::new(Mutex::new(bc));

    // POST /tx -> submit transaction
    let state_filter = warp::any().map(move || Arc::clone(&state));
    let submit = warp::path("tx")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(|tx: Transaction, state: Arc<Mutex<Blockchain>>| async move {
            let mut s = state.lock().unwrap();
            s.add_transaction(tx);
            // persist
            let _ = s.to_file(persist_file);
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({"status":"ok"})))
        });

    // POST /mine -> mine a block with validator in JSON { "validator": "validator-1" }
    let mine = warp::path("mine")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(|body: serde_json::Value, state: Arc<Mutex<Blockchain>>| async move {
            let validator = body.get("validator").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mut s = state.lock().unwrap();
            match s.mine_block(validator.clone()) {
                Some(b) => {
                    let _ = s.to_file(persist_file);
                    Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({"status":"mined","block":b})))
                },
                None => Ok::<_, warp::Rejection>(warp::reply::with_status(warp::reply::json(&serde_json::json!({"error":"invalid validator"})), warp::http::StatusCode::UNAUTHORIZED))
            }
        });

    // GET /chain -> return full chain
    let get_chain = warp::path("chain")
        .and(warp::get())
        .and(state_filter.clone())
        .map(|state: Arc<Mutex<Blockchain>>| {
            let s = state.lock().unwrap();
            warp::reply::json(&*s)
        });

    // health
    let health = warp::path("health").and(warp::get()).map(|| warp::reply::json(&serde_json::json!({"status":"ok"})));

    let routes = submit.or(mine).or(get_chain).or(health);

    println!("Starting Rust blockchain HTTP API on 127.0.0.1:3030");
    warp::serve(routes).run(([127,0,0,1], 3030)).await;
}
