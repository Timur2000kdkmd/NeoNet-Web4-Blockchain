Go P2P implementation (stage 1)
--------------------------------

This stage implements a basic libp2p-based node with:
- Stream echo handler on protocol /neonet/echo/1.0.0
- GossipSub-based pubsub with topic 'neonet-topic-1'
- Simple CLI to connect to peers by multiaddr or to send messages to peerID

How to build (locally):
1. Install Go (>=1.20) and set GOPATH/GOMOD properly.
2. From this folder run: go mod tidy
3. Build: go build -o neonet_p2p main.go
4. Run: ./neonet_p2p
5. Use another node to connect or paste a multiaddr to connect (e.g. /ip4/127.0.0.1/tcp/4001/p2p/Qm...)

Notes:
- This is a minimal, documented example for local testing. For production use:
  - Add persistent peerstore, NAT traversal, relay, security transports, rate-limiting.
  - Add proper logging, configuration, and tests.


Integration with Rust blockchain (Stage 2):
- The Rust blockchain exposes a local HTTP API on `http://127.0.0.1:3030` with endpoints:
  - POST /tx  -> submit transaction (JSON Transaction)
  - POST /mine -> trigger mining (JSON { "validator": "validator-1" })
  - GET /chain -> fetch full chain
  - GET /health -> health check
- The Go P2P node can call these endpoints locally (e.g., using net/http) to relay transactions from the network into the blockchain,
  or to gossip mined blocks to peers.
- Future improvement: replace HTTP bridge with libp2p streams or gRPC for authenticated messaging.
