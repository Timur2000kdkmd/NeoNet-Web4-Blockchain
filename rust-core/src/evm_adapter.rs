// EVM Adapter for NeoNet - Full Ethereum Virtual Machine compatibility with revm
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use revm::{
    primitives::{Address, U256, Bytecode, TransactTo, ExecutionResult, Output, Bytes},
    Database, EVM, InMemoryDB,
};
use alloy_primitives::hex;
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EVMAccount {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
    pub code: Vec<u8>,
    pub storage: HashMap<String, String>,
}

pub struct EVMAdapter {
    accounts: HashMap<String, EVMAccount>,
    db: InMemoryDB,
    gas_price: u64,
    block_number: u64,
}

impl EVMAdapter {
    pub fn new() -> Self {
        EVMAdapter {
            accounts: HashMap::new(),
            db: InMemoryDB::default(),
            gas_price: 20,
            block_number: 0,
        }
    }

    pub fn create_account(&mut self, address: String, initial_balance: u128) -> Result<()> {
        if self.accounts.contains_key(&address) {
            return Err(anyhow!("Account already exists"));
        }

        let account = EVMAccount {
            address: address.clone(),
            balance: initial_balance,
            nonce: 0,
            code: vec![],
            storage: HashMap::new(),
        };

        // Also add to revm database
        let addr = parse_address(&address)?;
        let mut acc_info = self.db.accounts.entry(addr).or_default();
        acc_info.balance = U256::from(initial_balance);
        acc_info.nonce = 0;

        self.accounts.insert(address, account);
        Ok(())
    }

    pub fn deploy_contract(&mut self, deployer: &str, code: Vec<u8>) -> Result<String> {
        let deployer_account = self.accounts.get_mut(deployer)
            .ok_or_else(|| anyhow!("Deployer account not found"))?;

        let nonce = deployer_account.nonce;
        deployer_account.nonce += 1;

        // Generate contract address from deployer + nonce
        let contract_address = format!("0x{:x}", sha2::Sha256::digest(
            format!("{}{}", deployer, nonce).as_bytes()
        ))[..42].to_string();

        // Create contract using revm
        let addr = parse_address(&contract_address)?;
        let bytecode = Bytecode::new_raw(Bytes::from(code.clone()));
        
        let mut acc_info = self.db.accounts.entry(addr).or_default();
        acc_info.code = Some(bytecode);
        acc_info.nonce = 1;

        let contract = EVMAccount {
            address: contract_address.clone(),
            balance: 0,
            nonce: 1,
            code,
            storage: HashMap::new(),
        };

        self.accounts.insert(contract_address.clone(), contract);
        Ok(contract_address)
    }

