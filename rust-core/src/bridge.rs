use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub pub_key: String,
    pub signature: String
}

#[derive(Serialize, Deserialize)]
struct Request {
    cmd: String,
    data: Option<serde_json::Value>
}

pub struct ChainState {
    pub chain: Vec<Block>,
    pub keypair: SigningKey,
    pub path: String,
}

fn calculate_hash(b: &Block) -> String {
    use sha2::{Sha256, Digest};
    let record = format!("{}{}{}{}{}", b.index, b.timestamp, b.data, b.prev_hash, b.nonce);
    let mut hasher = Sha256::new();
    hasher.update(record.as_bytes());
    let res = hasher.finalize();
    hex::encode(res)
}

fn mine_block(mut b: Block, difficulty: usize) -> Block {
    let target = "0".repeat(difficulty);
    loop {
        b.nonce += 1;
        b.hash = calculate_hash(&b);
        if &b.hash[..difficulty] == target {
            return b;
        }
    }
}

fn load_or_create_chain(path: &str, keypair: &SigningKey) -> Vec<Block> {
    if Path::new(path).exists() {
        if let Ok(s) = fs::read_to_string(path) {
            if let Ok(arr) = serde_json::from_str::<Vec<Block>>(&s) {
                return arr;
            }
        }
    }
    // create genesis
    let mut g = Block {
        index: 0,
        timestamp: Utc::now().to_rfc3339(),
        data: "genesis".to_string(),
        prev_hash: "".to_string(),
        hash: "".to_string(),
        nonce: 0,
        pub_key: hex::encode(keypair.verifying_key().to_bytes()),
        signature: "".to_string(),
    };
    g = mine_block(g, 1);
    // sign genesis
    let sig: Signature = keypair.sign(g.hash.as_bytes());
    g.signature = hex::encode(sig.to_bytes());
    let arr = vec![g];
    // persist
    let _ = fs::write(path, serde_json::to_string_pretty(&arr).unwrap());
    arr
}

fn save_chain(path: &str, chain: &Vec<Block>) {
    let _ = fs::write(path, serde_json::to_string_pretty(chain).unwrap());
}

fn handle_request(req: Request, state: &mut ChainState) -> serde_json::Value {
    match req.cmd.as_str() {
        "commit_block" => {
            if let Some(d) = req.data {
                if let Some(hv) = d.get("hash").and_then(|v| v.as_str()) {
                    let _ = fs::write(format!("committed_{}.txt", hv), "committed");
                    return json!({"ok":true, "committed": hv});
                }
            }
            return json!({"ok":false});
        },
        "get_chain" => {
            json!({"ok": true, "chain": state.chain})
        },
        "submit_tx" => {
            // data should contain "data" and optionally "from"
            if let Some(d) = req.data {
                let data_str = d.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let latest = state.chain.last().unwrap();
                let mut newb = Block {
                    index: latest.index + 1,
                    timestamp: Utc::now().to_rfc3339(),
                    data: data_str,
                    prev_hash: latest.hash.clone(),
                    hash: "".to_string(),
                    nonce: 0,
                    pub_key: hex::encode(state.keypair.verifying_key().to_bytes()),
                    signature: "".to_string(),
                };
                newb = mine_block(newb, 1);
                // sign block hash with keypair
                let sig: Signature = state.keypair.sign(newb.hash.as_bytes());
                newb.signature = hex::encode(sig.to_bytes());
                state.chain.push(newb.clone());
                save_chain(&state.path, &state.chain);
                json!({"ok":true, "block": newb})
            } else {
                json!({"ok":false, "error":"no data"})
            }
        },
        "put_chain" => {
            if let Some(d) = req.data {
                if let Ok(arr) = serde_json::from_value::<Vec<Block>>(d) {
                    // simple replace if longer
                    if arr.len() > state.chain.len() {
                        state.chain = arr;
                        save_chain(&state.path, &state.chain);
                        return json!({"ok":true, "replaced": true});
                    } else {
                        return json!({"ok":true, "replaced": false});
                    }
                }
            }
            json!({"ok":false})
        },
        _ => json!({"ok":false, "error":"unknown command"})
    }
}

fn handle_stream(mut s: TcpStream, shared: Arc<Mutex<ChainState>>) {
    let mut buf = Vec::new();
    match s.read_to_end(&mut buf) {
        Ok(_) => {
            if buf.is_empty() { return; }
            if let Ok(txt) = String::from_utf8(buf) {
                if let Ok(req) = serde_json::from_str::<Request>(&txt) {
                    let mut st = shared.lock().unwrap();
                    let resp = handle_request(req, &mut *st);
                    let _ = s.write_all(resp.to_string().as_bytes());
                }
            }
        }
        Err(e) => {
            eprintln!("read err: {:?}", e);
        }
    }
}

pub fn start_bridge() {
    thread::spawn(|| {
        // prepare keypair
        let keypath = "rust_keys/node_priv.hex";
        let path = "rust_chain_store.json";
        let kp = if Path::new(keypath).exists() {
            if let Ok(s) = fs::read_to_string(keypath) {
                if let Ok(bytes) = hex::decode(s.trim()) {
                    if bytes.len() == 32 {
                        SigningKey::from_bytes(&bytes.try_into().unwrap())
                    } else {
                        SigningKey::new(OsRng)
                    }
                } else {
                    SigningKey::new(OsRng)
                }
            } else {
                SigningKey::new(OsRng)
            }
        } else {
            SigningKey::new(OsRng)
        };
        // if key file absent, write it
        if !Path::new("rust_keys").exists() {
            let _ = fs::create_dir("rust_keys");
        }
        let _ = fs::write("rust_keys/node_priv.hex", hex::encode(kp.to_bytes()));
        // load or create chain
        let chain = load_or_create_chain(path, &kp);
        let state = ChainState { chain, keypair: kp, path: path.to_string() };
        let shared = Arc::new(Mutex::new(state));
        if let Ok(listener) = TcpListener::bind("127.0.0.1:6000") {
            println!("rust bridge listening on 127.0.0.1:6000");
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {
                        let shared2 = shared.clone();
                        thread::spawn(move || {
                            handle_stream(s, shared2);
                        });
                    }
                    Err(e) => {
                        eprintln!("incoming err {:?}", e);
                    }
                }
            }
        } else {
            eprintln!("could not bind 127.0.0.1:6000");
        }
    });
}




// sled-based persistence helpers (simple key "chain" -> JSON)
fn sled_save(path: &str, chain: &Vec<Block>) {
    if let Ok(db) = sled::open(path) {
        let _ = db.insert("chain", serde_json::to_vec(chain).unwrap());
        let _ = db.flush();
    }
}

fn sled_load(path: &str) -> Option<Vec<Block>> {
    if let Ok(db) = sled::open(path) {
        if let Ok(Some(v)) = db.get("chain") {
            if let Ok(arr) = serde_json::from_slice::<Vec<Block>>(&v) {
                return Some(arr);
            }
        }
    }
    None
}
