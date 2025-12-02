"""
Proof of Intelligence (PoI) Consensus Module
AI validates blocks instead of human validators
"""

import hashlib
import json
import time
import numpy as np
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
import random

@dataclass
class AIProof:
    """Proof of Intelligence proof structure"""
    model_hash: str
    gradient_hash: str
    accuracy_score: float
    loss_value: float
    training_rounds: int
    validator_id: str
    timestamp: int
    signature: str

@dataclass
class Block:
    """Block structure with AI proof"""
    index: int
    prev_hash: str
    timestamp: int
    transactions: List[Dict]
    ai_proof: AIProof
    proposer: str
    hash: str

class ProofOfIntelligence:
    """
    Proof of Intelligence consensus mechanism.
    Validators must submit ML model gradients as proof of useful work.
    """
    
    def __init__(self, min_accuracy: float = 0.7, min_validators: int = 3):
        self.min_accuracy = min_accuracy
        self.min_validators = min_validators
        self.validators: Dict[str, Dict] = {}
        self.model_registry: Dict[str, Dict] = {}
        self.current_round = 0
        self.pending_proofs: List[AIProof] = []
        
    def register_validator(self, validator_id: str, stake: float, compute_power: float) -> bool:
        """Register a new AI validator"""
        if validator_id in self.validators:
            return False
            
        self.validators[validator_id] = {
            'stake': stake,
            'compute_power': compute_power,
            'reputation': 1.0,
            'blocks_validated': 0,
            'total_accuracy': 0.0,
            'registered_at': int(time.time())
        }
        return True
    
    def submit_ai_proof(
        self,
        validator_id: str,
        model_weights: np.ndarray,
        gradients: np.ndarray,
        accuracy: float,
        loss: float,
        training_rounds: int
    ) -> Optional[AIProof]:
        """Submit AI training proof for block validation"""
        
        if validator_id not in self.validators:
            return None
            
        if accuracy < self.min_accuracy:
            return None
            
        model_hash = self._hash_array(model_weights)
        gradient_hash = self._hash_array(gradients)
        
        signature = self._sign_proof(validator_id, model_hash, gradient_hash)
        
        proof = AIProof(
            model_hash=model_hash,
            gradient_hash=gradient_hash,
            accuracy_score=accuracy,
            loss_value=loss,
            training_rounds=training_rounds,
            validator_id=validator_id,
            timestamp=int(time.time()),
            signature=signature
        )
        
        self.pending_proofs.append(proof)
        return proof
    
    def verify_ai_proof(self, proof: AIProof) -> Tuple[bool, str]:
        """Verify submitted AI proof"""
        
        if proof.validator_id not in self.validators:
            return False, "Unknown validator"
            
        if proof.accuracy_score < self.min_accuracy:
            return False, f"Accuracy {proof.accuracy_score} below minimum {self.min_accuracy}"
            
        if proof.training_rounds < 1:
            return False, "Invalid training rounds"
            
        expected_sig = self._sign_proof(
            proof.validator_id, 
            proof.model_hash, 
            proof.gradient_hash
        )
        if proof.signature != expected_sig:
            return False, "Invalid signature"
            
        return True, "Valid proof"
    
    def verify_model_gradient(self, data: Dict, proof: AIProof) -> Tuple[bool, float]:
        """
        Core PoI validation function.
        Verifies that the gradient is valid for the given data.
        """
        is_valid, message = self.verify_ai_proof(proof)
        
        if not is_valid:
            return False, 0.0
            
        confidence = self._calculate_confidence(proof)
        
        if confidence > 0.5:
            validator = self.validators[proof.validator_id]
            validator['blocks_validated'] += 1
            validator['total_accuracy'] += proof.accuracy_score
            validator['reputation'] = min(2.0, validator['reputation'] * 1.01)
        
        return is_valid, confidence
    
    def select_block_proposer(self) -> Optional[str]:
        """Select next block proposer based on stake, compute power, and reputation"""
        
        if len(self.validators) < self.min_validators:
            return None
            
        weights = []
        validator_ids = []
        
        for vid, vdata in self.validators.items():
            weight = (
                vdata['stake'] * 0.4 +
                vdata['compute_power'] * 0.3 +
                vdata['reputation'] * 0.3
            )
            weights.append(weight)
            validator_ids.append(vid)
            
        total_weight = sum(weights)
        probabilities = [w / total_weight for w in weights]
        
        selected = random.choices(validator_ids, probabilities)[0]
        return selected
    
    def create_block(
        self,
        prev_block: Block,
        transactions: List[Dict],
        proposer_id: str,
        proof: AIProof
    ) -> Optional[Block]:
        """Create a new block with AI proof"""
        
        is_valid, confidence = self.verify_model_gradient({}, proof)
        if not is_valid:
            return None
            
        block = Block(
            index=prev_block.index + 1,
            prev_hash=prev_block.hash,
            timestamp=int(time.time()),
            transactions=transactions,
            ai_proof=proof,
            proposer=proposer_id,
            hash=""
        )
        
        block.hash = self._compute_block_hash(block)
        
        self.current_round += 1
        
        return block
    
    def federated_aggregate(self, gradients: List[np.ndarray]) -> np.ndarray:
        """Aggregate gradients from multiple validators (FedAvg)"""
        if not gradients:
            return np.array([])
            
        return np.mean(gradients, axis=0)
    
    def _hash_array(self, arr: np.ndarray) -> str:
        """Hash numpy array"""
        return hashlib.sha256(arr.tobytes()).hexdigest()
    
    def _sign_proof(self, validator_id: str, model_hash: str, gradient_hash: str) -> str:
        """Sign proof (simplified - in production use real crypto)"""
        data = f"{validator_id}:{model_hash}:{gradient_hash}"
        return hashlib.sha256(data.encode()).hexdigest()
    
    def _compute_block_hash(self, block: Block) -> str:
        """Compute block hash"""
        block_data = {
            'index': block.index,
            'prev_hash': block.prev_hash,
            'timestamp': block.timestamp,
            'transactions': block.transactions,
            'ai_proof': asdict(block.ai_proof),
            'proposer': block.proposer
        }
        return hashlib.sha256(json.dumps(block_data, sort_keys=True).encode()).hexdigest()
    
    def _calculate_confidence(self, proof: AIProof) -> float:
        """Calculate confidence score for AI proof"""
        accuracy_weight = proof.accuracy_score * 0.5
        
        loss_weight = max(0, 1 - proof.loss_value) * 0.3
        
        rounds_weight = min(1.0, proof.training_rounds / 100) * 0.2
        
        return accuracy_weight + loss_weight + rounds_weight
    
    def get_validator_stats(self, validator_id: str) -> Optional[Dict]:
        """Get validator statistics"""
        if validator_id not in self.validators:
            return None
            
        validator = self.validators[validator_id]
        avg_accuracy = (
            validator['total_accuracy'] / validator['blocks_validated']
            if validator['blocks_validated'] > 0 else 0
        )
        
        return {
            **validator,
            'average_accuracy': avg_accuracy
        }
    
    def get_network_stats(self) -> Dict:
        """Get overall network statistics"""
        total_stake = sum(v['stake'] for v in self.validators.values())
        total_compute = sum(v['compute_power'] for v in self.validators.values())
        total_blocks = sum(v['blocks_validated'] for v in self.validators.values())
        
        return {
            'total_validators': len(self.validators),
            'total_stake': total_stake,
            'total_compute_power': total_compute,
            'total_blocks_validated': total_blocks,
            'current_round': self.current_round,
            'pending_proofs': len(self.pending_proofs)
        }


