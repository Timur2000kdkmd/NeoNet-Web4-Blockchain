# relayer-metrics.py
from prometheus_client import start_http_server, Counter, Gauge
import time, os

REPORTS = Counter('neonet_reports_relayed_total', 'Total reports relayed')
LAST_TX = Gauge('neonet_last_tx_time', 'Last relayer tx time')

def run(port=9101):
    start_http_server(port)
    while True:
        # update metrics from env or log scraping
        time.sleep(5)

if __name__ == '__main__':
    run()
