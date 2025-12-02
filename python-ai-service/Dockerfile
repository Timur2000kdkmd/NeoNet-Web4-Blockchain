FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
# NOTE: For PyTorch, choose appropriate wheel for your platform.
# Example for CPU:
# RUN pip install --no-cache-dir -r requirements.txt && pip install torch --index-url https://download.pytorch.org/whl/cpu
RUN pip install --no-cache-dir -r requirements.txt
COPY app ./app
COPY pytorch_trainer.py ./pytorch_trainer.py
COPY trainer.py ./trainer.py
COPY worker.py ./worker.py
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
