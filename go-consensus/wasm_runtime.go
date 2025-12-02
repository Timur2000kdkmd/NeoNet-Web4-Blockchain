package main

import (
    "context"
    "fmt"
    "io"
    "net/http"
    "os"
    "time"
    "encoding/json"
    "github.com/tetratelabs/wazero"
    "github.com/tetratelabs/wazero/api"
)

// HandleRelayExecute executes a wasm contract payload (simple deterministic exec stub)
func (n *Node) HandleRelayExecute(w http.ResponseWriter, r *http.Request) {
    // expect {"module_path":"./wasm/somemod.wasm","input":{...}}
    var in map[string]interface{}
    dec := json.NewDecoder(r.Body)
    if err := dec.Decode(&in); err != nil {
        http.Error(w, "invalid body", http.StatusBadRequest)
        return
    }
    modulePath, _ := in["module_path"].(string)
    // simplistic: load wasm module bytes and run in wazero with limited gas/time
    ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
    defer cancel()
    rtm := wazero.NewRuntime(ctx)
    defer rtm.Close(ctx)
    modBytes, err := os.ReadFile(modulePath)
    if err != nil {
        http.Error(w, "module not found", http.StatusBadRequest)
        return
    }
    // instantiate module
    mod, err := rtm.InstantiateModuleFromBinary(ctx, modBytes)
    if err != nil {
        http.Error(w, "instantiate error:"+err.Error(), http.StatusInternalServerError)
        return
    }
    // call '_start' if exists
    if fn := mod.ExportedFunction("_start"); fn != nil {
        _, err := fn.Call(ctx)
        if err != nil {
            http.Error(w, "exec error:"+err.Error(), http.StatusInternalServerError)
            return
        }
    }
    w.Write([]byte(`{"ok":true,"result":"executed"}`))
}
