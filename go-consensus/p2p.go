package main

import (
	"crypto/sha256"
	"crypto/ed25519"
	"encoding/hex"
	"io"
	"net/http"
	"bufio"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"strings"
	"sync"
	"time"
)

// Message types used on the wire
type WireMessage struct {
	Type string          `json:"type"`
	Body json.RawMessage `json:"body,omitempty"`
	From string          `json:"from,omitempty"`
}

// PeerList body
type PeerListBody struct {
	Peers []string `json:"peers"`
}

// Transaction body (simple)
type TxBody struct {
	Data string `json:"data"`
}

// Chain body
type ChainBody struct {
	Chain []*Block `json:"chain"`
}

// Node represents a local node
type Node struct {
	cfg       Config
	peersMu   sync.RWMutex
	peers     map[string]struct{}
	ln        net.Listener
	bc        *Blockchain
	logger    *log.Logger
	closeOnce sync.Once
}

// Config for the node
type Config struct {
	Port  string
	Peers string // comma-separated
}

func NewNode(cfg Config) *Node {
	n := &Node{
		cfg:    cfg,
		peers:  make(map[string]struct{}),
		bc:     NewBlockchain(),
		logger: log.Default(),
	}
	// seed peers from config
	if cfg.Peers != "" {
		for _, p := range strings.Split(cfg.Peers, ",") {
			p = strings.TrimSpace(p)
			if p != "" {
				n.peers[p] = struct{}{}
			}
		}
	}
	// create genesis if empty
	if len(n.bc.chain) == 0 {
		n.bc.CreateGenesis()
	}
	return n
}

func (n *Node) Start() {
	addr := fmt.Sprintf(":%s", n.cfg.Port)
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		n.logger.Fatalf("listen: %v", err)
	}
	n.ln = ln
	n.logger.Printf("listening on %s", addr)

	// connect to known peers in background goroutines
	go n.connectToInitialPeers()

	// accept loop
	for {
		conn, err := ln.Accept()
		if err != nil {
			n.logger.Printf("accept error: %v", err)
			continue
		}
		go n.handleConn(conn)
	}
}

func (n *Node) connectToInitialPeers() {
	n.peersMu.RLock()
	plist := make([]string, 0, len(n.peers))
	for p := range n.peers {
		plist = append(plist, p)
	}
	n.peersMu.RUnlock()

	for _, p := range plist {
		go n.connectAndHandshake(p)
	}
}

func (n *Node) connectAndHandshake(peerAddr string) {
	conn, err := net.DialTimeout("tcp", peerAddr, 3*time.Second)
	if err != nil {
		n.logger.Printf("connect %s error: %v", peerAddr, err)
		return
	}
	n.logger.Printf("connected to %s", peerAddr)
	// send our peer list
	pl := n.getPeerList()
	body, _ := json.Marshal(PeerListBody{Peers: pl})
	msg := WireMessage{Type: "peerList", Body: body, From: ":" + n.cfg.Port}
	enc := json.NewEncoder(conn)
	if err := enc.Encode(&msg); err != nil {
		n.logger.Printf("handshake send error to %s: %v", peerAddr, err)
		conn.Close()
		return
	}
	// request chain
	req := WireMessage{Type: "requestChain", From: ":" + n.cfg.Port}
	_ = enc.Encode(&req)

	// keep connection open for incoming messages from this peer
	n.handleConn(conn)
}

