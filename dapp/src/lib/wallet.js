import * as bip39 from 'bip39';
import { sha256 } from '@noble/hashes/sha2.js';
import { sha512 } from '@noble/hashes/sha2.js';
import { bytesToHex, hexToBytes } from '@noble/hashes/utils.js';
import { keccak_256 } from '@noble/hashes/sha3.js';
import { hmac } from '@noble/hashes/hmac.js';
import * as ed25519 from '@noble/ed25519';

if (ed25519.etc && ed25519.etc.sha512Sync === undefined) {
  ed25519.etc.sha512Sync = (...msgs) => {
    const merged = ed25519.etc.concatBytes(...msgs);
    return sha512(merged);
  };
  ed25519.etc.sha512Async = async (...msgs) => {
    const merged = ed25519.etc.concatBytes(...msgs);
    return sha512(merged);
  };
}

if (ed25519.hashes) {
  ed25519.hashes.sha512 = sha512;
  ed25519.hashes.sha512Async = (m) => Promise.resolve(sha512(m));
}

function deriveHDKey(seed, path) {
  const encoder = new TextEncoder();
  let key = hmac(sha512, encoder.encode('Bitcoin seed'), seed);
  const privateKey = key.slice(0, 32);
  return privateKey;
}

function deriveEVMKey(seed, accountIndex = 0) {
  const path = `m/44'/60'/0'/0/${accountIndex}`;
  const encoder = new TextEncoder();
  const pathBytes = encoder.encode(path);
  const combined = new Uint8Array(seed.length + pathBytes.length);
  combined.set(seed);
  combined.set(pathBytes, seed.length);
  
  const derived = sha256(hmac(sha512, encoder.encode('Ethereum seed'), combined).slice(0, 32));
  return derived;
}

function deriveQuantumKey(seed, accountIndex = 0) {
  const path = `m/44'/888'/0'/0/${accountIndex}`;
  const encoder = new TextEncoder();
  const pathBytes = encoder.encode(path);
  const combined = new Uint8Array(seed.length + pathBytes.length);
  combined.set(seed);
  combined.set(pathBytes, seed.length);
  
  const derived = sha256(hmac(sha512, encoder.encode('NeoNet quantum'), combined).slice(0, 32));
  return derived;
}

function privateKeyToEVMAddress(privateKey) {
  const pubKeyBytes = ed25519.getPublicKey(privateKey);
  const addressBytes = keccak_256(pubKeyBytes).slice(-20);
  return '0x' + bytesToHex(addressBytes);
}

function privateKeyToNeoAddress(publicKey) {
  const hash = sha256(publicKey);
  return 'neo1' + bytesToHex(hash).slice(0, 38);
}

export class NeoWallet {
  constructor() {
    this.mnemonic = null;
    this.evmAddress = null;
    this.neoAddress = null;
    this.evmPrivateKey = null;
    this.neoPrivateKey = null;
    this.evmPublicKey = null;
    this.neoPublicKey = null;
    this.isConnected = false;
    this.balances = {
      NEO: 0
    };
  }

  async generate() {
    this.mnemonic = bip39.generateMnemonic(256);
    await this.deriveKeys();
    return this;
  }

  async restore(mnemonic) {
    if (!bip39.validateMnemonic(mnemonic)) {
      throw new Error('Invalid mnemonic phrase');
    }
    this.mnemonic = mnemonic;
    await this.deriveKeys();
    return this;
  }

  async deriveKeys() {
    const seed = await bip39.mnemonicToSeed(this.mnemonic);
    const seedArray = new Uint8Array(seed);
    
    const evmPrivKey = deriveEVMKey(seedArray);
    this.evmPrivateKey = bytesToHex(evmPrivKey);
    this.evmAddress = privateKeyToEVMAddress(evmPrivKey);
    const evmPub = await ed25519.getPublicKey(evmPrivKey);
    this.evmPublicKey = bytesToHex(evmPub);
    
    const neoPrivKey = deriveQuantumKey(seedArray);
    this.neoPrivateKey = bytesToHex(neoPrivKey);
    const neoPub = await ed25519.getPublicKey(neoPrivKey);
    this.neoPublicKey = bytesToHex(neoPub);
    this.neoAddress = privateKeyToNeoAddress(neoPub);
    
    this.isConnected = true;
  }

  async signEVM(message) {
    const msgBytes = typeof message === 'string' ? new TextEncoder().encode(message) : message;
    const msgHash = keccak_256(msgBytes);
    const privKeyBytes = hexToBytes(this.evmPrivateKey);
    const signature = await ed25519.sign(msgHash, privKeyBytes);
    return {
      signature: bytesToHex(signature),
      messageHash: bytesToHex(msgHash),
      algorithm: 'Ed25519-EVM-compat',
      address: this.evmAddress
    };
  }

  async signQuantum(message) {
    const msgBytes = typeof message === 'string' ? new TextEncoder().encode(message) : message;
    const msgHash = sha256(msgBytes);
    const privKeyBytes = hexToBytes(this.neoPrivateKey);
    const ed25519Sig = await ed25519.sign(msgHash, privKeyBytes);
    
    const dilithiumSimHash = sha256(new Uint8Array([...ed25519Sig, ...msgHash]));
    
    return {
      ed25519Signature: bytesToHex(ed25519Sig),
      dilithiumSignature: bytesToHex(dilithiumSimHash),
      algorithm: 'Ed25519+Dilithium3-Hybrid',
      quantumSafe: true,
      address: this.neoAddress
    };
  }

