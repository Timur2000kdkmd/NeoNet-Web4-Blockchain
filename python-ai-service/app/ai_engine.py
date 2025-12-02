"""
AI Engine для NeoNet Web4 с Proof of Intelligence
Включает: PyTorch тренировку, fraud detection, federated learning
"""
import torch
import torch.nn as nn
import torch.optim as optim
from typing import List, Dict, Any, Tuple
import numpy as np
import json
import hashlib
from collections import defaultdict

class FraudDetectionModel(nn.Module):
    """Neural network for transaction fraud detection"""
    def __init__(self, input_size=10):
        super(FraudDetectionModel, self).__init__()
        self.fc1 = nn.Linear(input_size, 64)
        self.fc2 = nn.Linear(64, 32)
        self.fc3 = nn.Linear(32, 16)
        self.fc4 = nn.Linear(16, 1)
        self.dropout = nn.Dropout(0.2)
        self.relu = nn.ReLU()
        self.sigmoid = nn.Sigmoid()
    
    def forward(self, x):
        x = self.relu(self.fc1(x))
        x = self.dropout(x)
        x = self.relu(self.fc2(x))
        x = self.dropout(x)
        x = self.relu(self.fc3(x))
        x = self.sigmoid(self.fc4(x))
        return x

class GasOptimizerModel(nn.Module):
    """Neural network для оптимизации газа"""
    def __init__(self, input_size=8):
        super(GasOptimizerModel, self).__init__()
        self.fc1 = nn.Linear(input_size, 32)
        self.fc2 = nn.Linear(32, 16)
        self.fc3 = nn.Linear(16, 1)
        self.relu = nn.ReLU()
    
    def forward(self, x):
        x = self.relu(self.fc1(x))
        x = self.relu(self.fc2(x))
        x = self.fc3(x)
        return x

class ProofOfIntelligenceEngine:
    """Proof of Intelligence - консенсус через AI валидацию"""
    def __init__(self):
        self.fraud_model = FraudDetectionModel()
        self.gas_model = GasOptimizerModel()
        self.trained_models = {}
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        self.fraud_model.to(self.device)
        self.gas_model.to(self.device)
        
        self.fraud_threshold = 0.5
        self.intelligence_score = 0.0
    
    def extract_tx_features(self, tx: Dict[str, Any]) -> torch.Tensor:
        """Извлечение фич из транзакции для fraud detection"""
        features = [
            float(len(tx.get('data', ''))),
            float(tx.get('from_addr_age', 0)),
            float(tx.get('to_addr_age', 0)),
            float(tx.get('amount', 0)),
            float(tx.get('gas_price', 0)),
            float(tx.get('gas_limit', 0)),
            float(tx.get('nonce', 0)),
            float(len(tx.get('from', ''))),
            float(len(tx.get('to', ''))),
            float(hash(str(tx)) % 1000) / 1000.0
        ]
        return torch.tensor(features, dtype=torch.float32).to(self.device)
    
    def detect_fraud(self, tx: Dict[str, Any]) -> Tuple[bool, float]:
        """Детекция фрода с использованием ML"""
        self.fraud_model.eval()
        with torch.no_grad():
            features = self.extract_tx_features(tx)
            fraud_prob = self.fraud_model(features).item()
            is_fraud = fraud_prob > self.fraud_threshold
            return is_fraud, fraud_prob
    
    def optimize_gas(self, tx: Dict[str, Any]) -> int:
        """Оптимизация газа с помощью ML"""
        self.gas_model.eval()
        with torch.no_grad():
            features = torch.tensor([
                float(len(tx.get('data', ''))),
                float(tx.get('complexity', 1)),
                float(tx.get('storage_writes', 0)),
                float(tx.get('calls', 0)),
                float(tx.get('loops', 0)),
                float(len(tx.get('code', ''))),
                float(tx.get('current_gas_price', 100)),
                float(hash(str(tx)) % 100)
            ], dtype=torch.float32).to(self.device)
            
            optimized_gas = max(21000, int(self.gas_model(features).item() * 10000))
            return optimized_gas
    
    def validate_block(self, block: Dict[str, Any]) -> Tuple[bool, float]:
        """Валидация блока через AI (Proof of Intelligence)"""
        txs = block.get('transactions', [])
        if not txs:
            return True, 1.0
        
        fraud_count = 0
        total_confidence = 0.0
        
        for tx in txs:
            is_fraud, fraud_prob = self.detect_fraud(tx)
            if is_fraud:
                fraud_count += 1
            total_confidence += (1.0 - fraud_prob)
        
        block_valid = fraud_count == 0
        confidence = total_confidence / len(txs) if txs else 1.0
        
        self.intelligence_score = confidence
        return block_valid, confidence
    
    def calculate_mining_reward(self, intelligence_score: float, base_reward: float = 10.0) -> float:
        """Расчёт награды на основе intelligence score"""
        return base_reward * intelligence_score
    
    def train_fraud_detector(self, txs: List[Dict], labels: List[int], epochs=10):
        """Федеративное обучение fraud detector"""
        self.fraud_model.train()
        criterion = nn.BCELoss()
        optimizer = optim.Adam(self.fraud_model.parameters(), lr=0.001)
        
        for epoch in range(epochs):
            total_loss = 0.0
            for tx, label in zip(txs, labels):
                features = self.extract_tx_features(tx)
                optimizer.zero_grad()
                output = self.fraud_model(features)
                target = torch.tensor([float(label)], dtype=torch.float32).to(self.device)
                loss = criterion(output, target)
                loss.backward()
                optimizer.step()
                total_loss += loss.item()
            
            avg_loss = total_loss / len(txs) if txs else 0
            print(f"Epoch {epoch+1}/{epochs}, Loss: {avg_loss:.4f}")
        
        return self.fraud_model.state_dict()
    
    def federated_aggregate(self, model_updates: List[Dict]) -> Dict:
        """Федеративная агрегация моделей от разных нод"""
        if not model_updates:
            return self.fraud_model.state_dict()
        
        aggregated = {}
        for key in model_updates[0].keys():
            aggregated[key] = torch.stack([update[key] for update in model_updates]).mean(dim=0)
        
        self.fraud_model.load_state_dict(aggregated)
        return aggregated