func (n *Node) handleConn(conn net.Conn) {
	defer conn.Close()
	remote := conn.RemoteAddr().String()
	n.addPeer(remote)
	dec := json.NewDecoder(conn)
	enc := json.NewEncoder(conn)
	for {
		var msg WireMessage
		if err := dec.Decode(&msg); err != nil {
			// remote closed or invalid
			n.logger.Printf("decode from %s error: %v", remote, err)
			return
		}
		switch msg.Type {
		case "peerList":
			var pl PeerListBody
			_ = json.Unmarshal(msg.Body, &pl)
			for _, p := range pl.Peers {
				if p == "" { continue }
				n.addPeer(p)
			}
			// reply with our list
			my := n.getPeerList()
			b, _ := json.Marshal(PeerListBody{Peers: my})
			_ = enc.Encode(WireMessage{Type: "peerList", Body: b, From: ":" + n.cfg.Port})
		case "tx":
			var tx TxBody
			_ = json.Unmarshal(msg.Body, &tx)
			n.logger.Printf("received tx from %s: %s", msg.From, tx.Data)
			// create new block with tx and broadcast block
			var newB *Block
	// ask rust to create and persist block
	if resp, err := callRustSubmitTx(tx.Data); err == nil {
		// parse response JSON
		var robj map[string]interface{}
		_ = json.Unmarshal(resp, &robj)
		if robj["ok"] == true {
			bdata, _ := json.Marshal(robj["block"])
			var bb Block
			_ = json.Unmarshal(bdata, &bb)
			newB = &bb
		}
	} else {
		n.logger.Printf("rust submit tx error: %v", err)
	}
	if newB == nil {
		// fallback to local generation
		newB = n.bc.GenerateBlock(tx.Data)
	}

	// sign block using node key if available
	if data, err := os.ReadFile("keys/node_priv.hex"); err == nil {
		_ = SignBlock(newB, string(data))
	}
			if n.bc.AddBlock(newB) {
				n.logger.Printf("added block %d hash=%s", newB.Index, newB.Hash); go postBlockToAI(newB)
				n.broadcastBlock(newB)
			}
		case "block":
			var b Block
			_ = json.Unmarshal(msg.Body, &b)
			n.logger.Printf("received block %d from %s", b.Index, msg.From)
			if VerifyBlockSignature(&b) { if n.bc.AddBlock(&b) {
				n.logger.Printf("added block %d from peer", b.Index); go postBlockToAI(&b)
				n.broadcastBlock(&b)
			} else {
				// maybe chain is behind: request chain
				_ = enc.Encode(WireMessage{Type: "requestChain", From: ":" + n.cfg.Port})
			}
		case "requestChain":
			// reply with full chain
			cb, _ := json.Marshal(ChainBody{Chain: n.bc.chain})
			_ = enc.Encode(WireMessage{Type: "chain", Body: cb, From: ":" + n.cfg.Port})
		case "chain":
			var cb ChainBody
			_ = json.Unmarshal(msg.Body, &cb)
			n.logger.Printf("received chain from %s length=%d", msg.From, len(cb.Chain))
			n.bc.ReplaceChain(cb.Chain)
		default:
			n.logger.Printf("unknown msg type %s from %s", msg.Type, msg.From)
		}
	}
}

func (n *Node) getPeerList() []string {
	n.peersMu.RLock()
	defer n.peersMu.RUnlock()
	out := make([]string, 0, len(n.peers))
	for p := range n.peers {
		out = append(out, p)
	}
	return out
}

func (n *Node) addPeer(a string) {
	if a == "" { return }
	n.peersMu.Lock()
	defer n.peersMu.Unlock()
	if _, ok := n.peers[a]; !ok {
		n.peers[a] = struct{}{}
		n.logger.Printf("added peer %s", a)
	}
}

func (n *Node) broadcastBlock(b *Block) {
	n.peersMu.RLock()
	peers := make([]string, 0, len(n.peers))
	for p := range n.peers {
		peers = append(peers, p)
	}
	n.peersMu.RUnlock()

	body, _ := json.Marshal(b)
	msg := WireMessage{Type: "block", Body: body, From: ":" + n.cfg.Port}
	for _, p := range peers {
		go func(peerAddr string) {
			conn, err := net.DialTimeout("tcp", peerAddr, 2*time.Second)
			if err != nil {
				n.logger.Printf("broadcast connect %s err: %v", peerAddr, err)
				return
			}
			defer conn.Close()
			enc := json.NewEncoder(conn)
			_ = enc.Encode(msg)
		}(p)
	}
}


// POST_AI_INGEST helper


import (
	"crypto/sha256"
	"crypto/ed25519"
	"encoding/hex"
	"io"
	"net/http"
    "bytes"
    "net/http"
    "os"
)

func postBlockToAI(b *Block) {
    ai := os.Getenv("AI_BASE")
    if ai == "" {
        ai = "http://127.0.0.1:8000"
    }
    url := ai + "/ingest_block"
    // prepare JSON
    jb, err := json.Marshal(b)
    if err != nil {
        // log but don't fail
        return
    }
    client := &http.Client{Timeout: 3 * time.Second}
    resp, err := client.Post(url, "application/json", bytes.NewReader(jb))
    if err != nil {
        // ignore error
        return
    }
    resp.Body.Close()
}



// HTTP handlers for REST API
func (n *Node) HandleGetChain(w http.ResponseWriter, r *http.Request) {
    n.bc.mu.RLock()
    defer n.bc.mu.RUnlock()
    bts, _ := json.MarshalIndent(n.bc.chain, "", "  ")
    w.Header().Set("Content-Type", "application/json")
    w.Write(bts)
}

func (n *Node) HandleGetPeers(w http.ResponseWriter, r *http.Request) {
    pl := n.getPeerList()
    bts, _ := json.Marshal(pl)
    w.Header().Set("Content-Type", "application/json")
    w.Write(bts)
}

func (n *Node) HandlePostTx(w http.ResponseWriter, r *http.Request) {
    // expect JSON: {"data":"..."}
    var tb TxBody
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&tb); err != nil {
        http.Error(w, "invalid body", http.StatusBadRequest)
        return
    }
    // generate block locally from tx
    newB := n.bc.GenerateBlock(tb.Data)
    if n.bc.AddBlock(newB) {
        // persist
        n.bc.SaveToFile("chain_store.json")
        n.broadcastBlock(newB)
        go postBlockToAI(newB)
        w.WriteHeader(http.StatusCreated)
        b, _ := json.Marshal(newB)
        w.Write(b)
        return
    }
    http.Error(w, "failed to add block", http.StatusInternalServerError)
}




