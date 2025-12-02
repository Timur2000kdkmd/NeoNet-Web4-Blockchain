package main

import (
    "strings"
    "time"

    "context"
    "crypto/ecdsa"
    "encoding/hex"
    "fmt"
    "io/ioutil"
    "log"
    "math/big"
    "os"

    "github.com/ethereum/go-ethereum/accounts/abi"
    "github.com/ethereum/go-ethereum/accounts/abi/bind"
    "github.com/ethereum/go-ethereum/common"
    "github.com/ethereum/go-ethereum/crypto"
    "github.com/ethereum/go-ethereum/ethclient"
)

func mustReadFile(path string) string {
    b, err := ioutil.ReadFile(path)
    if err != nil {
        log.Fatalf("failed to read %s: %v", path, err)
    }
    return string(b)
}

func main() {
    rpcURL := os.Getenv("RPC_URL")
    if rpcURL == "" {
        rpcURL = "http://127.0.0.1:8545"
    }
    privateKeyHex := os.Getenv("PRIVATE_KEY_HEX")
    if privateKeyHex == "" {
        log.Fatal("set PRIVATE_KEY_HEX env var (hex of ECDSA private key)") 
    }

    abiRaw := mustReadFile("../contract.abi.json")
    binRaw := mustReadFile("../contract.bin")
    // remove possible newlines/spaces
    binRaw = string(binRaw)

    parsedAbi, err := abi.JSON(strings.NewReader(abiRaw))
    if err != nil {
        log.Fatalf("failed to parse abi: %v", err)
    }

    client, err := ethclient.Dial(rpcURL)
    if err != nil {
        log.Fatalf("failed to connect to rpc: %v", err)
    }
    defer client.Close()

    privBytes, err := hex.DecodeString(privateKeyHex)
    if err != nil {
        log.Fatalf("invalid private key hex: %v", err)
    }
    priv, err := crypto.ToECDSA(privBytes)
    if err != nil {
        log.Fatalf("failed to parse private key: %v", err)
    }
    fromAddr := crypto.PubkeyToAddress(priv.PublicKey)

    // prepare auth
    chainID, err := client.NetworkID(context.Background())
    if err != nil {
        log.Fatalf("failed to get chainID: %v", err)
    }
    auth, err := bind.NewKeyedTransactorWithChainID(priv, chainID)
    if err != nil {
        log.Fatalf("failed to create transactor: %v", err)
    }

    // deploy contract
    fmt.Println("Deploying contract...")
    input := []byte{} // constructor args if any
    address, tx, _, err := bind.DeployContract(auth, parsedAbi, common.FromHex(binRaw), client, input)
    if err != nil {
        log.Fatalf("deploy failed: %v", err)
    }
    fmt.Printf("tx sent: %s\n", tx.Hash().Hex())
    fmt.Printf("contract address: %s\n", address.Hex())

    // Wait for a short time for mining (for local dev chains)
    fmt.Println("Waiting for block mining... (sleep 5s)")
    // naive sleep; production should use receipts polling
    time.Sleep(5 * time.Second)

    // Create a bound contract instance
    contract := bind.NewBoundContract(address, parsedAbi, client, client, client)

    // call setX(42)
    fmt.Println("Calling setX(42)...") 
    tx2, err := contract.Transact(auth, "setX", big.NewInt(42))
    if err != nil {
        log.Fatalf("setX failed: %v", err)
    }
    fmt.Printf("setX tx: %s\n", tx2.Hash().Hex())

    // read x via call (no auth needed)
    var result *big.Int
    err = contract.Call(nil, &result, "x")
    if err != nil {
        log.Fatalf("call x failed: %v", err)
    }
    fmt.Printf("current x (call): %s\n", result.String())
}
