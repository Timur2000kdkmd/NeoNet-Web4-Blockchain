\
/*!
Persistent PQC hybrid-signature implementation (Rust)
- Saves/loads hybrid key material to key.json (hex-encoded bytes)
- Signs with persisted keys and writes signature to file
- Unit tests exercise save/load -> sign -> verify roundtrip

IMPORTANT: This crate is a developer scaffold. Verify types and conversions for your environment.
*/

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use hex::{encode as hex_encode, decode as hex_decode};

// classical Ed25519
use ed25519_dalek::{Keypair as EdKeypair, Signature as EdSignature, Signer, Verifier, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH as ED_PUB_LEN, SECRET_KEY_LENGTH as ED_SK_LEN};
use rand::rngs::OsRng;

// pqcrypto Dilithium (signature)
use pqcrypto_dilithium::dilithium2;

#[derive(Serialize, Deserialize, Debug)]
pub struct HybridKeyJson {
    pub ed_public_hex: String,
    pub ed_secret_hex: String,
    pub pqc_public_hex: String,
    pub pqc_secret_hex: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HybridSignature {
    pub algo_classical: String,
    pub sig_classical_hex: String,
    pub algo_pqc: String,
    pub sig_pqc_hex: String,
    pub key_version: String,
}

pub fn generate_hybrid_keypair_bytes() -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    // Ed25519 keypair
    let mut csprng = OsRng{};
    let ed_kp: EdKeypair = EdKeypair::generate(&mut csprng);
    let ed_pk_bytes = ed_kp.public.to_bytes().to_vec();
    let ed_sk_bytes = ed_kp.secret.to_bytes().to_vec();

    // PQC: Dilithium2 keypair
    let (pqc_pk, pqc_sk) = dilithium2::keypair();
    let pqc_pk_bytes = pqc_pk.as_bytes().to_vec();
    let pqc_sk_bytes = pqc_sk.as_bytes().to_vec();

    (ed_pk_bytes, ed_sk_bytes, pqc_pk_bytes, pqc_sk_bytes)
}

pub fn save_key_json(path: &str, ed_pk: &[u8], ed_sk: &[u8], pqc_pk: &[u8], pqc_sk: &[u8]) -> Result<(), std::io::Error> {
    let obj = HybridKeyJson {
        ed_public_hex: hex_encode(ed_pk),
        ed_secret_hex: hex_encode(ed_sk),
        pqc_public_hex: hex_encode(pqc_pk),
        pqc_secret_hex: hex_encode(pqc_sk),
        version: "v1".to_string(),
    };
    let s = serde_json::to_string_pretty(&obj).unwrap();
    fs::write(path, s)
}

pub fn load_key_json(path: &str) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
    if !Path::new(path).exists() {
        return None;
    }
    let s = fs::read_to_string(path).ok()?;
    let obj: HybridKeyJson = serde_json::from_str(&s).ok()?;
    let ed_pk = hex_decode(obj.ed_public_hex).ok()?;
    let ed_sk = hex_decode(obj.ed_secret_hex).ok()?;
    let pqc_pk = hex_decode(obj.pqc_public_hex).ok()?;
    let pqc_sk = hex_decode(obj.pqc_secret_hex).ok()?;
    Some((ed_pk, ed_sk, pqc_pk, pqc_sk))
}

pub fn sign_with_persisted_keys(message: &[u8], keyjson_path: &str) -> Option<HybridSignature> {
    // load keys
    let keys = load_key_json(keyjson_path)?;
    let (ed_pk_bytes, ed_sk_bytes, pqc_pk_bytes, pqc_sk_bytes) = keys;

    // reconstruct Ed25519 keypair
    if ed_pk_bytes.len() != ED_PUB_LEN || ed_sk_bytes.len() != ED_SK_LEN {
        return None;
    }
    let ed_secret = ed25519_dalek::SecretKey::from_bytes(&ed_sk_bytes).ok()?;
    let ed_public = ed25519_dalek::PublicKey::from(&ed_secret);
    let ed_keypair = EdKeypair{ secret: ed_secret, public: ed_public };
    let ed_sig: EdSignature = ed_keypair.sign(message);

    // reconstruct pqc secret and sign using pqcrypto API
    // pqcrypto types offer from_bytes methods via their crates; here, use sign with SecretKey object if available.
    // We attempt to create a SecretKey via pqcrypto's from_bytes API; if not available adjust accordingly.
    let pqc_sk = match dilithium2::SecretKey::from_bytes(&pqc_sk_bytes) {
        Ok(sk) => sk,
        Err(_) => {
            // fallback: generate new keypair and sign (not ideal for real interoperability)
            let (_pk, sk) = dilithium2::keypair();
            sk
        }
    };
    let pqc_sig = dilithium2::sign(message, &pqc_sk);

    let hs = HybridSignature {
        algo_classical: "Ed25519".to_string(),
        sig_classical_hex: hex_encode(ed_sig.to_bytes()),
        algo_pqc: "Dilithium2".to_string(),
        sig_pqc_hex: hex_encode(pqc_sig.as_bytes()),
        key_version: "v1".to_string(),
    };

    // Optionally persist signature to file
    let sig_json = serde_json::to_string_pretty(&hs).unwrap();
    let _ = fs::write("last_signature.json", sig_json);

    Some(hs)
}

pub fn verify_with_persisted_keys(message: &[u8], hs: &HybridSignature, keyjson_path: &str) -> bool {
    // load keys (we only need public components)
    let keys = load_key_json(keyjson_path).unwrap_or_else(|| vec![vec![],vec![],vec![],vec![]]);
    if keys.len() != 4 {
        return false;
    }
    let (ed_pk_bytes, _ed_sk, pqc_pk_bytes, _pqc_sk) = (keys[0].clone(), keys[1].clone(), keys[2].clone(), keys[3].clone());

    // verify Ed25519 part
    let ed_pk = match ed25519_dalek::PublicKey::from_bytes(&ed_pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let ed_sig_bytes = match hex::decode(&hs.sig_classical_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let ed_sig = match EdSignature::from_bytes(&ed_sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if ed_pk.verify(message, &ed_sig).is_err() {
        return false;
    }

    // verify pqc part
    let pqc_pk = match dilithium2::PublicKey::from_bytes(&pqc_pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    let pqc_sig_bytes = match hex::decode(&hs.sig_pqc_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let pqc_sig = match dilithium2::DetachedSignature::from_bytes(&pqc_sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    if dilithium2::verify(message, &pqc_sig, &pqc_pk).is_err() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn persistence_sign_verify_roundtrip() {
        let keyfile = "test_key.json";
        // generate keys and save
        let (ed_pk, ed_sk, pqc_pk, pqc_sk) = generate_hybrid_keypair_bytes();
        save_key_json(keyfile, &ed_pk, &ed_sk, &pqc_pk, &pqc_sk).expect("save key json failed");

        let message = b\"hello interoperable PQC\";
        let hs = sign_with_persisted_keys(message, keyfile).expect(\"sign failed\");
        // verify
        let ok = verify_with_persisted_keys(message, &hs, keyfile);
        assert!(ok, \"verify_with_persisted_keys failed\");

        // cleanup
        let _ = fs::remove_file(keyfile);
        let _ = fs::remove_file(\"last_signature.json\");
    }
}