// LoadChainFromFile tries to load persisted chain into node's blockchain.
func (n *Node) LoadChainFromFile(fn string) error {
    if n.bc == nil {
        n.bc = NewBlockchain()
    }
    err := n.bc.LoadFromFile(fn)
    if err != nil {
        return err
    }
    n.logger.Printf("loaded chain from %s length=%d", fn, len(n.bc.chain))
    return nil
}



func (n *Node) HandleRelay(w http.ResponseWriter, r *http.Request) {
    // expect JSON {"contract":"...","payload":{...}}
    var in map[string]interface{}
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&in); err != nil {
        http.Error(w, "invalid body", http.StatusBadRequest)
        return
    }
    // forward to cosmwasm-mock (assumes it's listening on 8001)
    mock := os.Getenv("COSMWASM_MOCK")
    if mock == "" { mock = "http://127.0.0.1:8001" }
    bts, _ := json.Marshal(in)
    client := &http.Client{Timeout: 5 * time.Second}
    resp, err := client.Post(mock+"/execute", "application/json", bytes.NewReader(bts))
    if err != nil {
        http.Error(w, "relay error: "+err.Error(), http.StatusInternalServerError)
        return
    }
    defer resp.Body.Close()
    out, _ := io.ReadAll(resp.Body)
    w.Write(out)
}



// callRustSubmitTx sends a JSON request to rust bridge at 127.0.0.1:6000 to submit a tx and returns the block JSON as bytes.
func callRustSubmitTx(data string) ([]byte, error) {
    req := map[string]interface{}{"cmd":"submit_tx", "data": map[string]string{"data": data}}
    jb, _ := json.Marshal(req)
    conn, err := net.DialTimeout("tcp", "127.0.0.1:6000", 2*time.Second)
    if err != nil {
        return nil, err
    }
    defer conn.Close()
    // write all then close write
    conn.SetWriteDeadline(time.Now().Add(2 * time.Second))
    _, err = conn.Write(jb)
    if err != nil { return nil, err }
    // read response
    conn.SetReadDeadline(time.Now().Add(3 * time.Second))
    resp := make([]byte, 65536)
    n, err := conn.Read(resp)
    if err != nil {
        return nil, err
    }
    return resp[:n], nil
}

// callRustGetChain asks rust bridge for full chain and returns bytes
func callRustGetChain() ([]byte, error) {
    req := map[string]interface{}{"cmd":"get_chain"}
    jb, _ := json.Marshal(req)
    conn, err := net.DialTimeout("tcp", "127.0.0.1:6000", 2*time.Second)
    if err != nil { return nil, err }
    defer conn.Close()
    conn.SetWriteDeadline(time.Now().Add(2 * time.Second))
    _, err = conn.Write(jb)
    if err != nil { return nil, err }
    conn.SetReadDeadline(time.Now().Add(3 * time.Second))
    resp := make([]byte, 1<<20)
    n, err := conn.Read(resp)
    if err != nil {
        return nil, err
    }
    return resp[:n], nil
}



// PBFT-lite structures and vote handling
type Vote struct {
    BlockHash string `json:"block_hash"`
    VoterPub  string `json:"voter_pub"`
    Signature string `json:"signature"`
    Round     int    `json:"round"`
}

var voteLock sync.Mutex
var votesMap = make(map[string]map[string]Vote) // blockhash -> voterpub -> Vote

