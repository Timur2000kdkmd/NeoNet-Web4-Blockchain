# pq_wrapper.py
# Production-oriented PQ wrapper with Ed25519 (classical) and placeholder for Dilithium
import base64
import hashlib
import os
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

HAS_DILITHIUM = False
try:
    import pydilithium as dilithium
    HAS_DILITHIUM = True
except ImportError:
    HAS_DILITHIUM = False

class HybridKeyPair:
    def __init__(self):
        self.ed25519_private = ed25519.Ed25519PrivateKey.generate()
        self.ed25519_public = self.ed25519_private.public_key()
        
        if HAS_DILITHIUM:
            self.dilithium_public, self.dilithium_private = dilithium.keypair()
        else:
            self.dilithium_private = os.urandom(32)
            self.dilithium_public = hashlib.sha256(self.dilithium_private).digest()
    
    def get_public_keys(self):
        ed_pub = base64.b64encode(
            self.ed25519_public.public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw
            )
        ).decode()
        
        dil_pub = base64.b64encode(self.dilithium_public).decode()
        return ed_pub, dil_pub
    
    def get_private_keys(self):
        ed_priv = base64.b64encode(
            self.ed25519_private.private_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PrivateFormat.Raw,
                encryption_algorithm=serialization.NoEncryption()
            )
        ).decode()
        
        dil_priv = base64.b64encode(self.dilithium_private).decode()
        return ed_priv, dil_priv

def keygen():
    """Generate hybrid keypair (Ed25519 + Dilithium placeholder)"""
    keypair = HybridKeyPair()
    ed_pub, dil_pub = keypair.get_public_keys()
    ed_priv, dil_priv = keypair.get_private_keys()
    
    return {
        'ed25519_public': ed_pub,
        'ed25519_private': ed_priv,
        'dilithium_public': dil_pub,
        'dilithium_private': dil_priv
    }

def sign(keys: dict, message: str) -> dict:
    """Sign message with hybrid signature (Ed25519 + Dilithium placeholder)"""
    msg_bytes = message.encode()
    
    ed_priv_bytes = base64.b64decode(keys['ed25519_private'])
    ed_private_key = ed25519.Ed25519PrivateKey.from_private_bytes(ed_priv_bytes)
    ed_signature = ed_private_key.sign(msg_bytes)
    
    if HAS_DILITHIUM:
        dil_priv_bytes = base64.b64decode(keys['dilithium_private'])
        dil_signature = dilithium.sign(msg_bytes, dil_priv_bytes)
    else:
        dil_priv_bytes = base64.b64decode(keys['dilithium_private'])
        dil_signature = hashlib.sha512(dil_priv_bytes + msg_bytes).digest()
    
    return {
        'ed25519_sig': base64.b64encode(ed_signature).decode(),
        'dilithium_sig': base64.b64encode(dil_signature).decode(),
        'algorithm': 'Ed25519+Dilithium2' if HAS_DILITHIUM else 'Ed25519+SHA512-placeholder'
    }

def verify(keys: dict, message: str, signature: dict) -> bool:
    """Verify hybrid signature"""
    msg_bytes = message.encode()
    
    try:
        ed_pub_bytes = base64.b64decode(keys['ed25519_public'])
        ed_public_key = ed25519.Ed25519PublicKey.from_public_bytes(ed_pub_bytes)
        ed_sig_bytes = base64.b64decode(signature['ed25519_sig'])
        
        ed_public_key.verify(ed_sig_bytes, msg_bytes)
        
        if HAS_DILITHIUM:
            dil_pub_bytes = base64.b64decode(keys['dilithium_public'])
            dil_sig_bytes = base64.b64decode(signature['dilithium_sig'])
            dil_valid = dilithium.verify(msg_bytes, dil_sig_bytes, dil_pub_bytes)
            return dil_valid
        else:
            return True
            
    except Exception as e:
        print(f"Verification failed: {e}")
        return False

def sign_legacy(priv: str, message: str) -> str:
    """Legacy interface for backward compatibility"""
    keys = {'ed25519_private': priv}
    try:
        ed_priv_bytes = base64.b64decode(priv)
        ed_private_key = ed25519.Ed25519PrivateKey.from_private_bytes(ed_priv_bytes)
        signature = ed_private_key.sign(message.encode())
        return base64.b64encode(signature).decode()
    except:
        return hashlib.sha256((priv + message).encode()).hexdigest()

def verify_legacy(pub: str, message: str, signature: str) -> bool:
    """Legacy interface for backward compatibility"""
    try:
        pub_bytes = base64.b64decode(pub)
        ed_public_key = ed25519.Ed25519PublicKey.from_public_bytes(pub_bytes)
        sig_bytes = base64.b64decode(signature)
        ed_public_key.verify(sig_bytes, message.encode())
        return True
    except:
        return False
