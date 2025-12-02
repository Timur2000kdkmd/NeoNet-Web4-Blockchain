

# Keystore support: load encrypted keystore JSON and decrypt using KEYSTORE_PASSWORD env var.
import os, json
KEYSTORE_PATH = os.environ.get('KEYSTORE_PATH', '')  # path to keystore JSON
KEYSTORE_PASSWORD = os.environ.get('KEYSTORE_PASSWORD', '')

def load_account_from_keystore(path, password):
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        # Try to use eth_account if available for proper decryption
        try:
            from eth_account import Account as EthAccount
            acct = EthAccount.from_key(EthAccount.decrypt(data, password))
            print('Loaded account from keystore:', acct.address)
            return acct
        except Exception as e:
            print('eth_account not available or decryption failed:', e)
            # As fallback, search manifest mapping (not secure) - expect relayer to run where eth libs exist
            return None
    except Exception as exc:
        print('Keystore load error', exc)
        return None

import time, requests, os, json
from web3 import Web3
from eth_account import Account
from eth_account.messages import encode_defunct

AI_BASE = os.environ.get('AI_BASE','http://ai-service:8000')
ETH_NODE = os.environ.get('ETH_NODE','http://localhost:8545')
PRIVATE_KEY = os.environ.get('RELAYER_KEY','')  # must be set for real signing
ORACLE_ADDR = os.environ.get('ORACLE_ADDR','0x' + '0'*40)
ORACLE_ABI_PATH = os.environ.get('ORACLE_ABI','/app/oracle_abi.json')

def load_abi():
    try:
        with open(ORACLE_ABI_PATH, 'r') as f:
            return json.load(f)
    except:
        return None

def poll_and_relay():
    print('Starting relayer, polling AI service for aggregated reports...')
    w3 = Web3(Web3.HTTPProvider(ETH_NODE))
    abi = load_abi()
    oracle = w3.eth.contract(address=Web3.to_checksum_address(ORACLE_ADDR), abi=abi) if abi else None
    acct = None
    # attempt to load keystore first
    if KEYSTORE_PATH and KEYSTORE_PASSWORD:
        acct = load_account_from_keystore(KEYSTORE_PATH, KEYSTORE_PASSWORD)
    if not acct and PRIVATE_KEY:
        acct = Account.from_key(PRIVATE_KEY) if PRIVATE_KEY else None
    while True:
        try:
            r = requests.get(f'{AI_BASE}/completed_reports')
            if r.status_code != 200:
                time.sleep(2)
                continue
            reports = r.json()
            for rid, rep in reports.items():
                if rep.get('relayed'):
                    continue
                proposalId_hex = rep['proposal_id']
                reportId_hex = rid
                resultHash_hex = rep['result_hash']
                signatures = rep.get('signatures', [])
                threshold = rep.get('threshold', max(1, len(signatures)//2 + 1))
                print('Relaying report', reportId_hex, 'for proposal', proposalId_hex)
                if oracle and acct:
                    try:
                        # convert hex strings to bytes32
                        proposalId = Web3.toBytes(hexstr=proposalId_hex)
                        reportId = Web3.toBytes(hexstr=reportId_hex)
                        resultHash = Web3.toBytes(hexstr=resultHash_hex)
                        tx = oracle.functions.submitReport(proposalId, reportId, resultHash, signatures, threshold).buildTransaction({
                            'nonce': w3.eth.get_transaction_count(acct.address),
                            'gas': 800000,
                            'gasPrice': w3.eth.gas_price
                        })
                        signed = w3.eth.account.sign_transaction(tx, private_key=PRIVATE_KEY)
                        txh = w3.eth.send_raw_transaction(signed.rawTransaction)
                        print('Submitted tx', txh.hex())
                        requests.post(f'{AI_BASE}/mark_relayed/{reportId_hex}', json={'relayer': acct.address, 'tx': txh.hex()})
                    except Exception as e:
                        print('oracle submit error', e)
                else:
                    requests.post(f'{AI_BASE}/mark_relayed/{reportId_hex}', json={'relayer': 'dev-relayer', 'note': 'local-only'})
        except Exception as e:
            print('Error', e)
        time.sleep(3)

if __name__ == '__main__':
    poll_and_relay()


# Bridge demo: after submitting to Oracle, call CosmWasm RPC to trigger a cross-runtime effect
import requests
COSMWASM_RPC = os.environ.get('COSMWASM_RPC','http://cosmwasm-mock:1317/wasm/execute')

def call_cosmwasm(contract='neonet_model_registry', msg=None, sender='relayer'):
    if msg is None:
        msg = {'action':'on_report','note':'bridge_demo'}
    try:
        r = requests.post(COSMWASM_RPC, json={'contract': contract, 'msg': msg, 'sender': sender}, timeout=5)
        print('cosmwasm response', r.status_code, r.text)
    except Exception as e:
        print('cosmwasm call error', e)