// HandleVote accepts votes from validators (AI or nodes)
func (n *Node) HandleVote(w http.ResponseWriter, r *http.Request) {
    var v Vote
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&v); err != nil {
        http.Error(w, "invalid vote", http.StatusBadRequest)
        return
    }
    // verify signature of vote against voter pubkey (ed25519)
    pubb, err := hex.DecodeString(v.VoterPub)
    if err != nil {
        http.Error(w, "bad pub", http.StatusBadRequest)
        return
    }
    sigb, err := hex.DecodeString(v.Signature)
    if err != nil {
        http.Error(w, "bad sig", http.StatusBadRequest)
        return
    }
    // message to verify is blockhash + round as bytes
    msg := []byte(fmt.Sprintf("%s:%d", v.BlockHash, v.Round))
    if !ed25519.Verify(ed25519.PublicKey(pubb), msg, sigb) {
        http.Error(w, "invalid signature", http.StatusBadRequest)
        return
    }
    // store vote
    voteLock.Lock()
    defer voteLock.Unlock()
    if votesMap[v.BlockHash] == nil {
        votesMap[v.BlockHash] = make(map[string]Vote)
    }
    votesMap[v.BlockHash][v.VoterPub] = v
    // check threshold: simple 2/3 of known validators (we'll read validators list)
    validators := n.getValidatorList()
    needed := (2*len(validators))/3 + 1
    count := len(votesMap[v.BlockHash])
    if count >= needed {
        // commit block by broadcasting commit message (here we just log and persist)
        n.logger.Printf("block %s committed by %d/%d votes", v.BlockHash, count, len(validators))
        // optionally notify rust to commit chain or mark block committed
    }
    w.Write([]byte(`{"ok":true,"votes":` + fmt.Sprintf("%d", count) + `}`))
}

// getValidatorList reads validators from validators.json or returns all known peers (fallback)
func (n *Node) getValidatorList() []string {
    var out []string
    // try read file
    data, err := os.ReadFile("validators.json")
    if err == nil {
        _ = json.Unmarshal(data, &out)
        if len(out) > 0 { return out }
    }
    // fallback: use known peers
    out = n.getPeerList()
    return out
}


// PBFT full flow structures
type PrePrepare struct {
    View   int    `json:"view"`
    Seq    int    `json:"seq"`
    Block  *Block `json:"block"`
    Leader string `json:"leader"`
    Sig    string `json:"sig"` // leader signature over block hash:seq:view
}
type PrepareMsg struct {
    View int    `json:"view"`
    Seq  int    `json:"seq"`
    Hash string `json:"hash"`
    Voter string `json:"voter"`
    Sig   string `json:"sig"`
}
type CommitMsg struct {
    View int    `json:"view"`
    Seq  int    `json:"seq"`
    Hash string `json:"hash"`
    Voter string `json:"voter"`
    Sig   string `json:"sig"`
}

// PBFT state (in-memory)
var pbftMu sync.Mutex
var prePrepares = make(map[int]*PrePrepare) // seq -> preprepare
var prepares = make(map[int]map[string]PrepareMsg) // seq -> voter -> prepare
var commits = make(map[int]map[string]CommitMsg) // seq -> voter -> commit
var pbftSeq = 1
var pbftView = 0

// helper to sign message by node's key (reads node_priv.hex)
func signMessage(msg string) string {
    // read private key from keys/node_priv.hex (hex-encoded 64 bytes)
    data, err := os.ReadFile("keys/node_priv.hex")
    if err != nil { return "" }
    privHex := strings.TrimSpace(string(data))
    privb, err := hex.DecodeString(privHex)
    if err != nil { return "" }
    if len(privb) == ed25519.PrivateKeySize {
        sig := ed25519.Sign(ed25519.PrivateKey(privb), []byte(msg))
        return hex.EncodeToString(sig)
    }
    // if priv is 32-byte seed, create private key
    if len(privb) == ed25519.SeedSize {
        priv := ed25519.NewKeyFromSeed(privb)
        sig := ed25519.Sign(priv, []byte(msg))
        return hex.EncodeToString(sig)
    }
    return ""
}

// proposeBlock is called by leader to broadcast preprepare
func (n *Node) proposeBlockToValidators(b *Block) {
    pbftMu.Lock()
    seq := pbftSeq
    pbftSeq++
    view := pbftView
    pbftMu.Unlock()
    pre := &PrePrepare{View:view, Seq:seq, Block:b, Leader:":"+n.cfg.Port}
    // sign message
    msg := fmt.Sprintf("%s:%d:%d", b.Hash, seq, view)
    pre.Sig = signMessage(msg)
    prePrepares[seq] = pre
    // broadcast preprepare to validators (HTTP /pbft/preprepare)
    vlist := n.getValidatorList()
    for _, v := range vlist {
        go func(addr string) {
            url := "http://127.0.0.1:8080/pbft/preprepare" // assume validator HTTP on 8080 for simplicity
            jb, _ := json.Marshal(pre)
            client := &http.Client{Timeout: 2 * time.Second}
            _, _ = client.Post(url, "application/json", bytes.NewReader(jb))
        }(v)
    }
}

