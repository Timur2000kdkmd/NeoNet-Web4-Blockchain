"""
Federated Learning для NeoNet - распределенное обучение AI моделей
"""
import torch
import torch.nn as nn
import torch.optim as optim
from typing import Dict, List, Any, Optional
import numpy as np
import json
from datetime import datetime

class FraudDetectionModel(nn.Module):
    """Neural network для детекции мошенничества в транзакциях"""
    def __init__(self, input_size: int = 10):
        super().__init__()
        self.fc1 = nn.Linear(input_size, 64)
        self.fc2 = nn.Linear(64, 32)
        self.fc3 = nn.Linear(32, 16)
        self.fc4 = nn.Linear(16, 1)
        self.relu = nn.ReLU()
        self.sigmoid = nn.Sigmoid()
        
    def forward(self, x):
        x = self.relu(self.fc1(x))
        x = self.relu(self.fc2(x))
        x = self.relu(self.fc3(x))
        x = self.sigmoid(self.fc4(x))
        return x

class FederatedLearningEngine:
    """Engine для federated learning в NeoNet"""
    
    def __init__(self, model_type: str = "fraud_detection"):
        self.model_type = model_type
        self.global_model = FraudDetectionModel()
        self.node_models: Dict[str, Dict[str, Any]] = {}
        self.training_rounds = 0
        self.aggregation_method = "fedavg"  # FedAvg algorithm
        
    def get_global_model_weights(self) -> Dict[str, List[float]]:
        """Получить веса глобальной модели для распространения"""
        state_dict = self.global_model.state_dict()
        weights = {}
        for key, tensor in state_dict.items():
            weights[key] = tensor.cpu().numpy().tolist()
        return weights
    
    def register_node(self, node_id: str) -> Dict[str, Any]:
        """Регистрация ноды для участия в federated learning"""
        if node_id not in self.node_models:
            self.node_models[node_id] = {
                "registered_at": datetime.utcnow().isoformat(),
                "training_rounds": 0,
                "last_update": None,
                "contribution_score": 0.0
            }
        
        return {
            "node_id": node_id,
            "model_weights": self.get_global_model_weights(),
            "training_round": self.training_rounds,
            "status": "registered"
        }
    
    def train_local_model(self, node_id: str, training_data: List[Dict[str, Any]], 
                          epochs: int = 5, learning_rate: float = 0.001) -> Dict[str, Any]:
        """Локальное обучение модели на данных ноды"""
        
        # Создаем локальную копию глобальной модели
        local_model = FraudDetectionModel()
        local_model.load_state_dict(self.global_model.state_dict())
        
        # Подготовка данных
        if not training_data:
            return {
                "status": "error",
                "message": "No training data provided"
            }
        
        # Простая симуляция обучения (для production нужны real данные)
        X_train = []
        y_train = []
        
        for sample in training_data:
            # Преобразуем features в тензор
            features = sample.get("features", [0.0] * 10)
            label = sample.get("is_fraud", 0)
            
            X_train.append(features)
            y_train.append([float(label)])
        
        X = torch.tensor(X_train, dtype=torch.float32)
        y = torch.tensor(y_train, dtype=torch.float32)
        
        # Training loop
        criterion = nn.BCELoss()
        optimizer = optim.Adam(local_model.parameters(), lr=learning_rate)
        
        losses = []
        for epoch in range(epochs):
            optimizer.zero_grad()
            outputs = local_model(X)
            loss = criterion(outputs, y)
            loss.backward()
            optimizer.step()
            losses.append(loss.item())
        
        # Получаем обученные веса
        local_weights = {}
        for key, tensor in local_model.state_dict().items():
            local_weights[key] = tensor.cpu().numpy().tolist()
        
        # Обновляем статистику ноды
        if node_id in self.node_models:
            self.node_models[node_id]["training_rounds"] += 1
            self.node_models[node_id]["last_update"] = datetime.utcnow().isoformat()
            self.node_models[node_id]["contribution_score"] += len(training_data)
        
        return {
            "status": "success",
            "node_id": node_id,
            "local_weights": local_weights,
            "training_samples": len(training_data),
            "epochs": epochs,
            "final_loss": losses[-1] if losses else 0.0,
            "avg_loss": np.mean(losses) if losses else 0.0
        }
    
    def aggregate_models(self, node_updates: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Агрегация моделей от разных нод (FedAvg algorithm)"""
        
        if not node_updates:
            return {
                "status": "error",
                "message": "No node updates to aggregate"
            }
        
        # FedAvg: взвешенное усреднение по количеству обучающих примеров
        total_samples = sum(update.get("training_samples", 0) for update in node_updates)
        
        if total_samples == 0:
            return {
                "status": "error",
                "message": "No training samples in updates"
            }
        
        # Инициализируем аккумуляторы для новых весов
        aggregated_weights = {}
        
        # Получаем структуру весов из первого обновления
        first_update = node_updates[0]
        local_weights = first_update.get("local_weights", {})
        
        for key in local_weights.keys():
            aggregated_weights[key] = np.zeros_like(np.array(local_weights[key]))
        
        # Взвешенное усреднение
        for update in node_updates:
            node_weights = update.get("local_weights", {})
            samples = update.get("training_samples", 0)
            weight_factor = samples / total_samples
            
            for key, weights in node_weights.items():
                aggregated_weights[key] += weight_factor * np.array(weights)
        
        # Обновляем глобальную модель
        new_state_dict = {}
        for key, weights in aggregated_weights.items():
            new_state_dict[key] = torch.tensor(weights, dtype=torch.float32)
        
        self.global_model.load_state_dict(new_state_dict)
        self.training_rounds += 1
        
        return {
            "status": "success",
            "training_round": self.training_rounds,
            "nodes_contributed": len(node_updates),
            "total_samples": total_samples,
            "aggregation_method": self.aggregation_method,
            "timestamp": datetime.utcnow().isoformat()
        }
    
    def predict(self, features: List[float]) -> Dict[str, Any]:
        """Предсказание используя глобальную модель"""
        self.global_model.eval()
        
        with torch.no_grad():
            X = torch.tensor([features], dtype=torch.float32)
            prediction = self.global_model(X)
            fraud_score = prediction.item()
        
        return {
            "fraud_probability": fraud_score,
            "is_fraud": fraud_score > 0.5,
            "confidence": abs(fraud_score - 0.5) * 2,  # 0 to 1 scale
            "model_version": self.training_rounds
        }
    
    def get_statistics(self) -> Dict[str, Any]:
        """Статистика federated learning"""
        return {
            "training_rounds": self.training_rounds,
            "registered_nodes": len(self.node_models),
            "aggregation_method": self.aggregation_method,
            "model_type": self.model_type,
            "nodes": {
                node_id: {
                    "training_rounds": info["training_rounds"],
                    "contribution_score": info["contribution_score"],
                    "last_update": info["last_update"]
                }
                for node_id, info in self.node_models.items()
            }
        }

# Global instance
fl_engine = FederatedLearningEngine()
