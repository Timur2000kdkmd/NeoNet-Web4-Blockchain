# Runbook - Full Automation & Cloud Deployment

## Local automated deploy (recommended steps)
1. Ensure you have Node.js, npm, Docker, docker-compose, and git installed.
2. From repository root, run:
   ```bash
   chmod +x automation/deploy_all.sh
   ./automation/deploy_all.sh --gen-keystore
   ```
   This will:
   - Start Hardhat node
   - Deploy contracts to local Hardhat node
   - Generate a keystore and update relayer/.env
   - Start docker-compose (ai-service, ai-worker, relayer, cosmwasm-mock, postgres, redis, geth)
3. Check logs:
   - Hardhat node logs: contracts/hardhat/hardhat_node.log
   - AI service: docker-compose logs ai-service
   - Relayer: docker-compose logs relayer

## Cloud deploy (high level)
1. Use Terraform skeleton to provision cluster & KMS.
2. Build and push Docker images (ai-service, relayer, cosmwasm-mock, geth/rpc if needed) to your container registry.
3. Create Kubernetes secrets for DATABASE_URL, KEYSTORE, KMS credentials.
4. Apply k8s manifests in `k8s/` and adapt to your needs (ingress, autoscaling, resource limits).
5. Use managed KMS/HSM to store private keys; configure relayer to use KMS for signing instead of local keystore.

## Security checklist
- Rotate keys periodically and enforce least privilege.
- Run smart contract audits and formal verification.
- Use rate-limiting and mutual TLS between services.
- Enforce network policies in Kubernetes and enable monitoring & alerting.


## On-chain AI training
The ai-trainer process reads blocks from the ETH node (geth) and performs incremental deterministic training on simple features extracted from transactions. In production, replace trainer logic with real ML pipelines (PyTorch/TensorFlow) and use deterministic seeding and checkpointing. Use GPUs and data sharding for scale.

## Monitoring
- Helm chart includes HPA and Prometheus ServiceMonitor template. Deploy kube-prometheus-stack for full Prometheus/Grafana monitoring.
