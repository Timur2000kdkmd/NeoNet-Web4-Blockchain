package main

import (
        "context"
        "encoding/json"
        "fmt"
        "log"
        "time"

        libp2p "github.com/libp2p/go-libp2p"
        pubsub "github.com/libp2p/go-libp2p-pubsub"
        "github.com/libp2p/go-libp2p/core/host"
        "github.com/libp2p/go-libp2p/core/network"
        "github.com/libp2p/go-libp2p/core/peer"
        ma "github.com/multiformats/go-multiaddr"
)

const (
        NeoNetProtocol = "/neonet/1.0.0"
        NeoNetTopic    = "neonet-consensus"
        NeoNetBlockTopic = "neonet-blocks"
        NeoNetTxTopic  = "neonet-transactions"
)

type LibP2PNode struct {
        host      host.Host
        ctx       context.Context
        ps        *pubsub.PubSub
        blockSub  *pubsub.Subscription
        txSub     *pubsub.Subscription
        node      *Node
        logger    *log.Logger
}

func NewLibP2PNode(ctx context.Context, node *Node) (*LibP2PNode, error) {
        h, err := libp2p.New(
                libp2p.ListenAddrStrings("/ip4/0.0.0.0/tcp/0"),
                libp2p.EnableNATService(),
                libp2p.EnableRelay(),
        )
        if err != nil {
                return nil, fmt.Errorf("failed to create libp2p host: %w", err)
        }

        ps, err := pubsub.NewGossipSub(ctx, h)
        if err != nil {
                h.Close()
                return nil, fmt.Errorf("failed to create pubsub: %w", err)
        }

        lpn := &LibP2PNode{
                host:   h,
                ctx:    ctx,
                ps:     ps,
                node:   node,
                logger: log.Default(),
        }

        h.SetStreamHandler(NeoNetProtocol, lpn.handleStream)

        if err := lpn.subscribeToTopics(); err != nil {
                h.Close()
                return nil, err
        }

        go lpn.DiscoverPeers()

        lpn.logger.Printf("LibP2P Node started with ID: %s", h.ID().Pretty())
        for _, addr := range h.Addrs() {
                lpn.logger.Printf("Listening on: %s/p2p/%s", addr.String(), h.ID().Pretty())
        }

        return lpn, nil
}

func (lpn *LibP2PNode) subscribeToTopics() error {
        blockTopic, err := lpn.ps.Join(NeoNetBlockTopic)
        if err != nil {
                return fmt.Errorf("failed to join block topic: %w", err)
        }

        txTopic, err := lpn.ps.Join(NeoNetTxTopic)
        if err != nil {
                return fmt.Errorf("failed to join tx topic: %w", err)
        }

        blockSub, err := blockTopic.Subscribe()
        if err != nil {
                return fmt.Errorf("failed to subscribe to blocks: %w", err)
        }

        txSub, err := txTopic.Subscribe()
        if err != nil {
                return fmt.Errorf("failed to subscribe to txs: %w", err)
        }

        lpn.blockSub = blockSub
        lpn.txSub = txSub

        go lpn.handleBlockMessages()
        go lpn.handleTxMessages()

        return nil
}

func (lpn *LibP2PNode) handleBlockMessages() {
        for {
                msg, err := lpn.blockSub.Next(lpn.ctx)
                if err != nil {
                        lpn.logger.Printf("block subscription error: %v", err)
                        return
                }

                if msg.ReceivedFrom == lpn.host.ID() {
                        continue
                }

                var block Block
                if err := json.Unmarshal(msg.Data, &block); err != nil {
                        lpn.logger.Printf("failed to unmarshal block: %v", err)
                        continue
                }

                lpn.logger.Printf("Received block %d via libp2p from %s", block.Index, msg.ReceivedFrom.Pretty())

                if VerifyBlockSignature(&block) {
                        if lpn.node.bc.AddBlock(&block) {
                                lpn.logger.Printf("Added block %d from libp2p network", block.Index)
                                go postBlockToAI(&block)
                        }
                } else {
                        lpn.logger.Printf("Invalid block signature from %s", msg.ReceivedFrom.Pretty())
                }
        }
}

