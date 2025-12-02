package main

import (
    "context"
    "fmt"
    "io/ioutil"
    "log"

    "github.com/tetratelabs/wazero"
    "github.com/tetratelabs/wazero/api"
)

func main() {
    ctx := context.Background()

    // read wasm file (built from wasm_rust)
    wasmPath := "neonet_wasm_contract.wasm"
    wasmBytes, err := ioutil.ReadFile(wasmPath)
    if err != nil {
        log.Fatalf("failed to read wasm file '%s': %v", wasmPath, err)
    }

    r := wazero.NewRuntime(ctx)
    defer r.Close(ctx)

    mod, err := r.InstantiateModuleFromBinary(ctx, wasmBytes)
    if err != nil {
        log.Fatalf("failed to instantiate wasm module: %v", err)
    }
    defer mod.Close(ctx)

    // call exported function 'add' (i32, i32) -> i32 if exported without mangling
    fn := mod.ExportedFunction("add")
    if fn == nil {
        log.Println("function 'add' not found in wasm exports")
        return
    }
    // call with two integers
    results, err := fn.Call(ctx, 3, 4)
    if err != nil {
        log.Fatalf("call failed: %v", err)
    }
    fmt.Printf("wasm add result: %d\n", uint32(results[0]))
    _ = api.Function(fn) // keep reference to avoid linter complaint
}