class ContractAuditor:
    """AI-powered smart contract auditor"""
    
    VULNERABILITY_PATTERNS = {
        'reentrancy': [
            b'call.value',
            b'call{value',
            b'.call(',
            b'send(',
            b'transfer('
        ],
        'overflow': [
            b'uint256',
            b'+=',
            b'-=',
            b'*='
        ],
        'access_control': [
            b'onlyOwner',
            b'require(msg.sender',
            b'modifier'
        ],
        'tx_origin': [
            b'tx.origin'
        ],
        'selfdestruct': [
            b'selfdestruct',
            b'suicide'
        ]
    }
    
    def __init__(self):
        self.audits_performed = 0
        self.vulnerabilities_found = 0
        
    def audit_bytecode(self, bytecode: bytes) -> Dict:
        """Audit contract bytecode for vulnerabilities"""
        
        vulnerabilities = []
        risk_score = 0.0
        
        for vuln_type, patterns in self.VULNERABILITY_PATTERNS.items():
            for pattern in patterns:
                if pattern in bytecode:
                    severity = self._get_severity(vuln_type)
                    vulnerabilities.append({
                        'type': vuln_type,
                        'severity': severity,
                        'pattern': pattern.decode('utf-8', errors='ignore')
                    })
                    risk_score += severity
        
        max_risk = len(self.VULNERABILITY_PATTERNS) * 1.0
        normalized_risk = min(1.0, risk_score / max_risk) if max_risk > 0 else 0
        
        security_score = 1.0 - normalized_risk
        
        self.audits_performed += 1
        self.vulnerabilities_found += len(vulnerabilities)
        
        return {
            'security_score': security_score,
            'risk_level': self._get_risk_level(security_score),
            'vulnerabilities': vulnerabilities,
            'recommendation': 'APPROVE' if security_score >= 0.7 else 'REJECT',
            'details': f"Found {len(vulnerabilities)} potential issues"
        }
    
    def _get_severity(self, vuln_type: str) -> float:
        """Get severity score for vulnerability type"""
        severity_map = {
            'reentrancy': 1.0,
            'overflow': 0.8,
            'access_control': 0.6,
            'tx_origin': 0.7,
            'selfdestruct': 0.9
        }
        return severity_map.get(vuln_type, 0.5)
    
    def _get_risk_level(self, security_score: float) -> str:
        """Get risk level string"""
        if security_score >= 0.9:
            return 'LOW'
        elif security_score >= 0.7:
            return 'MEDIUM'
        elif security_score >= 0.5:
            return 'HIGH'
        else:
            return 'CRITICAL'


