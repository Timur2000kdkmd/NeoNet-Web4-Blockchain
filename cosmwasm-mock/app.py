from fastapi import FastAPI
from pydantic import BaseModel
import uvicorn, time, json

app = FastAPI(title='CosmWasm Mock RPC')

class ExecRequest(BaseModel):
    contract: str
    msg: dict
    sender: str

@app.post('/wasm/execute')
def wasm_execute(req: ExecRequest):
    # simulate executing a WASM contract and return a deterministic result
    res = {
        'contract': req.contract,
        'msg': req.msg,
        'sender': req.sender,
        'executed_at': int(time.time()),
        'result': 'wasm-executed'
    }
    return res

if __name__ == '__main__':
    uvicorn.run(app, host='0.0.0.0', port=1317)
