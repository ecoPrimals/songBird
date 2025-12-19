# MNIST Digit Classification

**Difficulty:** Beginner  
**Time:** 15 minutes  
**Dataset:** MNIST (60,000 handwritten digits)  
**Goal:** Achieve >90% accuracy

---

## What You'll Learn

- Connect to EcoPrimals federation
- Submit your first ML training task
- Monitor progress
- Get results with cryptographic verification

---

## Quick Start

### 1. Check Requirements

```bash
pip install torch torchvision
```

### 2. Test Locally (Optional)

```bash
python train.py
```

This runs on your CPU/GPU to verify the code works.

### 3. Submit to Federation

```bash
python submit.py
```

This sends your task to the federation's GPUs!

---

## What Happens

1. **Your laptop** → Sends `train.py` to Songbird
2. **Songbird** → Finds available GPU (e.g., Northgate RTX 5090)
3. **ToadStool** → Provisions environment and runs training
4. **Results** → Return to you with cryptographic receipt

---

## Expected Results

- **Training time:** 3-5 minutes on federation GPU
- **Final accuracy:** ~95% (3 epochs)
- **Receipt:** Cryptographic proof of execution

---

## Experiment Ideas

Try modifying `train.py`:

1. **More layers:** Add another `fc` layer to the network
2. **More epochs:** Change `epochs=3` to `epochs=10`
3. **Different optimizer:** Try `optim.SGD` instead of `optim.Adam`
4. **Batch size:** Experiment with `batch_size=32` or `batch_size=128`

Each time you modify the code, run `python submit.py` again and compare the receipts!

---

## Files

- `train.py` - The training script (standard PyTorch)
- `submit.py` - Helper to submit to federation
- `requirements.txt` - Python dependencies

---

## Troubleshooting

**"No SONGBIRD_URL set"**
- Ask your instructor for the federation URL
- Set it: `export SONGBIRD_URL=ws://192.168.1.144:8080`

**"Connection refused"**
- Check you're on the same network as the federation
- Verify the URL is correct

**"Task failed"**
- Check your `train.py` code for errors
- Make sure all imports are correct
- Ask your instructor for help

---

## Next Steps

Once you complete this project, try:
- `02-cifar10-images` - More complex image classification
- `03-sentiment-analysis` - Natural language processing
- Modify this project to improve accuracy!

