# python-ai-service/trainer.py
# Simple incremental 'training' on on-chain transactions.
# For demo: fetch latest block txs from ETH node, extract simple features and update a JSON model (weights).
import os, requests, json, time, hashlib
MODEL_FILE = os.environ.get('MODEL_FILE', '/data/model.json')
ETH_NODE = os.environ.get('ETH_NODE', 'http://geth:8545')

def extract_features_from_tx(tx):
    # simplistic features: input length, value parity, to address bytes sum mod 100
    input_data = tx.get('input','')
    value = int(tx.get('value','0'),16) if tx.get('value') else 0
    to = tx.get('to','') or ''
    f1 = len(input_data)
    f2 = value % 2
    f3 = sum(bytearray(to.encode())) % 100 if to else 0
    return [f1, f2, f3]

def load_model():
    try:
        with open(MODEL_FILE,'r') as f:
            return json.load(f)
    except:
        # initialize model as simple feature weights
        return {'weights':[0.1,0.1,0.1], 'count':0}

def save_model(m):
    dirn = os.path.dirname(MODEL_FILE)
    if dirn and not os.path.exists(dirn):
        os.makedirs(dirn, exist_ok=True)
    with open(MODEL_FILE,'w') as f:
        json.dump(m,f)

def train_on_block(block_number=None):
    # fetch block by number or latest
    payload = {'jsonrpc':'2.0','id':1,'method':'eth_getBlockByNumber','params':[block_number if block_number else 'latest', True]}
    r = requests.post(ETH_NODE, json=payload, timeout=5)
    if r.status_code != 200:
        return None
    block = r.json().get('result', {})
    txs = block.get('transactions', [])
    if not txs:
        return None
    model = load_model()
    lr = 0.01
    for tx in txs:
        feats = extract_features_from_tx(tx)
        # dummy target: whether input length is even
        target = 1 if feats[0] % 2 == 0 else 0
        # simple linear model update: w += lr*(target - pred)*x
        pred = sum(w*x for w,x in zip(model['weights'], feats))
        err = target - pred
        model['weights'] = [w + lr*err*x for w,x in zip(model['weights'], feats)]
        model['count'] += 1
    save_model(model)
    return model

if __name__ == '__main__':
    while True:
        try:
            m = train_on_block()
            if m:
                print('Trained model, count=', m['count'])
        except Exception as e:
            print('train error', e)
        time.sleep(5)
