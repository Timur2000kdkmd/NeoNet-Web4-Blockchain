\
package main

import (
    "encoding/hex"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "log"
    "os"

    // Uncomment when liboqs-go is available and liboqs is installed
    // "github.com/open-quantum-safe/liboqs-go/oqs"
)

type HybridKeyJson struct {
    EdPublicHex string `json:"ed_public_hex"`
    EdSecretHex string `json:"ed_secret_hex"`
    PqcPublicHex string `json:"pqc_public_hex"`
    PqcSecretHex string `json:"pqc_secret_hex"`
    Version string `json:"version"`
}

type HybridSignature struct {
    AlgoClassical string `json:"algo_classical"`
    SigClassicalHex string `json:"sig_classical_hex"`
    AlgoPqc string `json:"algo_pqc"`
    SigPqcHex string `json:"sig_pqc_hex"`
    KeyVersion string `json:"key_version"`
}

func main() {
    // Read key.json (produced by Rust tests/run)
    kdata, err := ioutil.ReadFile("test_key.json")
    if err != nil {
        log.Fatalf("failed to read test_key.json: %v", err)
    }
    var keys HybridKeyJson
    if err := json.Unmarshal(kdata, &keys); err != nil {
        log.Fatalf("failed to unmarshal key json: %v", err)
    }

    // Read last_signature.json (produced by Rust)
    sdata, err := ioutil.ReadFile("last_signature.json")
    if err != nil {
        log.Fatalf("failed to read last_signature.json: %v", err)
    }
    var hs HybridSignature
    if err := json.Unmarshal(sdata, &hs); err != nil {
        log.Fatalf("failed to unmarshal signature json: %v", err)
    }

    msg := []byte("hello interoperable PQC")

    // Verify classical Ed25519 part locally using standard library (if desired)
    edSigBytes, _ := hex.DecodeString(hs.SigClassicalHex)
    // Ed25519 verification would be done here (not implemented in this example)

    // Verify PQC part using liboqs-go (pseudo-code)
    // Uncomment and adapt after installing liboqs and liboqs-go:
    /*
    sig := oqs.NewSignature("Dilithium2")
    defer sig.Free()
    pub, _ := hex.DecodeString(keys.PqcPublicHex)
    ok, err := sig.Verify(msg, sigBytes, pub)
    if err != nil {
        log.Fatalf("liboqs verify error: %v", err)
    }
    if ok {
        fmt.Println("PQC signature verified by liboqs-go")
    } else {
        fmt.Println("PQC signature failed verification by liboqs-go")
    }
    */

    fmt.Println("Loaded key.json and signature; please run liboqs-go verification by uncommenting code and installing liboqs.")
    os.Exit(0)
}
