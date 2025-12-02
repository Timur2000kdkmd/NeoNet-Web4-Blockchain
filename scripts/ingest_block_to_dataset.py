# ingest_block_to_dataset.py
# Fetch block transactions and append to a daily jsonl file for dataset.
import os, requests, json, time
ETH_NODE = os.environ.get('ETH_NODE', 'http://geth:8545')
OUT_DIR = os.environ.get('NEONET_DATA_DIR', '/data/transactions')

def fetch_block(num='latest'):
    payload = {'jsonrpc':'2.0','id':1,'method':'eth_getBlockByNumber','params':[num, True]}
    r = requests.post(ETH_NODE, json=payload, timeout=5)
    return r.json().get('result')

def append_block(block):
    ts = int(time.time())
    fname = os.path.join(OUT_DIR, f'block_{block.get("number","latest")}.jsonl')
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(fname,'a') as fh:
        for tx in block.get('transactions', []):
            fh.write(json.dumps(tx) + '\n')

if __name__=='__main__':
    b = fetch_block()
    if b:
        append_block(b)
        print('Appended block', b.get('number'))
    else:
        print('No block fetched')