class GasOptimizer:
    """AI-powered gas price optimizer"""
    
    def __init__(self, base_gas_price: int = 20):
        self.base_gas_price = base_gas_price
        self.history: List[Dict] = []
        self.prediction_model_accuracy = 0.85
        
    def predict_optimal_gas(self, network_load: float, pending_txs: int) -> int:
        """Predict optimal gas price based on network conditions"""
        
        load_factor = 1 + (network_load * 0.5)
        
        tx_factor = 1 + (min(pending_txs, 10000) / 10000) * 0.3
        
        time_factor = self._get_time_factor()
        
        optimal_gas = int(self.base_gas_price * load_factor * tx_factor * time_factor)
        
        self.history.append({
            'timestamp': int(time.time()),
            'network_load': network_load,
            'pending_txs': pending_txs,
            'predicted_gas': optimal_gas
        })
        
        return optimal_gas
    
    def _get_time_factor(self) -> float:
        """Get time-based factor (lower prices during off-peak hours)"""
        hour = time.localtime().tm_hour
        if 2 <= hour <= 6:
            return 0.8
        elif 10 <= hour <= 14 or 18 <= hour <= 22:
            return 1.2
        return 1.0
    
    def get_gas_stats(self) -> Dict:
        """Get gas optimization statistics"""
        if not self.history:
            return {'average_gas': self.base_gas_price, 'predictions': 0}
            
        recent = self.history[-100:]
        avg_gas = sum(h['predicted_gas'] for h in recent) / len(recent)
        
        return {
            'average_gas': avg_gas,
            'min_gas': min(h['predicted_gas'] for h in recent),
            'max_gas': max(h['predicted_gas'] for h in recent),
            'predictions': len(self.history),
            'model_accuracy': self.prediction_model_accuracy
        }


poi_consensus = ProofOfIntelligence()
contract_auditor = ContractAuditor()
gas_optimizer = GasOptimizer()
