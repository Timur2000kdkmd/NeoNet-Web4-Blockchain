package main

import (
	"flag"
	"log"
	"net/http"
)

func main() {
	port := flag.String("port", "50051", "listen port")
	peers := flag.String("peers", "", "comma-separated peer addresses (host:port)")
	httpPort := flag.String("http", "8080", "http api port")
	flag.Parse()

	cfg := Config{
		Port:  *port,
		Peers: *peers,
	}
	node := NewNode(cfg)
	log.Printf("Starting node on :%s", cfg.Port)

	// load persisted chain if exists
	if err := node.LoadChainFromFile("chain_store.json"); err != nil {
		log.Printf("no persisted chain loaded: %v", err)
	}

	// start REST API
	go func() {
		mux := http.NewServeMux()
		mux.HandleFunc("/chain", node.HandleGetChain)
		mux.HandleFunc("/peers", node.HandleGetPeers)
	mux.HandleFunc("/vote", node.HandleVote)
	mux.HandleFunc("/pbft/preprepare", node.HandlePBFTPrePrepare)
	mux.HandleFunc("/pbft/prepare", node.HandlePBFTPrepare)
	mux.HandleFunc("/pbft/commit", node.HandlePBFTCommit)
	mux.HandleFunc("/vote", node.HandleVote)
		mux.HandleFunc("/tx", node.HandlePostTx)
	mux.HandleFunc("/relay_execute", node.HandleRelayExecute)
		log.Printf("HTTP API listening on :%s", *httpPort)
		log.Fatal(http.ListenAndServe(":"+*httpPort, mux))
	}()

	node.Start()
}