class DualGovernance:
    """DualGov: AI + DAO управление сетью"""
    def __init__(self, ai_engine: ProofOfIntelligenceEngine):
        self.ai_engine = ai_engine
        self.proposals = {}
        self.votes = defaultdict(lambda: {'for': 0, 'against': 0, 'ai_score': 0.0})
    
    def create_proposal(self, proposal_id: str, description: str, proposer: str) -> Dict:
        """Создание предложения для голосования"""
        self.proposals[proposal_id] = {
            'id': proposal_id,
            'description': description,
            'proposer': proposer,
            'status': 'active',
            'ai_recommendation': None
        }
        return self.proposals[proposal_id]
    
    def ai_analyze_proposal(self, proposal_id: str) -> float:
        """AI анализ предложения"""
        if proposal_id not in self.proposals:
            return 0.0
        
        proposal = self.proposals[proposal_id]
        description = proposal['description']
        
        complexity = len(description.split())
        sentiment_score = (hash(description) % 100) / 100.0
        ai_score = min(1.0, (complexity / 50.0) * sentiment_score)
        
        self.proposals[proposal_id]['ai_recommendation'] = ai_score
        return ai_score
    
    def vote(self, proposal_id: str, voter: str, support: bool, weight: float = 1.0):
        """Голосование (DAO часть)"""
        if support:
            self.votes[proposal_id]['for'] += weight
        else:
            self.votes[proposal_id]['against'] += weight
    
    def execute_proposal(self, proposal_id: str, ai_weight: float = 0.3) -> bool:
        """Выполнение предложения с учётом AI + DAO голосов"""
        if proposal_id not in self.proposals:
            return False
        
        ai_score = self.ai_analyze_proposal(proposal_id)
        dao_votes = self.votes[proposal_id]
        
        dao_support = dao_votes['for'] / (dao_votes['for'] + dao_votes['against'] + 0.001)
        
        final_score = (ai_score * ai_weight) + (dao_support * (1 - ai_weight))
        
        if final_score > 0.5:
            self.proposals[proposal_id]['status'] = 'executed'
            return True
        else:
            self.proposals[proposal_id]['status'] = 'rejected'
            return False

def create_ai_powered_network():
    """Создание AI-управляемой сети (Web4)"""
    ai_engine = ProofOfIntelligenceEngine()
    governance = DualGovernance(ai_engine)
    
    return {
        'ai_engine': ai_engine,
        'governance': governance,
        'status': 'initialized',
        'network_type': 'Web4'
    }

if __name__ == "__main__":
    network = create_ai_powered_network()
    print(f"NeoNet Web4 Network initialized with Proof of Intelligence")
    
    test_tx = {
        'from': 'alice',
        'to': 'bob',
        'amount': 100,
        'data': 'transfer',
        'gas_price': 20,
        'gas_limit': 21000,
        'nonce': 1
    }
    
    is_fraud, fraud_prob = network['ai_engine'].detect_fraud(test_tx)
    print(f"Fraud detection: {is_fraud}, probability: {fraud_prob:.4f}")
    
    optimized_gas = network['ai_engine'].optimize_gas(test_tx)
    print(f"Optimized gas: {optimized_gas}")
