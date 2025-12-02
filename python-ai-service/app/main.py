from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel
from typing import Optional
import uuid, asyncio, time, os, json
from sqlalchemy import create_engine, MetaData, Table, Column, Integer, String, Text, JSON
from sqlalchemy.sql import select
import redis

try:
    from ai_engine import ProofOfIntelligenceEngine, DualGovernance, create_ai_powered_network
    AI_ENGINE_ENABLED = True
except ImportError:
    AI_ENGINE_ENABLED = False
    print("AI Engine not available, running without AI features")

DATABASE_URL = os.environ.get('DATABASE_URL', 'sqlite:///./neoai.db')
REDIS_URL = os.environ.get('REDIS_URL', 'redis://localhost:6379/0')

if AI_ENGINE_ENABLED:
    network = create_ai_powered_network()
    ai_engine = network['ai_engine']
    governance = network['governance']
else:
    ai_engine = None
    governance = None

# create DB engine (sync for simplicity)
engine = create_engine(DATABASE_URL, connect_args={"check_same_thread": False} if DATABASE_URL.startswith("sqlite") else {})
metadata = MetaData()

tasks = Table('tasks', metadata,
    Column('id', String, primary_key=True),
    Column('model_id', String),
    Column('payload_ref', String),
    Column('state', String),
    Column('assigned_to', String),
    Column('result', JSON),
)

miners = Table('miners', metadata,
    Column('id', String, primary_key=True),
    Column('info', JSON),
)

metadata.create_all(engine)

r = redis.Redis.from_url(REDIS_URL, decode_responses=True)

app = FastAPI(title="NeoNet AI Service (persistent)")

class MinerRegister(BaseModel):
    miner_id: Optional[str]
    cpu_cores: int
    gpu_memory_mb: int
    endpoint: str

class TaskRequest(BaseModel):
    model_id: str
    payload_ref: str
    priority: int = 1

@app.post("/register_miner")
async def register_miner(m: MinerRegister):
    miner_uid = m.miner_id or str(uuid.uuid4())
    with engine.begin() as conn:
        conn.execute(miners.insert().values(id=miner_uid, info=json.loads(m.json())))
    return {"miner_uid": miner_uid, "status": "registered"}

@app.post("/submit_task")
async def submit_task(t: TaskRequest):
    task_id = str(uuid.uuid4())
    with engine.begin() as conn:
        conn.execute(tasks.insert().values(id=task_id, model_id=t.model_id, payload_ref=t.payload_ref, state='queued'))
    # push to redis queue for workers
    r.lpush('task_queue', task_id)
    return {"task_id": task_id, "status": "queued"}

@app.get("/task_status/{task_id}")
async def task_status(task_id: str):
    with engine.begin() as conn:
        row = conn.execute(select([tasks]).where(tasks.c.id==task_id)).fetchone()
        if not row:
            raise HTTPException(status_code=404, detail="Task not found")
        return dict(row)

@app.post("/task_result/{task_id}")
async def task_result(task_id: str, payload: dict):
    with engine.begin() as conn:
        conn.execute(tasks.update().where(tasks.c.id==task_id).values(state='completed', result=payload))
    return {"status":"ok"}

@app.get("/tasks")
async def list_tasks():
    with engine.begin() as conn:
        rows = conn.execute(select([tasks])).fetchall()
        return [dict(r) for r in rows]

# deterministic aggregation & completed_reports (kept for relayer)
from hashlib import sha256
COMPLETED_REPORTS = {}

