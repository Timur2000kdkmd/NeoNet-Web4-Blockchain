package main

import "testing"

func TestCalculateHashConsistency(t *testing.T) {
    b := &Block{Index:1, Timestamp:"2025-01-01T00:00:00Z", Data:"x", PrevHash:"abc", Nonce:0}
    h := calculateHash(b)
    if h == "" {
        t.Fatalf("hash empty")
    }
}