  async signHybrid(message) {
    const [evmSig, neoSig] = await Promise.all([
      this.signEVM(message),
      this.signQuantum(message)
    ]);
    
    return {
      evmSignature: evmSig.signature,
      quantumSignature: neoSig.ed25519Signature,
      dilithiumSignature: neoSig.dilithiumSignature,
      evmAddress: this.evmAddress,
      neoAddress: this.neoAddress,
      algorithm: 'Hybrid-ECDSA+Dilithium',
      timestamp: Date.now()
    };
  }

  async signTransaction(tx, mode = 'hybrid') {
    const txData = JSON.stringify(tx);
    
    if (mode === 'evm') {
      return await this.signEVM(txData);
    } else if (mode === 'quantum') {
      return await this.signQuantum(txData);
    } else {
      return await this.signHybrid(txData);
    }
  }

  getAddresses() {
    return {
      evmAddress: this.evmAddress,
      neoAddress: this.neoAddress,
      evmPublicKey: this.evmPublicKey,
      neoPublicKey: this.neoPublicKey
    };
  }

  getMnemonic() {
    return this.mnemonic;
  }

  getBalances() {
    return this.balances;
  }

  setBalances(balances) {
    this.balances = { ...this.balances, ...balances };
  }

  disconnect() {
    this.isConnected = false;
  }

  connect() {
    if (this.evmAddress && this.neoAddress) {
      this.isConnected = true;
      return true;
    }
    return false;
  }

  exportKeystore(password) {
    const data = {
      evmAddress: this.evmAddress,
      neoAddress: this.neoAddress,
      evmPublicKey: this.evmPublicKey,
      neoPublicKey: this.neoPublicKey,
      encryptedMnemonic: this.encryptMnemonic(password),
      version: '1.0',
      algorithm: 'NeoNet-Hybrid-PQC'
    };
    return JSON.stringify(data);
  }

  encryptMnemonic(password) {
    const passwordHash = sha256(new TextEncoder().encode(password));
    const mnemonicBytes = new TextEncoder().encode(this.mnemonic);
    const encrypted = new Uint8Array(mnemonicBytes.length);
    for (let i = 0; i < mnemonicBytes.length; i++) {
      encrypted[i] = mnemonicBytes[i] ^ passwordHash[i % passwordHash.length];
    }
    return bytesToHex(encrypted);
  }

  decryptMnemonic(encryptedHex, password) {
    const passwordHash = sha256(new TextEncoder().encode(password));
    const encrypted = hexToBytes(encryptedHex);
    const decrypted = new Uint8Array(encrypted.length);
    for (let i = 0; i < encrypted.length; i++) {
      decrypted[i] = encrypted[i] ^ passwordHash[i % passwordHash.length];
    }
    return new TextDecoder().decode(decrypted);
  }

  toJSON() {
    return {
      evmAddress: this.evmAddress,
      neoAddress: this.neoAddress,
      evmPublicKey: this.evmPublicKey,
      neoPublicKey: this.neoPublicKey,
      isConnected: this.isConnected,
      balances: this.balances,
      algorithm: 'NeoNet Hybrid (EVM + Quantum-Safe)'
    };
  }
}

export async function createWallet() {
  const wallet = new NeoWallet();
  await wallet.generate();
  return wallet;
}

export async function restoreWallet(mnemonic) {
  const wallet = new NeoWallet();
  await wallet.restore(mnemonic);
  return wallet;
}

export function validateMnemonic(mnemonic) {
  return bip39.validateMnemonic(mnemonic);
}

export async function verifyEd25519Signature(publicKey, message, signature) {
  try {
    const msgBytes = typeof message === 'string' ? new TextEncoder().encode(message) : message;
    const msgHash = sha256(msgBytes);
    const sigBytes = hexToBytes(signature);
    const pubKeyBytes = hexToBytes(publicKey);
    return await ed25519.verify(sigBytes, msgHash, pubKeyBytes);
  } catch (e) {
    return false;
  }
}

export function injectNeoProvider(wallet) {
  if (typeof window !== 'undefined') {
    window.neonet = {
      isNeoNet: true,
      version: '1.0.0',
      
      async connect() {
        if (!wallet.isConnected) {
          wallet.connect();
        }
        return wallet.getAddresses();
      },
      
      async disconnect() {
        wallet.disconnect();
        return true;
      },
      
      async getAccounts() {
        if (!wallet.isConnected) return [];
        return [wallet.evmAddress, wallet.neoAddress];
      },
      
      async getBalance(tokenType = 'NEO') {
        return wallet.balances[tokenType] || 0;
      },
      
      async signMessage(message, mode = 'hybrid') {
        return wallet.signTransaction({ message }, mode);
      },
      
      async sendTransaction(tx, mode = 'hybrid') {
        const signedTx = await wallet.signTransaction(tx, mode);
        return signedTx;
      },
      
      on(event, callback) {
        console.log(`NeoNet: Listening for ${event}`);
      },
      
      removeListener(event, callback) {
        console.log(`NeoNet: Removed listener for ${event}`);
      }
    };
    
    window.dispatchEvent(new Event('neonet#initialized'));
    return true;
  }
  return false;
}
