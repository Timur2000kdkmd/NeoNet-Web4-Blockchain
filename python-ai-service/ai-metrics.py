# ai-metrics.py - simple Prometheus exporter stub for ai-service
from prometheus_client import start_http_server, Counter, Gauge
import time, os

REQUESTS = Counter('neonet_ai_requests_total', 'Total AI requests')
QUEUE_LEN = Gauge('neonet_queue_length', 'Queue length')

def run(port=8001):
    start_http_server(port)
    while True:
        # In production, fetch real queue length
        QUEUE_LEN.set(int(os.environ.get('QUEUE_LEN', '0')))
        time.sleep(5)

if __name__ == '__main__':
    run()