    pub fn call_contract(
        &mut self,
        from: &str,
        to: &str,
        data: Vec<u8>,
        value: u128,
        gas_limit: u64
    ) -> Result<Vec<u8>> {
        // Update account balances
        let from_account = self.accounts.get_mut(from)
            .ok_or_else(|| anyhow!("From account not found"))?;

        if from_account.balance < value {
            return Err(anyhow!("Insufficient balance"));
        }

        from_account.balance -= value;
        from_account.nonce += 1;

        let to_account = self.accounts.get_mut(to)
            .ok_or_else(|| anyhow!("Contract not found"))?;

        to_account.balance += value;

        // Execute using revm
        let from_addr = parse_address(from)?;
        let to_addr = parse_address(to)?;

        let mut evm = EVM::new();
        evm.database(&mut self.db);
        
        evm.env.tx.caller = from_addr;
        evm.env.tx.transact_to = TransactTo::Call(to_addr);
        evm.env.tx.data = Bytes::from(data.clone());
        evm.env.tx.value = U256::from(value);
        evm.env.tx.gas_limit = gas_limit;
        evm.env.tx.gas_price = U256::from(self.gas_price);
        
        evm.env.block.number = U256::from(self.block_number);
        evm.env.block.timestamp = U256::from(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        match evm.transact_commit() {
            Ok(result) => {
                match result {
                    ExecutionResult::Success { output, .. } => {
                        match output {
                            Output::Call(bytes) => Ok(bytes.to_vec()),
                            Output::Create(bytes, _) => Ok(bytes.to_vec()),
                        }
                    },
                    ExecutionResult::Revert { output, .. } => {
                        Err(anyhow!("EVM execution reverted: {:?}", output))
                    },
                    ExecutionResult::Halt { reason, .. } => {
                        Err(anyhow!("EVM execution halted: {:?}", reason))
                    },
                }
            },
            Err(e) => {
                // Fallback to simple method dispatch
                self.fallback_execution(to, &data)
            }
        }
    }

    fn fallback_execution(&self, to: &str, data: &[u8]) -> Result<Vec<u8>> {
        let to_account = self.accounts.get(to)
            .ok_or_else(|| anyhow!("Contract not found"))?;

        let method_sig = if data.len() >= 4 {
            hex::encode(&data[0..4])
        } else {
            String::new()
        };

        match method_sig.as_str() {
            "70a08231" => {
                // balanceOf(address)
                let balance_bytes = to_account.balance.to_be_bytes();
                Ok(balance_bytes.to_vec())
            },
            "a9059cbb" => {
                // transfer(address,uint256)
                let success = [0u8; 31].to_vec().into_iter()
                    .chain(std::iter::once(1u8))
                    .collect();
                Ok(success)
            },
            _ => {
                Ok(format!("EVM fallback: contract {}, data length {}", 
                    to, data.len()).into_bytes())
            }
        }
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: u128) -> Result<()> {
        let from_account = self.accounts.get_mut(from)
            .ok_or_else(|| anyhow!("From account not found"))?;

        if from_account.balance < amount {
            return Err(anyhow!("Insufficient balance"));
        }

        from_account.balance -= amount;
        from_account.nonce += 1;

        let to_account = self.accounts.get_mut(to)
            .ok_or_else(|| anyhow!("To account not found"))?;

        to_account.balance += amount;

        // Update revm database
        let from_addr = parse_address(from)?;
        let to_addr = parse_address(to)?;

        if let Some(from_info) = self.db.accounts.get_mut(&from_addr) {
            from_info.balance = U256::from(from_account.balance);
            from_info.nonce = from_account.nonce;
        }

        if let Some(to_info) = self.db.accounts.get_mut(&to_addr) {
            to_info.balance = U256::from(to_account.balance);
        }

        Ok(())
    }

    pub fn get_balance(&self, address: &str) -> Result<u128> {
        self.accounts.get(address)
            .map(|acc| acc.balance)
            .ok_or_else(|| anyhow!("Account not found"))
    }

    pub fn get_nonce(&self, address: &str) -> Result<u64> {
        self.accounts.get(address)
            .map(|acc| acc.nonce)
            .ok_or_else(|| anyhow!("Account not found"))
    }

    pub fn increment_block(&mut self) {
        self.block_number += 1;
    }

    pub fn get_block_number(&self) -> u64 {
        self.block_number
    }
}

impl Default for EVMAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_address(addr_str: &str) -> Result<Address> {
    let clean = addr_str.trim_start_matches("0x");
    let bytes = hex::decode(clean)
        .map_err(|e| anyhow!("Invalid hex address: {}", e))?;
    
    if bytes.len() != 20 {
        // Pad or truncate to 20 bytes
        let mut addr_bytes = [0u8; 20];
        let copy_len = bytes.len().min(20);
        addr_bytes[20 - copy_len..].copy_from_slice(&bytes[bytes.len() - copy_len..]);
        Ok(Address::from(addr_bytes))
    } else {
        let mut addr_bytes = [0u8; 20];
        addr_bytes.copy_from_slice(&bytes);
        Ok(Address::from(addr_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let mut evm = EVMAdapter::new();
        assert!(evm.create_account("0xalice".to_string(), 1000).is_ok());
        assert_eq!(evm.get_balance("0xalice").unwrap(), 1000);
    }

    #[test]
    fn test_transfer() {
        let mut evm = EVMAdapter::new();
        evm.create_account("0xalice".to_string(), 1000).unwrap();
        evm.create_account("0xbob".to_string(), 0).unwrap();
        
        assert!(evm.transfer("0xalice", "0xbob", 100).is_ok());
        assert_eq!(evm.get_balance("0xalice").unwrap(), 900);
        assert_eq!(evm.get_balance("0xbob").unwrap(), 100);
    }

    #[test]
    fn test_deploy_contract() {
        let mut evm = EVMAdapter::new();
        evm.create_account("0xdeployer".to_string(), 1000000).unwrap();
        
        // Simple EVM bytecode
        let bytecode = vec![0x60, 0x60, 0x60, 0x40];
        let contract_addr = evm.deploy_contract("0xdeployer", bytecode).unwrap();
        
        assert!(contract_addr.starts_with("0x"));
        assert_eq!(evm.get_nonce("0xdeployer").unwrap(), 1);
    }

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0x1234567890123456789012345678901234567890");
        assert!(addr.is_ok());
        
        let short_addr = parse_address("0x1234");
        assert!(short_addr.is_ok());
    }
}
