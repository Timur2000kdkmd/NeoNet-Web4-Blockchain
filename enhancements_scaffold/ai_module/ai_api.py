"""Simple AI API to interact with the AINode scaffold.
Endpoints:
- GET /status
- POST /train_step  (body: list of data items)
- POST /propose     (fetch chain state and return a vote, optionally submit to chain)
"""
from flask import Flask, request, jsonify
from agent import AINode
import os

app = Flask(__name__)
NODE_ID = os.getenv('AI_NODE_ID', 'ai-node-1')
BLOCKCHAIN_API = os.getenv('BLOCKCHAIN_API', 'http://127.0.0.1:3030')

node = AINode(NODE_ID, blockchain_api=BLOCKCHAIN_API)

@app.route('/status', methods=['GET'])
def status():
    return jsonify({{'node_id': node.node_id, 'model_state': node.model_state}})

@app.route('/train_step', methods=['POST'])
def train_step():
    data = request.get_json() or []
    res = node.train_step(data)
    return jsonify(res)

@app.route('/propose', methods=['POST'])
def propose():
    chain = node.fetch_chain_state()
    vote = node.propose_vote(chain)
    submit = request.args.get('submit', 'false').lower() == 'true'
    result = {{'vote': vote}}
    if submit:
        r = node.submit_vote_on_chain(vote)
        result['submit_result'] = r
    return jsonify(result)

if __name__ == '__main__':
    app.run(host='127.0.0.1', port=5001)
