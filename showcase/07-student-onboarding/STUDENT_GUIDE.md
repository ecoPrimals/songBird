# 📚 Student Guide - EcoPrimals Federated ML

**Welcome!** This guide will walk you through using the EcoPrimals federation to train machine learning models from your laptop.

---

## Table of Contents

1. [What is This?](#what-is-this)
2. [Setup](#setup)
3. [Your First Task](#your-first-task)
4. [Understanding Results](#understanding-results)
5. [Next Steps](#next-steps)
6. [Troubleshooting](#troubleshooting)

---

## What is This?

### The Problem

Training machine learning models requires:
- Expensive GPUs ($500-$2000+)
- Cloud credits ($$$)
- Long wait times on university clusters
- Complex infrastructure setup

### The EcoPrimals Solution

**Train ML models on distributed GPUs from your laptop—no hardware, no cloud bills, no setup.**

You write standard Python code (PyTorch, TensorFlow, etc.). The federation:
- Finds available GPUs automatically
- Runs your code on remote hardware
- Returns results with cryptographic verification
- Costs you nothing

### How It Works

```
Your Laptop
    ↓ (Submit task)
Songbird Coordinator
    ↓ (Route to available GPU)
ToadStool Compute Nodes
    ↓ (Train your model)
Results + Cryptographic Receipt
    ↓ (Return to you)
Your Laptop
```

---

## Setup

### Prerequisites

- Python 3.8 or newer
- pip (Python package manager)
- Network access to federation (ask instructor for URL)

### Step 1: Install Client (2 minutes)

```bash
# Navigate to client directory
cd showcase/07-student-onboarding/client

# Install dependencies
pip install -r requirements.txt

# Install client
pip install -e .
```

### Step 2: Get Federation URL

Ask your instructor for the **Songbird URL**. It will look like:
- `ws://192.168.1.144:8080` (local network)
- `wss://songbird.university.edu` (internet, coming soon)

Set it as an environment variable:

```bash
# On Mac/Linux
export SONGBIRD_URL="ws://192.168.1.144:8080"

# On Windows (PowerShell)
$env:SONGBIRD_URL="ws://192.168.1.144:8080"

# On Windows (CMD)
set SONGBIRD_URL=ws://192.168.1.144:8080
```

**Tip:** Add this to your shell profile so you don't have to set it every time!

### Step 3: Test Connection

```bash
python -m ecoprimals_client.connect
```

You should see:
```
🎵 Connecting to Songbird...
✅ Connected to Federation!
   Available nodes: 6
   Total GPUs: 6 (88GB VRAM)
```

If this works, **you're ready to go!** 🎉

---

## Your First Task

Let's train an MNIST digit classifier on the federation.

### Step 1: Navigate to Project

```bash
cd projects/01-mnist-digits
```

### Step 2: Review the Code (Optional)

Open `train.py` and look at the code. **It's standard PyTorch!** No special APIs, no federation-specific code. Just normal ML training.

### Step 3: Submit to Federation

```bash
python submit.py
```

You'll see:
```
🎵 Connecting to Songbird at ws://192.168.1.144:8080...
✅ Connected to Federation!

🚀 Submitting task: train.py
   Dataset: mnist
   GPU: Required

✅ Task accepted!
   Task ID: task-abc123
   Allocated to: Northgate
   GPU: RTX 5090 (24GB)

⏳ Waiting for task to complete...
```

### Step 4: Watch It Train

The client will show real-time progress as your model trains on the remote GPU. This takes 3-5 minutes.

### Step 5: Get Results

When complete, you'll see:
```
============================================================
📊 RESULTS
============================================================
   final_test_accuracy: 95.12
   final_loss: 0.1534
   training_time_seconds: 187.3
   device: cuda

📜 Cryptographic receipt saved to: receipt_task-abc123.json
============================================================
```

**Congratulations!** You just trained a neural network on distributed infrastructure! 🎉

---

## Understanding Results

### The Results JSON

Your results are saved in `results.json`:

```json
{
  "model": "SimpleNet",
  "dataset": "MNIST",
  "final_test_accuracy": 95.12,
  "final_loss": 0.1534,
  "epochs": 3,
  "training_time_seconds": 187.3,
  "device": "cuda",
  "history": [...]
}
```

This shows:
- **What model** you trained
- **What dataset** you used
- **How accurate** it became
- **How long** it took
- **Where** it ran (GPU)

### The Cryptographic Receipt

The receipt (`receipt_task-abc123.json`) proves:

1. **What code ran** - Exact script
2. **When it ran** - Timestamp
3. **Where it ran** - Which node/GPU
4. **What results were** - Metrics
5. **Cryptographic signature** - Proof it's authentic

**Why this matters:** You can prove your work is reproducible. Anyone can verify:
- The exact code you ran
- The exact results you got
- That it really happened (not fabricated)

This is important for:
- Academic integrity
- Research reproducibility
- Collaborative projects

---

## Next Steps

### Experiment with the Code

Try modifying `train.py`:

**1. Add more layers:**
```python
class SimpleNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.flatten = nn.Flatten()
        self.fc1 = nn.Linear(28 * 28, 256)  # Bigger!
        self.relu1 = nn.ReLU()
        self.fc2 = nn.Linear(256, 128)      # Extra layer!
        self.relu2 = nn.ReLU()
        self.fc3 = nn.Linear(128, 10)
```

**2. Train longer:**
```python
epochs = 10  # Instead of 3
```

**3. Try different optimizers:**
```python
optimizer = optim.SGD(model.parameters(), lr=0.01, momentum=0.9)
```

Each time you modify the code:
1. Save `train.py`
2. Run `python submit.py`
3. Compare the new receipt with your previous ones!

### Try More Complex Projects

Once you're comfortable:
- **02-cifar10-images** - Color image classification
- **03-sentiment-analysis** - Natural language processing
- **Your own project!** - Adapt any Kaggle competition

---

## Troubleshooting

### "No SONGBIRD_URL set"

**Problem:** Environment variable not configured.

**Solution:**
```bash
export SONGBIRD_URL="ws://192.168.1.144:8080"  # Ask instructor for URL
```

### "Connection refused"

**Problem:** Can't reach Songbird.

**Solutions:**
1. Check you're on the same network (WiFi/Ethernet)
2. Verify the URL with your instructor
3. Make sure Songbird is running (ask instructor)

### "Task failed"

**Problem:** Your code has an error.

**Solutions:**
1. Test locally first: `python train.py`
2. Check the error message in the result
3. Verify all imports are correct
4. Ask your instructor for help

### "Import error: No module named X"

**Problem:** Missing Python package.

**Solution:**
```bash
pip install torch torchvision  # Or whatever package is missing
```

### Still Stuck?

**Ask your instructor!** They can:
- Verify federation is running
- Check your connection
- Help debug your code
- Provide additional resources

---

## Tips for Success

### 1. Test Locally First

Before submitting to federation:
```bash
python train.py
```

This catches syntax errors and import issues quickly.

### 2. Start Simple

Don't try to train GPT-4 on your first task. Start with MNIST, then increase complexity.

### 3. Save Your Receipts

Keep all your cryptographic receipts! They're proof of your work and useful for comparing experiments.

### 4. Read the Code

Even if you don't fully understand PyTorch yet, read through `train.py`. ML code is just Python—you can learn it!

### 5. Experiment Freely

GPU time on the federation is free (for students). Try things! Break things! Learn!

---

## What You're Learning

By using the EcoPrimals federation, you're learning:

### Machine Learning
- Neural network architectures
- Training loops and optimization
- Evaluation and metrics
- Hyperparameter tuning

### Distributed Systems
- Client-server architecture
- Remote procedure calls
- Asynchronous programming
- Resource scheduling

### Cryptography
- Digital signatures
- Verification and provenance
- Reproducible research
- Trust models

### Infrastructure
- Compute sovereignty
- Federated coordination
- Resource sharing
- Capability-based systems

**You're not just learning ML—you're learning how to build the future of computing!** 🚀

---

## Questions?

- Check `docs/FAQ.md` for common questions
- Ask your instructor during office hours
- Collaborate with classmates
- Read the tutorials in `tutorials/`

**Happy learning!** 🌿✨

