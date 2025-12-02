# pq_demo.py
# Demo "post-quantum-like" signature interface.
# WARNING: This is a non-secure placeholder for demo and testing only.
# Replace with a real PQ library (e.g., PQClean / liboqs / Dilithium Python bindings)
# for production use.

import hashlib, hmac, base64

def demo_sign(secret: str, message: str) -> str:
    # naive HMAC-based signature as placeholder
    sig = hmac.new(secret.encode(), message.encode(), hashlib.sha256).digest()
    return base64.b64encode(sig).decode()

def demo_verify(pub_dummy: str, message: str, signature: str) -> bool:
    # pub_dummy is not used in this demo; in PQ libs, use public key
    expected = demo_sign(pub_dummy, message)
    return expected == signature

# Wrapper API mirroring expected PQ library
def keygen():
    # returns (pub, priv) - placeholders
    priv = 'demo-priv-key'
    pub = 'demo-priv-key'  # in real PQ, pub != priv
    return pub, priv