@app.post('/submit_miner_result/{task_id}')
def submit_miner_result(task_id: str, payload: dict):
    # payload: {miner_id, result, sig}
    # store result in task.result.results list (here simplified)
    with engine.begin() as conn:
        row = conn.execute(select([tasks]).where(tasks.c.id==task_id)).fetchone()
        if not row:
            return {'error':'task not found'}
        results = row['result'] or {'results': []}
        results['results'].append(payload)
        conn.execute(tasks.update().where(tasks.c.id==task_id).values(result=results))
        collected = len(results['results'])
    # aggregation threshold
    threshold_required = int(os.environ.get('AGG_THRESHOLD', max(1, collected)))
    if collected >= threshold_required:
        # deterministic aggregation
        sorted_results = sorted(results['results'], key=lambda r: r['miner_id'])
        concat = ''.join([r['result'] for r in sorted_results])
        result_hash = sha256(concat.encode()).hexdigest()
        report_id = sha256((task_id + result_hash).encode()).hexdigest()
        signatures = [r.get('sig','') for r in sorted_results if r.get('sig')]
        COMPLETED_REPORTS[report_id] = {
            'task_id': task_id,
            'proposal_id': '0x' + '0'*64,
            'result_hash': '0x' + result_hash,
            'signatures': signatures,
            'threshold': max(1, len(signatures)//2 + 1),
            'relayed': False
        }
        # mark task done
        with engine.begin() as conn:
            conn.execute(tasks.update().where(tasks.c.id==task_id).values(state='completed'))
        return {'report_id': report_id, 'result_hash': '0x' + result_hash}
    return {'status':'partial', 'collected': collected}

@app.get('/completed_reports')
def completed_reports():
    return COMPLETED_REPORTS

# health and metrics for autoscaling
@app.get('/queue_length')
def queue_length():
    qlen = r.llen('task_queue')
    return {'queue_length': qlen}

@app.get('/scale_signal')
def scale_signal():
    qlen = r.llen('task_queue')
    # simple policy: 0-2 -> 1 replica, 3-10 -> 2 replicas, >10 -> 4 replicas
    if qlen <= 2:
        desired = 1
    elif qlen <= 10:
        desired = 2
    else:
        desired = 4
    return {'desired_replicas': desired, 'queue_length': qlen}




from pydantic import BaseModel
from fastapi import Body

class BlockModel(BaseModel):
    index: int
    timestamp: str
    data: str
    prev_hash: str
    hash: str
    nonce: int

@app.post('/ingest_block')
def ingest_block(block: BlockModel, background_tasks: BackgroundTasks):
    '''
    Receive a block from the network and enqueue it for AI processing.
    Stores block into a JSONL file and pushes a task onto Redis "task_queue".
    '''
    try:
        # verify signature if present
        try:
            import binascii
            from cryptography.hazmat.primitives.asymmetric import ed25519
            pub = block.dict().get('pub_key')
            sig = block.dict().get('signature')
            if pub and sig:
                pubb = binascii.unhexlify(pub)
                sigb = binascii.unhexlify(sig)
                pk = ed25519.Ed25519PublicKey.from_public_bytes(pubb)
                msg = f"{block.index}{block.timestamp}{block.data}{block.prev_hash}{block.nonce}"
                pk.verify(sigb, msg.encode())
        except Exception as e:
            raise HTTPException(status_code=400, detail=f"invalid signature: {e}")

        # verify signature if present
        try:
            pub = block.dict().get('pub_key')
            sig = block.dict().get('signature')
            if pub and sig:
                pubb = binascii.unhexlify(pub)
                sigb = binascii.unhexlify(sig)
                pk = ed25519.Ed25519PublicKey.from_public_bytes(pubb)
                # compute message (same as Go calculateHash): index+timestamp+data+prev_hash+nonce as string
                msg = f"{block.index}{block.timestamp}{block.data}{block.prev_hash}{block.nonce}"
                pk.verify(sigb, msg.encode())
        except Exception as e:
            # if verification fails, reject
            raise HTTPException(status_code=400, detail=f"invalid signature: {e}")

        # append block to file for dataset
        OUT_DIR = os.environ.get('OUT_DIR', './ai_data')
        os.makedirs(OUT_DIR, exist_ok=True)
        fname = os.path.join(OUT_DIR, f'block_{block.index}.jsonl')
        with open(fname, 'a', encoding='utf-8') as fh:
            fh.write(json.dumps(block.dict()) + '\n')
        # push a task into redis queue for worker to pick up
        r = redis.Redis.from_url(REDIS_URL, decode_responses=True)
        task_id = str(uuid.uuid4())
        task_payload = {'id': task_id, 'type': 'ingest_block', 'block_index': block.index}
        r.rpush('task_queue', json.dumps(task_payload))
        # return immediate response
        return {'status': 'enqueued', 'task_id': task_id}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
