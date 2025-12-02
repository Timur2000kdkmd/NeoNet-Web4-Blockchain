package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
)

func main() {
	fmt.Println("=== NeoNet Consensus Node Starting ===")
	fmt.Println("Version: 0.1.0 - Go P2P + PBFT Consensus")

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	peers := os.Getenv("PEERS")

	cfg := Config{
		Port:  port,
		Peers: peers,
	}

	node := NewNode(cfg)

	if err := node.LoadChainFromFile("chain_store.json"); err != nil {
		log.Printf("Could not load chain from file: %v, creating genesis", err)
		node.bc.CreateGenesis()
	}

	log.Printf("Blockchain loaded with %d blocks", len(node.bc.chain))

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	libp2pNode, err := NewLibP2PNode(ctx, node)
	if err != nil {
		log.Fatalf("Failed to create LibP2P node: %v", err)
	}
	defer libp2pNode.Close()

	bootstrapPeers := os.Getenv("BOOTSTRAP_PEERS")
	if bootstrapPeers != "" {
		log.Printf("Connecting to bootstrap peers: %s", bootstrapPeers)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/chain", node.HandleGetChain)
	mux.HandleFunc("/peers", node.HandleGetPeers)
	mux.HandleFunc("/tx", node.HandlePostTx)
	mux.HandleFunc("/vote", node.HandleVote)
	mux.HandleFunc("/relay", node.HandleRelay)
	mux.HandleFunc("/pbft/preprepare", node.HandlePBFTPrePrepare)
	mux.HandleFunc("/pbft/prepare", node.HandlePBFTPrepare)
	mux.HandleFunc("/pbft/commit", node.HandlePBFTCommit)

	httpAddr := fmt.Sprintf(":%s", port)
	server := &http.Server{
		Addr:    httpAddr,
		Handler: mux,
	}

	go func() {
		log.Printf("HTTP API listening on %s", httpAddr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("HTTP server error: %v", err)
		}
	}()

	go node.Start()

	log.Println("=== NeoNet Consensus Node Ready ===")
	log.Printf("LibP2P ID: %s", libp2pNode.host.ID().Pretty())
	log.Printf("HTTP API: http://localhost:%s", port)
	log.Printf("Blockchain: %d blocks", len(node.bc.chain))
	log.Println("Press Ctrl+C to shutdown")

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	log.Println("Shutting down gracefully...")
	node.bc.SaveToFile("chain_store.json")
	server.Shutdown(ctx)
	log.Println("Shutdown complete")
}