// Handler for /pbft/preprepare
func (n *Node) HandlePBFTPrePrepare(w http.ResponseWriter, r *http.Request) {
    var pp PrePrepare
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&pp); err != nil {
        http.Error(w, "invalid", http.StatusBadRequest); return
    }
    // basic check: verify block signature via VerifyBlockSignature (provided)
    if pp.Block == nil || !VerifyBlockSignature(pp.Block) {
        http.Error(w, "bad block sig", http.StatusBadRequest); return
    }
    // store and send Prepare
    pbftMu.Lock()
    prePrepares[pp.Seq] = &pp
    if prepares[pp.Seq] == nil { prepares[pp.Seq] = make(map[string]PrepareMsg) }
    pbftMu.Unlock()
    // send Prepare message
    prep := PrepareMsg{View:pp.View, Seq:pp.Seq, Hash:pp.Block.Hash, Voter:":"+n.cfg.Port, Sig: signMessage(fmt.Sprintf("%s:%d:%d", pp.Block.Hash, pp.Seq, pp.View))}
    // broadcast to validators
    vlist := n.getValidatorList()
    for _, v := range vlist {
        go func(addr string) {
            url := "http://127.0.0.1:8080/pbft/prepare"
            jb, _ := json.Marshal(prep)
            client := &http.Client{Timeout: 2 * time.Second}
            _, _ = client.Post(url, "application/json", bytes.NewReader(jb))
        }(v)
    }
    w.Write([]byte(`{"ok":true}`))
}

// Handler for /pbft/prepare
func (n *Node) HandlePBFTPrepare(w http.ResponseWriter, r *http.Request) {
    var pm PrepareMsg
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&pm); err != nil {
        http.Error(w, "invalid", http.StatusBadRequest); return
    }
    pbftMu.Lock()
    if prepares[pm.Seq] == nil { prepares[pm.Seq] = make(map[string]PrepareMsg) }
    prepares[pm.Seq][pm.Voter] = pm
    cnt := len(prepares[pm.Seq])
    val := n.getValidatorList()
    need := (2*len(val))/3 + 1
    pbftMu.Unlock()
    if cnt >= need {
        // send commit
        cm := CommitMsg{View:pm.View, Seq:pm.Seq, Hash:pm.Hash, Voter:":"+n.cfg.Port, Sig: signMessage(fmt.Sprintf("%s:%d:%d", pm.Hash, pm.Seq, pm.View))}
        for _, v := range val {
            go func(addr string) {
                url := "http://127.0.0.1:8080/pbft/commit"
                jb, _ := json.Marshal(cm)
                client := &http.Client{Timeout:2*time.Second}
                _, _ = client.Post(url, "application/json", bytes.NewReader(jb))
            }(v)
        }
    }
    w.Write([]byte(`{"ok":true}`))
}

// Handler for /pbft/commit
func (n *Node) HandlePBFTCommit(w http.ResponseWriter, r *http.Request) {
    var cm CommitMsg
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&cm); err != nil {
        http.Error(w, "invalid", http.StatusBadRequest); return
    }
    pbftMu.Lock()
    if commits[cm.Seq] == nil { commits[cm.Seq] = make(map[string]CommitMsg) }
    commits[cm.Seq][cm.Voter] = cm
    cnt := len(commits[cm.Seq])
    val := n.getValidatorList()
    need := (2*len(val))/3 + 1
    pbftMu.Unlock()
    if cnt >= need {
        // commit achieved, notify rust bridge to persist/mark committed
        go func(h string) {
            req := map[string]interface{}{"cmd":"commit_block","data":map[string]string{"hash":h}}
            jb, _ := json.Marshal(req)
            conn, err := net.DialTimeout("tcp","127.0.0.1:6000",2*time.Second)
            if err == nil {
                conn.Write(jb); conn.Close()
            }
        }(cm.Hash)
        n.logger.Printf("seq %d committed by %d/%d", cm.Seq, cnt, len(val))
    }
    w.Write([]byte(`{"ok":true}`))
}


// isLeaderFor checks if this node is the leader for given view and seq (simple round-robin by validators list)
func (n *Node) isLeaderFor(view int, seq int) bool {
    vals := n.getValidatorList()
    if len(vals) == 0 { return false }
    idx := seq % len(vals)
    leader := vals[idx]
    my := ":" + n.cfg.Port
    return leader == my || leader == ("127.0.0.1"+my) || leader == ("localhost"+my)
}
