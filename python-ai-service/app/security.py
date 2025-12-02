"""
Security Module для NeoNet - защита от атак и изоляция контрактов
"""
import hashlib
import time
from typing import Dict, List, Optional, Set
from collections import defaultdict, deque
import asyncio

class NodeAttestation:
    """Аттестация узлов сети для предотвращения Sybil атак"""
    
    def __init__(self):
        self.attested_nodes: Dict[str, Dict] = {}
        self.attestation_challenges: Dict[str, Dict] = {}
        self.min_stake_required = 1000  # Минимальный stake для аттестации
        
    def create_challenge(self, node_id: str) -> Dict:
        """Создать challenge для аттестации ноды"""
        challenge = hashlib.sha256(
            f"{node_id}{time.time()}".encode()
        ).hexdigest()
        
        self.attestation_challenges[node_id] = {
            "challenge": challenge,
            "timestamp": time.time(),
            "solved": False
        }
        
        return {
            "node_id": node_id,
            "challenge": challenge,
            "expires_in": 300  # 5 минут
        }
    
    def verify_attestation(self, node_id: str, response: str, stake: int) -> Dict:
        """Проверить ответ на challenge и stake"""
        
        if node_id not in self.attestation_challenges:
            return {"status": "error", "message": "No challenge found"}
        
        challenge_data = self.attestation_challenges[node_id]
        
        # Проверка timeout
        if time.time() - challenge_data["timestamp"] > 300:
            return {"status": "error", "message": "Challenge expired"}
        
        # Проверка stake
        if stake < self.min_stake_required:
            return {"status": "error", "message": f"Insufficient stake: {stake} < {self.min_stake_required}"}
        
        # Простая проверка (в production здесь будет криптографическая проверка)
        expected_response = hashlib.sha256(
            challenge_data["challenge"].encode()
        ).hexdigest()
        
        if response == expected_response:
            self.attested_nodes[node_id] = {
                "stake": stake,
                "attested_at": time.time(),
                "reputation": 100
            }
            challenge_data["solved"] = True
            
            return {
                "status": "success",
                "node_id": node_id,
                "attested": True,
                "reputation": 100
            }
        else:
            return {"status": "error", "message": "Invalid attestation response"}
    
    def is_attested(self, node_id: str) -> bool:
        """Проверить аттестован ли узел"""
        return node_id in self.attested_nodes
    
    def get_reputation(self, node_id: str) -> int:
        """Получить репутацию узла"""
        if node_id in self.attested_nodes:
            return self.attested_nodes[node_id]["reputation"]
        return 0

class RateLimiter:
    """Rate limiting для защиты от DoS атак"""
    
    def __init__(self, max_requests: int = 100, window_seconds: int = 60):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.request_history: Dict[str, deque] = defaultdict(deque)
        self.blocked_ips: Set[str] = set()
        self.block_duration = 300  # 5 минут блокировки
        self.block_times: Dict[str, float] = {}
    
    def check_rate_limit(self, client_id: str) -> Dict:
        """Проверить rate limit для клиента"""
        
        # Проверка на блокировку
        if client_id in self.blocked_ips:
            if time.time() - self.block_times[client_id] > self.block_duration:
                # Разблокировать после истечения времени
                self.blocked_ips.remove(client_id)
                del self.block_times[client_id]
            else:
                remaining = int(self.block_duration - (time.time() - self.block_times[client_id]))
                return {
                    "allowed": False,
                    "reason": "blocked",
                    "unblock_in": remaining
                }
        
        current_time = time.time()
        history = self.request_history[client_id]
        
        # Удалить старые запросы за пределами окна
        while history and history[0] < current_time - self.window_seconds:
            history.popleft()
        
        # Проверить лимит
        if len(history) >= self.max_requests:
            # Превышен лимит - блокировать
            self.blocked_ips.add(client_id)
            self.block_times[client_id] = current_time
            
            return {
                "allowed": False,
                "reason": "rate_limit_exceeded",
                "max_requests": self.max_requests,
                "window_seconds": self.window_seconds,
                "blocked_for": self.block_duration
            }
        
        # Записать новый запрос
        history.append(current_time)
        
        return {
            "allowed": True,
            "requests_remaining": self.max_requests - len(history),
            "window_seconds": self.window_seconds
        }

