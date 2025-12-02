package main

import (
	"crypto/ed25519"
	"encoding/hex"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"sync"
	"time"
	"strconv"
)

type Block struct {
	PubKey string `json:"pub_key"`
	Signature string `json:"signature"`
	Index     int    `json:"index"`
	Timestamp string `json:"timestamp"`
	Data      string `json:"data"`
	PrevHash  string `json:"prev_hash"`
	Hash      string `json:"hash"`
	Nonce     int    `json:"nonce"`
}

type Blockchain struct {
	chain []*Block
	mu    sync.RWMutex
}

func NewBlockchain() *Blockchain {
	return &Blockchain{chain: make([]*Block, 0)}
}

func (bc *Blockchain) CreateGenesis() {
	gen := &Block{
		Index:     0,
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Data:      "genesis",
		PrevHash:  "",
		Nonce:     0,
	}
	gen.Hash = calculateHash(gen)
	bc.chain = append(bc.chain, gen)
}

func (bc *Blockchain) Latest() *Block {
	bc.mu.RLock()
	defer bc.mu.RUnlock()
	if len(bc.chain) == 0 { return nil }
	return bc.chain[len(bc.chain)-1]
}

func calculateHash(b *Block) string {
	record := fmt.Sprintf("%d%s%s%s%d", b.Index, b.Timestamp, b.Data, b.PrevHash, b.Nonce)
	h := sha256.Sum256([]byte(record))
	return hex.EncodeToString(h[:])
}

// Simple proof-of-work: find nonce such that hash has prefix of difficulty zeros
const difficulty = 2

func mine(b *Block) {
	for {
		b.Nonce++
		h := calculateHash(b)
		if h[:difficulty] == stringsRepeat("0", difficulty) {
			b.Hash = h
			return
		}
	}
}

func (bc *Blockchain) GenerateBlock(data string) *Block {
	latest := bc.Latest()
	var idx int
	var prev string
	if latest == nil {
		idx = 1
		prev = ""
	} else {
		idx = latest.Index + 1
		prev = latest.Hash
	}
	b := &Block{
		Index:     idx,
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Data:      data,
		PrevHash:  prev,
		Nonce:     0,
	}
	mine(b)
	return b
}

func (bc *Blockchain) AddBlock(b *Block) bool {
	bc.mu.Lock()
	defer bc.mu.Unlock()
	latest := bc.Latest()
	// simple validation
	if latest != nil {
		if b.Index != latest.Index+1 {
			return false
		}
		if b.PrevHash != latest.Hash {
			return false
		}
		if calculateHash(b) != b.Hash {
			return false
		}
	}
	bc.chain = append(bc.chain, b)
	return true
}

func (bc *Blockchain) ReplaceChain(newChain []*Block) bool {
	bc.mu.Lock()
	defer bc.mu.Unlock()
	if len(newChain) <= len(bc.chain) {
		return false
	}
	// validate newChain
	for i := 1; i < len(newChain); i++ {
		prev := newChain[i-1]
		curr := newChain[i]
		if curr.PrevHash != prev.Hash || calculateHash(curr) != curr.Hash {
			return false
		}
	}
	bc.chain = newChain
	return true
}

// helper to avoid importing strings package just for Repeat in this file
func stringsRepeat(s string, count int) string {
	if count <= 0 { return "" }
	out := ""
	for i:=0;i<count;i++ { out += s }
	return out
}


// Persistence helpers


import (
	"crypto/ed25519"
	"encoding/hex"
    "io/ioutil"
    "encoding/json"
)

// SaveToFile saves the chain to a JSON file atomically.
func (bc *Blockchain) SaveToFile(fn string) error {
    bc.mu.RLock()
    defer bc.mu.RUnlock()
    bts, err := json.MarshalIndent(bc.chain, "", "  ")
    if err != nil { return err }
    tmp := fn + ".tmp"
    if err := ioutil.WriteFile(tmp, bts, 0644); err != nil { return err }
    return os.Rename(tmp, fn)
}

// LoadFromFile loads chain from JSON file.
func (bc *Blockchain) LoadFromFile(fn string) error {
    bc.mu.Lock()
    defer bc.mu.Unlock()
    bts, err := ioutil.ReadFile(fn)
    if err != nil { return err }
    var arr []*Block
    if err := json.Unmarshal(bts, &arr); err != nil { return err }
    // validate chain
    if len(arr) == 0 { return fmt.Errorf("empty chain") }
    for i := 1; i < len(arr); i++ {
        prev := arr[i-1]
        curr := arr[i]
        if curr.PrevHash != prev.Hash || calculateHash(curr) != curr.Hash {
            return fmt.Errorf("invalid chain at index %d", i)
        }
    }
    bc.chain = arr
    return nil
}



// SignBlock signs a block with given hex-encoded private key (ed25519)
func SignBlock(b *Block, privHex string) error {
	priv, err := hex.DecodeString(privHex)
	if err != nil { return err }
	if len(priv) != ed25519.PrivateKeySize { return fmt.Errorf("bad priv size") }
	// compute hash to sign
	msg := calculateHash(b)
	sig := ed25519.Sign(ed25519.PrivateKey(priv), []byte(msg))
	b.Signature = hex.EncodeToString(sig)
	// pubkey
	pub := ed25519.PrivateKey(priv).Public().(ed25519.PublicKey)
	b.PubKey = hex.EncodeToString(pub)
	return nil
}

// VerifyBlockSignature verifies signature on block
func VerifyBlockSignature(b *Block) bool {
	if b.Signature == "" || b.PubKey == "" { return false }
	pub, err := hex.DecodeString(b.PubKey)
	if err != nil { return false }
	sig, err := hex.DecodeString(b.Signature)
	if err != nil { return false }
	msg := calculateHash(b)
	return ed25519.Verify(ed25519.PublicKey(pub), []byte(msg), sig)
}
