import os, time, requests, json
import redis
import binascii
from cryptography.hazmat.primitives.asymmetric import ed25519


AI_BASE = os.environ.get('AI_BASE', 'http://ai-service:8000')
REDIS_URL = os.environ.get('REDIS_URL','redis://localhost:6379/0')
r = redis.Redis.from_url(REDIS_URL, decode_responses=True)

def process_task(task_id):
    # simulate processing
    print('worker processing', task_id)
    time.sleep(2)
    result = 'ok:'+task_id
    # sign using demo PQ API
    priv = os.environ.get('MINER_PRIV','demo-miner-secret')
    sig = demo_sign(priv, result)
    payload = {'miner_id': os.environ.get('MINER_ID','miner-demo-1'), 'result': result, 'sig': sig}
    requests.post(f'{AI_BASE}/submit_miner_result/{task_id}', json=payload)
    print('submitted result for', task_id)

def main_loop():
    print('worker started')
    while True:
        item = r.rpop('task_queue')
        if item:
            try:
                process_task(item)
            except Exception as e:
                print('process error', e)
        else:
            time.sleep(1)

if __name__ == '__main__':
    main_loop()

# optionally trigger on-chain trainer
from trainer import train_on_block

def maybe_train():
    try:
        train_on_block()
    except Exception as e:
        print('trainer error', e)

# append call in main loop
if __name__ == '__main__':
    # run original main loop but after processing call trainer
    main_loop()



def handle_task_payload(task):
    try:
        t = json.loads(task)
    except:
        print('invalid task payload', task)
        return
    typ = t.get('type')
    tid = t.get('id')
    if typ == 'ingest_block':
        # simple processing: create a report file and sign it
        idx = t.get('block_index')
        report = {'task_id': tid, 'block_index': idx, 'result': 'processed'}
        # store report
        outdir = os.environ.get('OUT_DIR','./ai_data')
        os.makedirs(outdir, exist_ok=True)
        with open(os.path.join(outdir, f'report_{tid}.json'), 'w', encoding='utf-8') as fh:
            json.dump(report, fh)
        # simulate model update and store artifact
        try:
            store_model_artifact('model_latest', {'task_id': tid, 'block_index': idx, 'trained': True})
        except Exception as e:
            print('model store error', e)
        # sign using demo_sign and push to completed_reports via HTTP if available
        try:
            sig = None
        try:
            priv_hex = open('python_key_priv.hex','r').read().strip()
            priv = binascii.unhexlify(priv_hex)
            sk = ed25519.Ed25519PrivateKey.from_private_bytes(priv)
            sig = binascii.hexlify(sk.sign(json.dumps(report).encode())).decode()
        except Exception as e:
            print('ed25519 sign error', e)
            sig = ''
            ai_base = os.environ.get('AI_BASE','http://localhost:8000')
            requests.post(f'{ai_base}/submit_miner_result/{tid}', json={'report': report, 'signature': sig})
        except Exception as e:
            print('submit report error', e)
    else:
        # fallback existing handling
        print('unknown task type', typ)

# Replace main loop consumer to call handle_task_payload
def main_loop():
    print('worker main loop (patched)')
    while True:
        task = r.blpop('task_queue', timeout=5)
        if not task:
            time.sleep(1)
            continue
        # blpop returns (queue, value)
        _, val = task
        print('worker popped', val)
        # if task was ingest_block, attempt to send vote for the block (AI acts as validator)
        try:
            tjson = json.loads(val)
            if tjson.get('type') == 'ingest_block':
                blk_idx = tjson.get('block_index')
                # read block file to get hash
                bfile = os.path.join(os.environ.get('OUT_DIR','./ai_data'), f'block_{blk_idx}.jsonl')
                if os.path.exists(bfile):
                    with open(bfile,'r') as bf:
                        last = None
                        for line in bf:
                            last = line
                        if last:
                            try:
                                bj = json.loads(last)
                                send_vote_for_block(blk_idx, bj.get('hash',''))
                            except:
                                pass
        except Exception as e:
            print('vote send attempt error', e)

        handle_task_payload(val)

# At end of file ensure main_loop invoked



def send_vote_for_block(block_index, block_hash, round=1):
    try:
        pkf = os.environ.get('AI_PRIV','python_key_priv.hex')
        with open(pkf,'r') as fh:
            privhex = fh.read().strip()
        priv = binascii.unhexlify(privhex)
        # use cryptography to sign
        from cryptography.hazmat.primitives.asymmetric import ed25519 as _ed
        sk = _ed.Ed25519PrivateKey.from_private_bytes(priv)
        msg = f"{block_hash}:{round}".encode()
        sig = sk.sign(msg)
        sighex = binascii.hexlify(sig).decode()
        pub = sk.public_key().public_bytes(encoding=__import__('serialization').serialization.Encoding.Raw, format=__import__('serialization').serialization.PublicFormat.Raw) if False else None
    except Exception as e:
        print('send_vote error', e)
        return
    try:
        # derive pubkey from private bytes if possible (cryptography doesn't provide direct bytes accessor for public without serialization)
        try:
            from cryptography.hazmat.primitives.asymmetric import ed25519 as _ed2
            pk = _ed2.Ed25519PrivateKey.from_private_bytes(priv)
            pubkey = pk.public_key().public_bytes(encoding=__import__('serialization').serialization.Encoding.Raw, format=__import__('serialization').serialization.PublicFormat.Raw)
            pubhex = binascii.hexlify(pubkey).decode()
        except Exception:
            pubhex = ''
        payload = {'block_hash': block_hash, 'voter_pub': pubhex, 'signature': sighex, 'round': round}
        node = os.environ.get('NODE_HTTP','http://127.0.0.1:8080')
        requests.post(node + '/vote', json=payload, timeout=3)
    except Exception as e:
        print('send_vote post error', e)



def store_model_artifact(model_name, metadata):
    outdir = os.path.join(os.path.dirname(__file__), "model_registry")
    os.makedirs(outdir, exist_ok=True)
    fname = os.path.join(outdir, f"{model_name}.json")
    with open(fname, 'w') as fh:
        json.dump(metadata, fh)
    print("stored model artifact", fname)