class ContractSandbox:
    """Sandbox изоляция для выполнения смарт-контрактов"""
    
    def __init__(self):
        self.max_execution_time = 10  # секунд
        self.max_memory_mb = 256
        self.max_storage_ops = 1000
        self.forbidden_operations = [
            "subprocess",
            "os.system",
            "eval",
            "exec",
            "__import__"
        ]
    
    def validate_contract_code(self, code: str) -> Dict:
        """Валидация кода контракта перед выполнением"""
        
        violations = []
        
        # Проверка на запрещенные операции
        for forbidden in self.forbidden_operations:
            if forbidden in code:
                violations.append(f"Forbidden operation: {forbidden}")
        
        # Проверка длины кода
        if len(code) > 100000:  # 100KB max
            violations.append("Code too large")
        
        if violations:
            return {
                "valid": False,
                "violations": violations
            }
        
        return {
            "valid": True,
            "size_bytes": len(code.encode()),
            "max_execution_time": self.max_execution_time,
            "max_memory_mb": self.max_memory_mb
        }
    
    async def execute_sandboxed(self, contract_address: str, method: str, 
                                 args: List, timeout: Optional[int] = None) -> Dict:
        """Выполнение контракта в изолированной среде с timeout"""
        
        execution_timeout = timeout or self.max_execution_time
        
        try:
            # В production здесь будет реальная изоляция через контейнер/VM
            # Сейчас используем asyncio timeout для симуляции
            async def _execute():
                # Имитация выполнения контракта
                await asyncio.sleep(0.1)
                return {
                    "status": "success",
                    "result": f"Method {method} executed with {len(args)} args",
                    "gas_used": 21000,
                    "execution_time_ms": 100
                }
            
            result = await asyncio.wait_for(_execute(), timeout=execution_timeout)
            return result
            
        except asyncio.TimeoutError:
            return {
                "status": "error",
                "error": "execution_timeout",
                "timeout_seconds": execution_timeout
            }
        except Exception as e:
            return {
                "status": "error",
                "error": str(e)
            }

class SecurityMonitor:
    """Мониторинг безопасности и обнаружение аномалий"""
    
    def __init__(self):
        self.anomaly_scores: Dict[str, float] = {}
        self.suspicious_patterns: List[Dict] = []
        self.alert_threshold = 0.8
    
    def analyze_transaction_pattern(self, tx_data: Dict) -> Dict:
        """Анализ паттернов транзакций для обнаружения аномалий"""
        
        anomaly_score = 0.0
        flags = []
        
        # Проверка на подозрительные паттерны
        if "value" in tx_data and tx_data["value"] > 1000000:
            anomaly_score += 0.3
            flags.append("high_value_transfer")
        
        if "gas_price" in tx_data and tx_data["gas_price"] > 500:
            anomaly_score += 0.2
            flags.append("unusually_high_gas")
        
        # Проверка на rapid transactions
        sender = tx_data.get("from", "unknown")
        if sender in self.anomaly_scores:
            if self.anomaly_scores[sender] > self.alert_threshold:
                anomaly_score += 0.4
                flags.append("high_frequency_sender")
        
        self.anomaly_scores[sender] = anomaly_score
        
        if anomaly_score > self.alert_threshold:
            self.suspicious_patterns.append({
                "tx_hash": tx_data.get("hash", "unknown"),
                "anomaly_score": anomaly_score,
                "flags": flags,
                "timestamp": time.time()
            })
        
        return {
            "anomaly_score": anomaly_score,
            "is_suspicious": anomaly_score > self.alert_threshold,
            "flags": flags,
            "recommendation": "block" if anomaly_score > 0.9 else "monitor"
        }
    
    def get_security_report(self) -> Dict:
        """Получить отчет по безопасности"""
        return {
            "total_monitored_addresses": len(self.anomaly_scores),
            "suspicious_patterns_detected": len(self.suspicious_patterns),
            "high_risk_addresses": sum(1 for score in self.anomaly_scores.values() if score > self.alert_threshold),
            "recent_suspicious": self.suspicious_patterns[-10:] if self.suspicious_patterns else []
        }

# Global instances
attestation = NodeAttestation()
rate_limiter = RateLimiter(max_requests=100, window_seconds=60)
sandbox = ContractSandbox()
security_monitor = SecurityMonitor()
