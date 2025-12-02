# Running PyTorch trainer

## Local (CPU)
1. Install dependencies:
   pip install -r python-ai-service/requirements.txt
   pip install torch torchvision  # CPU wheel

2. Prepare dataset:
   - Ensure geth is running and `scripts/ingest_block_to_dataset.py` can fetch blocks
   - Run ingestion: python scripts/ingest_block_to_dataset.py

3. Run trainer:
   python python-ai-service/pytorch_trainer.py

## Kubernetes (recommended for scale)
- Build image with PyTorch (choose the appropriate base image with CUDA if using GPUs).
- Push to registry and set `helm/neonet/values.yaml` to enable GPUs and appropriate resources.
- Deploy via Helm and mount secrets (KMS/Key Vault) and persistent volumes for checkpoints and dataset.
