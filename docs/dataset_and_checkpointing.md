# Dataset management and checkpointing

- Transactions are ingested via `scripts/ingest_block_to_dataset.py` into `NEONET_DATA_DIR` (default `/data/transactions`).
- Each file is jsonl with one transaction per line: this allows efficient streaming and incremental dataset build.
- The PyTorch trainer (`python-ai-service/pytorch_trainer.py`) reads `.jsonl` files and trains a small model, saving checkpoints to `NEONET_CHECKPOINT_DIR` (default `/data/checkpoints`).
- For large-scale training:
  - Use sharded storage (S3/GCS) and data loaders that stream.
  - Use distributed training (torch.distributed) across GPU instances.
  - Store checkpoints in object storage and use a model registry (MLflow, Weights & Biases).
