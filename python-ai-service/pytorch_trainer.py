# pytorch_trainer.py
# Advanced PyTorch trainer for NeoNet.
# - Builds a small feedforward model
# - Loads dataset from disk (transactions saved as JSON lines)
# - Supports checkpointing and resume
# - Uses deterministic behavior via seeds
# Notes:
# - Install PyTorch in your environment (CPU or GPU build)
#   pip install torch torchvision
# - This script is intended to run inside ai-trainer container with access to ETH node data.

import os
import json
import random
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import hashlib
import time

DATA_DIR = os.environ.get('NEONET_DATA_DIR', '/data/transactions')
CHECKPOINT_DIR = os.environ.get('NEONET_CHECKPOINT_DIR', '/data/checkpoints')
BATCH_SIZE = int(os.environ.get('NEO_BATCH_SIZE', '32'))
LR = float(os.environ.get('NEO_LR', '1e-3'))
SEED = int(os.environ.get('NEO_SEED', '42'))
DEVICE = torch.device('cuda' if torch.cuda.is_available() else 'cpu')

random.seed(SEED)
torch.manual_seed(SEED)

class TxDataset(Dataset):
    def __init__(self, data_dir):
        self.files = []
        for fname in os.listdir(data_dir):
            if fname.endswith('.jsonl'):
                self.files.append(os.path.join(data_dir, fname))
        self.samples = []
        for f in self.files:
            with open(f,'r') as fh:
                for line in fh:
                    try:
                        self.samples.append(json.loads(line))
                    except:
                        continue
    def __len__(self):
        return len(self.samples)
    def __getitem__(self, idx):
        tx = self.samples[idx]
        # extract simple features (same as earlier trainer but numeric tensor)
        input_data = tx.get('input','')
        value = int(tx.get('value','0'),16) if tx.get('value') else 0
        to = tx.get('to','') or ''
        f1 = len(input_data)
        f2 = value % 2
        f3 = sum(bytearray(to.encode())) % 100 if to else 0
        x = torch.tensor([f1, f2, f3], dtype=torch.float32)
        # label: parity of input length
        y = torch.tensor([1.0 if f1 % 2 == 0 else 0.0], dtype=torch.float32)
        return x, y

class SimpleModel(nn.Module):
    def __init__(self, in_dim=3, hidden=16):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(in_dim, hidden),
            nn.ReLU(),
            nn.Linear(hidden, 1),
            nn.Sigmoid()
        )
    def forward(self, x):
        return self.net(x)

def save_checkpoint(model, optimizer, epoch, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    torch.save({
        'epoch': epoch,
        'model_state': model.state_dict(),
        'optim_state': optimizer.state_dict()
    }, path)
    print('Saved checkpoint', path)

def load_checkpoint(model, optimizer, path, device=DEVICE):
    if not os.path.exists(path):
        return 0
    ckpt = torch.load(path, map_location=device)
    model.load_state_dict(ckpt['model_state'])
    optimizer.load_state_dict(ckpt['optim_state'])
    print('Loaded checkpoint from', path)
    return ckpt.get('epoch', 0)

def train_loop(data_dir=DATA_DIR, checkpoint_dir=CHECKPOINT_DIR, epochs=5):
    dataset = TxDataset(data_dir)
    if len(dataset) == 0:
        print('No data found in', data_dir)
        return
    dataloader = DataLoader(dataset, batch_size=BATCH_SIZE, shuffle=True)
    model = SimpleModel().to(DEVICE)
    optimizer = optim.Adam(model.parameters(), lr=LR)
    loss_fn = nn.BCELoss()
    last_ckpt = os.path.join(checkpoint_dir, 'last.pt')
    start_epoch = load_checkpoint(model, optimizer, last_ckpt)
    for epoch in range(start_epoch+1, start_epoch+1+epochs):
        model.train()
        total_loss = 0.0
        for xb, yb in dataloader:
            xb = xb.to(DEVICE)
            yb = yb.to(DEVICE)
            pred = model(xb)
            loss = loss_fn(pred.squeeze(), yb.squeeze())
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
        avg = total_loss / len(dataloader)
        print(f'Epoch {epoch} avg_loss {avg:.6f}')
        save_checkpoint(model, optimizer, epoch, last_ckpt)
    print('Training finished')

if __name__ == '__main__':
    train_loop()