func (lpn *LibP2PNode) handleTxMessages() {
        for {
                msg, err := lpn.txSub.Next(lpn.ctx)
                if err != nil {
                        lpn.logger.Printf("tx subscription error: %v", err)
                        return
                }

                if msg.ReceivedFrom == lpn.host.ID() {
                        continue
                }

                var tx TxBody
                if err := json.Unmarshal(msg.Data, &tx); err != nil {
                        lpn.logger.Printf("failed to unmarshal tx: %v", err)
                        continue
                }

                lpn.logger.Printf("Received tx via libp2p from %s: %s", msg.ReceivedFrom.Pretty(), tx.Data)

                newBlock := lpn.node.bc.GenerateBlock(tx.Data)
                if lpn.node.bc.AddBlock(newBlock) {
                        lpn.logger.Printf("Added block from tx, broadcasting...")
                        lpn.BroadcastBlock(newBlock)
                }
        }
}

func (lpn *LibP2PNode) handleStream(s network.Stream) {
        defer s.Close()

        lpn.logger.Printf("New stream from %s", s.Conn().RemotePeer().Pretty())

        dec := json.NewDecoder(s)
        enc := json.NewEncoder(s)

        var msg WireMessage
        if err := dec.Decode(&msg); err != nil {
                lpn.logger.Printf("stream decode error: %v", err)
                return
        }

        switch msg.Type {
        case "requestChain":
                lpn.node.bc.mu.RLock()
                chain := lpn.node.bc.chain
                lpn.node.bc.mu.RUnlock()

                cb, _ := json.Marshal(ChainBody{Chain: chain})
                response := WireMessage{Type: "chain", Body: cb, From: lpn.host.ID().Pretty()}
                enc.Encode(response)

        case "peerDiscovery":
                peers := lpn.GetConnectedPeers()
                body, _ := json.Marshal(map[string]interface{}{"peers": peers})
                response := WireMessage{Type: "peerList", Body: body, From: lpn.host.ID().Pretty()}
                enc.Encode(response)
        }
}

func (lpn *LibP2PNode) BroadcastBlock(block *Block) error {
        blockTopic, err := lpn.ps.Join(NeoNetBlockTopic)
        if err != nil {
                return err
        }

        data, err := json.Marshal(block)
        if err != nil {
                return err
        }

        return blockTopic.Publish(lpn.ctx, data)
}

func (lpn *LibP2PNode) BroadcastTx(tx *TxBody) error {
        txTopic, err := lpn.ps.Join(NeoNetTxTopic)
        if err != nil {
                return err
        }

        data, err := json.Marshal(tx)
        if err != nil {
                return err
        }

        return txTopic.Publish(lpn.ctx, data)
}

func (lpn *LibP2PNode) ConnectToPeer(multiaddr string) error {
        maddr, err := ma.NewMultiaddr(multiaddr)
        if err != nil {
                return fmt.Errorf("invalid multiaddr: %w", err)
        }

        peerInfo, err := peer.AddrInfoFromP2pAddr(maddr)
        if err != nil {
                return fmt.Errorf("failed to get peer info: %w", err)
        }

        ctx, cancel := context.WithTimeout(lpn.ctx, 10*time.Second)
        defer cancel()

        if err := lpn.host.Connect(ctx, *peerInfo); err != nil {
                return fmt.Errorf("failed to connect to peer: %w", err)
        }

        lpn.logger.Printf("Connected to peer: %s", peerInfo.ID.Pretty())
        return nil
}

func (lpn *LibP2PNode) GetConnectedPeers() []string {
        peers := lpn.host.Network().Peers()
        peerAddrs := make([]string, 0, len(peers))

        for _, p := range peers {
                peerAddrs = append(peerAddrs, p.Pretty())
        }

        return peerAddrs
}

func (lpn *LibP2PNode) DiscoverPeers() {
        ticker := time.NewTicker(30 * time.Second)
        defer ticker.Stop()

        for {
                select {
                case <-lpn.ctx.Done():
                        return
                case <-ticker.C:
                        peers := lpn.GetConnectedPeers()
                        lpn.logger.Printf("Connected peers: %d - %v", len(peers), peers)
                }
        }
}

func (lpn *LibP2PNode) Close() error {
        if lpn.blockSub != nil {
                lpn.blockSub.Cancel()
        }
        if lpn.txSub != nil {
                lpn.txSub.Cancel()
        }
        return lpn.host.Close()
}
